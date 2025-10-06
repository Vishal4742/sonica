//! Configuration management for the audio engine

use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Database configuration
    pub database: DatabaseConfig,
    /// Redis configuration
    pub redis: RedisConfig,
    /// Vector database configuration
    pub vector_db: VectorDbConfig,
    /// Audio processing configuration
    pub audio: AudioConfig,
    /// Recognition configuration
    pub recognition: RecognitionConfig,
    /// Server configuration
    pub server: ServerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub max_connections: u32,
    pub connection_timeout: u64,
    pub key_prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorDbConfig {
    pub provider: String, // "pinecone" or "weaviate"
    pub api_key: String,
    pub environment: String,
    pub index_name: String,
    pub dimensions: u32,
    pub metric: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub window_size: usize,
    pub hop_size: usize,
    pub overlap: f32,
    pub min_duration: f32,
    pub max_duration: f32,
    pub noise_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognitionConfig {
    pub threshold: f32,
    pub max_candidates: usize,
    pub cache_ttl: u64,
    pub batch_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub max_request_size: usize,
    pub timeout: u64,
}

impl Config {
    /// Load configuration from environment variables and config files
    pub fn load() -> Result<Self, config::ConfigError> {
        let mut settings = config::Config::default();
        
        // Load from config file if it exists
        if let Ok(config_file) = env::var("SONICA_CONFIG_FILE") {
            settings = settings.add_source(config::File::with_name(&config_file));
        } else {
            // Try default config files
            for config_file in &["config.toml", "config.yaml", "config.json"] {
                if std::path::Path::new(config_file).exists() {
                    settings = settings.add_source(config::File::with_name(config_file));
                    break;
                }
            }
        }
        
        // Override with environment variables
        settings = settings.add_source(
            config::Environment::with_prefix("SONICA")
                .separator("_")
                .list_separator(",")
        );
        
        settings.try_deserialize()
    }
    
    /// Get default configuration
    pub fn default() -> Self {
        Self {
            database: DatabaseConfig {
                url: env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "postgresql://sonica:password@localhost/sonica".to_string()),
                max_connections: 100,
                min_connections: 10,
                connection_timeout: 30,
            },
            redis: RedisConfig {
                url: env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
                max_connections: 50,
                connection_timeout: 5,
                key_prefix: "sonica:".to_string(),
            },
            vector_db: VectorDbConfig {
                provider: env::var("VECTOR_DB_PROVIDER")
                    .unwrap_or_else(|_| "pinecone".to_string()),
                api_key: env::var("VECTOR_DB_API_KEY")
                    .unwrap_or_else(|_| "your-api-key".to_string()),
                environment: env::var("VECTOR_DB_ENVIRONMENT")
                    .unwrap_or_else(|_| "us-west1-gcp".to_string()),
                index_name: env::var("VECTOR_DB_INDEX_NAME")
                    .unwrap_or_else(|_| "sonica-music".to_string()),
                dimensions: 1024,
                metric: "cosine".to_string(),
            },
            audio: AudioConfig {
                sample_rate: 44100,
                window_size: 4096,
                hop_size: 2048,
                overlap: 0.5,
                min_duration: 3.0,
                max_duration: 30.0,
                noise_threshold: 0.01,
            },
            recognition: RecognitionConfig {
                threshold: 0.8,
                max_candidates: 100,
                cache_ttl: 3600, // 1 hour
                batch_size: 50,
            },
            server: ServerConfig {
                host: env::var("HOST")
                    .unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("PORT")
                    .unwrap_or_else(|_| "8000".to_string())
                    .parse()
                    .unwrap_or(8000),
                workers: num_cpus::get(),
                max_request_size: 10 * 1024 * 1024, // 10MB
                timeout: 30,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.audio.sample_rate, 44100);
        assert_eq!(config.recognition.threshold, 0.8);
        assert!(!config.database.url.is_empty());
    }

    #[test]
    fn test_config_validation() {
        let config = Config::default();
        
        // Validate audio config
        assert!(config.audio.sample_rate > 0);
        assert!(config.audio.window_size > 0);
        assert!(config.audio.overlap > 0.0 && config.audio.overlap < 1.0);
        assert!(config.audio.min_duration > 0.0);
        assert!(config.audio.max_duration > config.audio.min_duration);
        
        // Validate recognition config
        assert!(config.recognition.threshold > 0.0 && config.recognition.threshold <= 1.0);
        assert!(config.recognition.max_candidates > 0);
        assert!(config.recognition.cache_ttl > 0);
        
        // Validate server config
        assert!(config.server.port > 0);
        assert!(config.server.workers > 0);
        assert!(config.server.max_request_size > 0);
    }
}
