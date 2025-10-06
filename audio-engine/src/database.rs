//! Database operations for audio fingerprints and songs

use crate::fingerprint::Fingerprint;
use crate::error::AudioEngineError;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Song information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    pub id: Uuid,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub language: String,
    pub genre: Option<String>,
    pub duration: Option<i32>,
    pub release_year: Option<i32>,
    pub audio_url: Option<String>,
    pub artwork_url: Option<String>,
    pub popularity_score: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Recognition result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognitionResult {
    pub song: Song,
    pub confidence: f32,
    pub processing_time_ms: u64,
    pub matched_segments: Vec<MatchedSegment>,
}

/// Matched audio segment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchedSegment {
    pub start_time: f32,
    pub end_time: f32,
    pub confidence: f32,
}

/// Database operations
pub struct Database {
    pool: PgPool,
    redis: redis::Client,
}

impl Database {
    /// Create new database instance
    pub async fn new(config: &crate::config::Config) -> Result<Self, AudioEngineError> {
        // Initialize PostgreSQL connection pool
        let pool = PgPool::connect(&config.database.url).await?;
        
        // Initialize Redis client
        let redis = redis::Client::open(config.redis.url.as_str())?;
        
        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;
        
        Ok(Self { pool, redis })
    }
    
    /// Add new song to database
    pub async fn add_song(&self, song: Song, fingerprint: Fingerprint) -> Result<(), AudioEngineError> {
        let mut tx = self.pool.begin().await?;
        
        // Insert song
        let song_id = sqlx::query!(
            r#"
            INSERT INTO songs (id, title, artist, album, language, genre, duration, release_year, audio_url, artwork_url, popularity_score)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id
            "#,
            song.id,
            song.title,
            song.artist,
            song.album,
            song.language,
            song.genre,
            song.duration,
            song.release_year,
            song.audio_url,
            song.artwork_url,
            song.popularity_score
        )
        .fetch_one(&mut *tx)
        .await?;
        
        // Store fingerprint
        let fingerprint_data = fingerprint.to_bytes()?;
        sqlx::query!(
            r#"
            INSERT INTO fingerprints (song_id, fingerprint_data, created_at)
            VALUES ($1, $2, $3)
            "#,
            song_id.id,
            fingerprint_data,
            Utc::now()
        )
        .execute(&mut *tx)
        .await?;
        
        tx.commit().await?;
        
        // Cache the song
        self.cache_song(&song).await?;
        
        Ok(())
    }
    
