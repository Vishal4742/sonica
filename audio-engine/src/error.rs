//! Error types for the audio engine

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AudioEngineError {
    #[error("Audio processing error: {0}")]
    AudioProcessing(#[from] anyhow::Error),
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Invalid audio format: {0}")]
    InvalidAudioFormat(String),
    
    #[error("Audio too short: {duration}s (minimum: {minimum}s)")]
    AudioTooShort { duration: f32, minimum: f32 },
    
    #[error("Audio too long: {duration}s (maximum: {maximum}s)")]
    AudioTooLong { duration: f32, maximum: f32 },
    
    #[error("Recognition failed: {reason}")]
    RecognitionFailed { reason: String },
    
    #[error("Song not found: {song_id}")]
    SongNotFound { song_id: String },
    
    #[error("Rate limit exceeded: {limit} requests per {window}")]
    RateLimitExceeded { limit: u32, window: String },
    
    #[error("Authentication failed: {reason}")]
    AuthenticationFailed { reason: String },
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl AudioEngineError {
    /// Get HTTP status code for the error
    pub fn status_code(&self) -> u16 {
        match self {
            AudioEngineError::InvalidAudioFormat(_) => 400,
            AudioEngineError::AudioTooShort { .. } => 400,
            AudioEngineError::AudioTooLong { .. } => 400,
            AudioEngineError::RecognitionFailed { .. } => 422,
            AudioEngineError::SongNotFound { .. } => 404,
            AudioEngineError::RateLimitExceeded { .. } => 429,
            AudioEngineError::AuthenticationFailed { .. } => 401,
            AudioEngineError::Database(_) => 500,
            AudioEngineError::Redis(_) => 500,
            AudioEngineError::Internal(_) => 500,
            _ => 500,
        }
    }
    
    /// Get error code for API responses
    pub fn error_code(&self) -> &'static str {
        match self {
            AudioEngineError::InvalidAudioFormat(_) => "INVALID_AUDIO_FORMAT",
            AudioEngineError::AudioTooShort { .. } => "AUDIO_TOO_SHORT",
            AudioEngineError::AudioTooLong { .. } => "AUDIO_TOO_LONG",
            AudioEngineError::RecognitionFailed { .. } => "RECOGNITION_FAILED",
            AudioEngineError::SongNotFound { .. } => "SONG_NOT_FOUND",
            AudioEngineError::RateLimitExceeded { .. } => "RATE_LIMIT_EXCEEDED",
            AudioEngineError::AuthenticationFailed { .. } => "AUTHENTICATION_FAILED",
            AudioEngineError::Database(_) => "DATABASE_ERROR",
            AudioEngineError::Redis(_) => "CACHE_ERROR",
            AudioEngineError::Internal(_) => "INTERNAL_ERROR",
            _ => "UNKNOWN_ERROR",
        }
    }
}
