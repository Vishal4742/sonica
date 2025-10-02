use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use validator::Validate;

// Language enum for supported languages
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    Hindi,
    Bhojpuri,
    English,
    Other(String),
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::Hindi => write!(f, "hi"),
            Language::Bhojpuri => write!(f, "bh"),
            Language::English => write!(f, "en"),
            Language::Other(lang) => write!(f, "{}", lang),
        }
    }
}

impl FromStr for Language {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "hi" | "hindi" => Ok(Language::Hindi),
            "bh" | "bhojpuri" => Ok(Language::Bhojpuri),
            "en" | "english" => Ok(Language::English),
            other => Ok(Language::Other(other.to_string())),
        }
    }
}

// Song model matching the database schema
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Song {
    pub id: Uuid,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub language: String, // Stored as string in DB, converted to Language enum when needed
    pub duration: f64,
    pub r2_audio_url: Option<String>,
    pub r2_thumbnail_url: Option<String>,
    pub audio_key: Option<String>,
    pub thumbnail_key: Option<String>,
    pub recognition_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Song {
    #[allow(dead_code)]
    pub fn get_language(&self) -> Language {
        Language::from_str(&self.language).unwrap_or(Language::English)
    }

    #[allow(dead_code)]
    pub fn set_language(&mut self, language: Language) {
        self.language = language.to_string();
    }
}

impl fmt::Display for Song {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {} ({})", self.artist, self.title, self.album.as_deref().unwrap_or("Unknown Album"))
    }
}

// Fingerprint model for audio fingerprinting
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Fingerprint {
    pub id: Uuid,
    pub song_id: Uuid,
    pub hash: String,
    pub time_offset: f64,
    pub created_at: DateTime<Utc>,
}

impl fmt::Display for Fingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fingerprint {} at {:.3}s", self.hash, self.time_offset)
    }
}

// Recognition model for tracking song recognitions
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Recognition {
    pub id: Uuid,
    pub song_id: Uuid,
    pub recognized_at: DateTime<Utc>,
    pub confidence: f64,
    pub user_ip: Option<String>,
    pub processing_time_ms: Option<i32>,
    pub audio_duration: Option<f64>,
    pub created_at: DateTime<Utc>,
}

impl fmt::Display for Recognition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Recognition of song {} with {:.2}% confidence", 
               self.song_id, self.confidence * 100.0)
    }
}

// API Request/Response structs

// Request to create a new song
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateSongRequest {
    #[validate(length(min = 1, max = 255, message = "Title must be between 1 and 255 characters"))]
    pub title: String,
    
    #[validate(length(min = 1, max = 255, message = "Artist must be between 1 and 255 characters"))]
    pub artist: String,
    
    #[validate(length(max = 255, message = "Album name cannot exceed 255 characters"))]
    pub album: Option<String>,
    
    pub language: Language,
    
    #[validate(range(min = 0.1, max = 3600.0, message = "Duration must be between 0.1 and 3600 seconds"))]
    pub duration: f64,
    
    #[validate(url(message = "Invalid audio URL format"))]
    pub r2_audio_url: Option<String>,
    
    #[validate(url(message = "Invalid thumbnail URL format"))]
    pub r2_thumbnail_url: Option<String>,
    
    #[validate(length(max = 500, message = "Audio key cannot exceed 500 characters"))]
    pub audio_key: Option<String>,
    
    #[validate(length(max = 500, message = "Thumbnail key cannot exceed 500 characters"))]
    pub thumbnail_key: Option<String>,
}

// Response for song creation
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSongResponse {
    pub success: bool,
    pub message: String,
    pub song_id: Option<Uuid>,
    pub song: Option<Song>,
}

// Request for audio recognition
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct RecognizeAudioRequest {
    #[validate(length(min = 1, max = 32, message = "Hash must be between 1 and 32 characters"))]
    pub hash: String,
    
    #[validate(range(min = 0.0, max = 3600.0, message = "Time offset must be between 0 and 3600 seconds"))]
    pub time_offset: f64,
    
    pub user_ip: Option<String>,
}