    /// Get song by ID
    pub async fn get_song(&self, song_id: &Uuid) -> Result<Option<Song>, AudioEngineError> {
        // Try cache first
        if let Some(cached_song) = self.get_cached_song(song_id).await? {
            return Ok(Some(cached_song));
        }
        
        // Query database
        let row = sqlx::query!(
            r#"
            SELECT id, title, artist, album, language, genre, duration, release_year, audio_url, artwork_url, popularity_score, created_at, updated_at
            FROM songs
            WHERE id = $1
            "#,
            song_id
        )
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            let song = Song {
                id: row.id,
                title: row.title,
                artist: row.artist,
                album: row.album,
                language: row.language,
                genre: row.genre,
                duration: row.duration,
                release_year: row.release_year,
                audio_url: row.audio_url,
                artwork_url: row.artwork_url,
                popularity_score: row.popularity_score,
                created_at: row.created_at,
                updated_at: row.updated_at,
            };
            
            // Cache the song
            self.cache_song(&song).await?;
            
            Ok(Some(song))
        } else {
            Ok(None)
        }
    }
    
    /// Search for similar songs
    pub async fn search_similar(&self, fingerprint: &Fingerprint, limit: usize) -> Result<Vec<Song>, AudioEngineError> {
        // This is a simplified implementation
        // In production, you would use a vector database like Pinecone or Weaviate
        
        let rows = sqlx::query!(
            r#"
            SELECT s.id, s.title, s.artist, s.album, s.language, s.genre, s.duration, s.release_year, s.audio_url, s.artwork_url, s.popularity_score, s.created_at, s.updated_at
            FROM songs s
            ORDER BY s.popularity_score DESC
            LIMIT $1
            "#,
            limit as i64
        )
        .fetch_all(&self.pool)
        .await?;
        
        let songs = rows.into_iter().map(|row| Song {
            id: row.id,
            title: row.title,
            artist: row.artist,
            album: row.album,
            language: row.language,
            genre: row.genre,
            duration: row.duration,
            release_year: row.release_year,
            audio_url: row.audio_url,
            artwork_url: row.artwork_url,
            popularity_score: row.popularity_score,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }).collect();
        
        Ok(songs)
    }
    
    /// Get fingerprint for a song
    pub async fn get_fingerprint(&self, song_id: &Uuid) -> Result<Fingerprint, AudioEngineError> {
        let row = sqlx::query!(
            r#"
            SELECT fingerprint_data
            FROM fingerprints
            WHERE song_id = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            song_id
        )
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            Fingerprint::from_bytes(&row.fingerprint_data)
        } else {
            Err(AudioEngineError::SongNotFound { song_id: song_id.to_string() })
        }
    }
    
    /// Search songs by text
    pub async fn search_songs(&self, query: &str, limit: usize) -> Result<Vec<Song>, AudioEngineError> {
        let search_query = format!("%{}%", query);
        
        let rows = sqlx::query!(
            r#"
            SELECT id, title, artist, album, language, genre, duration, release_year, audio_url, artwork_url, popularity_score, created_at, updated_at
            FROM songs
            WHERE title ILIKE $1 OR artist ILIKE $1 OR album ILIKE $1
            ORDER BY popularity_score DESC, title ASC
            LIMIT $2
            "#,
            search_query,
            limit as i64
        )
        .fetch_all(&self.pool)
        .await?;
        
        let songs = rows.into_iter().map(|row| Song {
            id: row.id,
            title: row.title,
            artist: row.artist,
            album: row.album,
            language: row.language,
            genre: row.genre,
            duration: row.duration,
            release_year: row.release_year,
            audio_url: row.audio_url,
            artwork_url: row.artwork_url,
            popularity_score: row.popularity_score,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }).collect();
        
        Ok(songs)
    }
    
    /// Get popular songs
    pub async fn get_popular_songs(&self, language: Option<&str>, limit: usize) -> Result<Vec<Song>, AudioEngineError> {
        let rows = if let Some(lang) = language {
            sqlx::query!(
                r#"
                SELECT id, title, artist, album, language, genre, duration, release_year, audio_url, artwork_url, popularity_score, created_at, updated_at
                FROM songs
                WHERE language = $1
                ORDER BY popularity_score DESC
                LIMIT $2
                "#,
                lang,
                limit as i64
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query!(
                r#"
                SELECT id, title, artist, album, language, genre, duration, release_year, audio_url, artwork_url, popularity_score, created_at, updated_at
                FROM songs
                ORDER BY popularity_score DESC
                LIMIT $1
                "#,
                limit as i64
            )
            .fetch_all(&self.pool)
            .await?
        };
        
        let songs = rows.into_iter().map(|row| Song {
            id: row.id,
            title: row.title,
            artist: row.artist,
            album: row.album,
            language: row.language,
            genre: row.genre,
            duration: row.duration,
            release_year: row.release_year,
            audio_url: row.audio_url,
            artwork_url: row.artwork_url,
            popularity_score: row.popularity_score,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }).collect();
        
        Ok(songs)
    }
    
    /// Log recognition result
    pub async fn log_recognition(&self, song_id: &Uuid, confidence: f32, processing_time_ms: u64) -> Result<(), AudioEngineError> {
        sqlx::query!(
            r#"
            INSERT INTO recognition_logs (song_id, confidence, processing_time_ms, created_at)
            VALUES ($1, $2, $3, $4)
            "#,
            song_id,
            confidence,
            processing_time_ms as i64,
            Utc::now()
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    /// Cache song in Redis
    async fn cache_song(&self, song: &Song) -> Result<(), AudioEngineError> {
        let mut conn = self.redis.get_async_connection().await?;
        let key = format!("song:{}", song.id);
        let data = serde_json::to_string(song)?;
        
        redis::cmd("SETEX")
            .arg(&key)
            .arg(3600) // 1 hour TTL
            .arg(&data)
            .execute_async(&mut conn)
            .await?;
        
        Ok(())
    }
    
    /// Get cached song from Redis
    async fn get_cached_song(&self, song_id: &Uuid) -> Result<Option<Song>, AudioEngineError> {
        let mut conn = self.redis.get_async_connection().await?;
        let key = format!("song:{}", song_id);
        
        let data: Option<String> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut conn)
            .await?;
        
        if let Some(data) = data {
            let song: Song = serde_json::from_str(&data)?;
            Ok(Some(song))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fingerprint::Fingerprint;

    #[tokio::test]
    async fn test_database_operations() {
        // This test would require a test database
        // For now, we'll just test the data structures
        
        let song = Song {
            id: Uuid::new_v4(),
            title: "Test Song".to_string(),
            artist: "Test Artist".to_string(),
            album: Some("Test Album".to_string()),
            language: "en".to_string(),
            genre: Some("pop".to_string()),
            duration: Some(180),
            release_year: Some(2023),
            audio_url: Some("https://example.com/audio.mp3".to_string()),
            artwork_url: Some("https://example.com/artwork.jpg".to_string()),
            popularity_score: 0.8,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        assert_eq!(song.title, "Test Song");
        assert_eq!(song.artist, "Test Artist");
    }
}