// Result of audio matching
#[derive(Debug, Serialize, Deserialize)]
pub struct MatchResult {
    pub song: Song,
    pub confidence: f64,
    pub time_offset: f64,
    pub processing_time_ms: i32,
    pub matched_at: DateTime<Utc>,
}

// Response for recognition
#[derive(Debug, Serialize, Deserialize)]
pub struct RecognitionResponse {
    pub success: bool,
    pub message: String,
    pub r#match: Option<MatchResult>,
    pub recognition_id: Option<Uuid>,
}

// Pagination parameters
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PaginationParams {
    #[validate(range(min = 1, max = 1000, message = "Page must be between 1 and 1000"))]
    pub page: Option<u32>,
    
    #[validate(range(min = 1, max = 100, message = "Limit must be between 1 and 100"))]
    pub limit: Option<u32>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(20),
        }
    }
}

// Paginated songs response
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedSongs {
    pub songs: Vec<Song>,
    pub total: u64,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

// Song statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct SongStats {
    pub song_id: Uuid,
    pub total_recognitions: i64,
    pub avg_confidence: Option<f64>,
    pub avg_processing_time: Option<f64>,
    pub first_recognized: Option<DateTime<Utc>>,
    pub last_recognized: Option<DateTime<Utc>>,
}

// System statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStats {
    pub total_songs: u64,
    pub total_fingerprints: u64,
    pub total_recognitions: u64,
    pub avg_recognition_time_ms: Option<f64>,
    pub most_popular_song: Option<Song>,
    pub languages: Vec<(String, u64)>,
    pub created_at: DateTime<Utc>,
}

// Search parameters
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct SearchParams {
    #[validate(length(max = 255, message = "Search query cannot exceed 255 characters"))]
    pub query: Option<String>,
    
    pub artist: Option<String>,
    pub album: Option<String>,
    pub language: Option<Language>,
    
    #[validate(range(min = 0.0, max = 3600.0, message = "Min duration must be between 0 and 3600 seconds"))]
    pub min_duration: Option<f64>,
    
    #[validate(range(min = 0.0, max = 3600.0, message = "Max duration must be between 0 and 3600 seconds"))]
    pub max_duration: Option<f64>,
    
    #[validate(range(min = 0, message = "Min recognition count cannot be negative"))]
    pub min_recognition_count: Option<i32>,
}

// Bulk fingerprint creation
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct BulkFingerprintRequest {
    pub song_id: Uuid,
    
    #[validate(length(min = 1, max = 10000, message = "Must provide between 1 and 10000 fingerprints"))]
    pub fingerprints: Vec<FingerprintData>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct FingerprintData {
    #[validate(length(min = 1, max = 32, message = "Hash must be between 1 and 32 characters"))]
    pub hash: String,
    
    #[validate(range(min = 0.0, max = 3600.0, message = "Time offset must be between 0 and 3600 seconds"))]
    pub time_offset: f64,
}

// Error response
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
    pub code: Option<String>,
    pub details: Option<serde_json::Value>,
}

impl ErrorResponse {
    #[allow(dead_code)]
    pub fn new(error: String) -> Self {
        Self {
            success: false,
            error,
            code: None,
            details: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_code(error: String, code: String) -> Self {
        Self {
            success: false,
            error,
            code: Some(code),
            details: None,
        }
    }
}

// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
    pub database: String,
    pub timestamp: DateTime<Utc>,
}

// API response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    #[allow(dead_code)]
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
            error: None,
        }
    }

    #[allow(dead_code)]
    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            message: None,
            error: Some(error),
        }
    }

    #[allow(dead_code)]
    pub fn with_message(data: T, message: String) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: Some(message),
            error: None,
        }
    }
}
