use serde::{Deserialize, Serialize};
use std::env;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr};
use validator::{Validate, ValidationError};

/// Configuration error types
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    MissingVariable(String),
    InvalidUrl(String),
    InvalidPort(String),
    InvalidIpAddress(String),
    InvalidBoolean(String),
    InvalidInteger(String),
    InvalidFloat(String),
    ValidationError(String),
    IoError(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::MissingVariable(var) => write!(f, "Missing required environment variable: {}", var),
            ConfigError::InvalidUrl(url) => write!(f, "Invalid URL format: {}", url),
            ConfigError::InvalidPort(port) => write!(f, "Invalid port number: {}", port),
            ConfigError::InvalidIpAddress(ip) => write!(f, "Invalid IP address: {}", ip),
            ConfigError::InvalidBoolean(val) => write!(f, "Invalid boolean value: {}", val),
            ConfigError::InvalidInteger(val) => write!(f, "Invalid integer value: {}", val),
            ConfigError::InvalidFloat(val) => write!(f, "Invalid float value: {}", val),
            ConfigError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            ConfigError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

/// R2 (Cloudflare) credentials configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct R2Credentials {
    pub endpoint: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub bucket_name: String,
    pub region: String,
}

impl R2Credentials {
    /// Create a new R2Credentials instance
    pub fn new(
        endpoint: String,
        access_key_id: String,
        secret_access_key: String,
        bucket_name: String,
        region: String,
    ) -> Self {
        Self {
            endpoint,
            access_key_id,
            secret_access_key,
            bucket_name,
            region,
        }
    }

    /// Mask sensitive values for logging
    pub fn masked(&self) -> MaskedR2Credentials {
        MaskedR2Credentials {
            endpoint: self.endpoint.clone(),
            access_key_id: mask_string(&self.access_key_id, 4),
            secret_access_key: mask_string(&self.secret_access_key, 4),
            bucket_name: self.bucket_name.clone(),
            region: self.region.clone(),
        }
    }
}

/// Masked version of R2Credentials for safe logging
#[derive(Debug, Clone, Serialize)]
pub struct MaskedR2Credentials {
    pub endpoint: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub bucket_name: String,
    pub region: String,
}

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Config {
    // Server Configuration
    #[validate(range(min = 1, max = 65535))]
    pub port: u16,
    pub host: IpAddr,
    pub log_level: String,
    
    // Database Configuration
    pub database_url: String,
    pub database_max_connections: u32,
    pub database_acquire_timeout: u64,
    
    // Redis Configuration
    pub redis_url: String,
    pub redis_max_connections: u32,
    pub redis_connection_timeout: u64,
    
    // R2 Storage Configuration
    pub r2_credentials: Option<R2Credentials>,
    
    // Audio Processing Configuration
    pub max_audio_file_size: u64,
    pub allowed_audio_formats: Vec<String>,
    pub audio_upload_path: String,
    pub temp_audio_path: String,
    
    // Machine Learning Configuration
    pub model_path: String,
    pub default_model_name: String,
    pub default_language: String,
    #[validate(range(min = 0.0, max = 1.0))]
    pub confidence_threshold: f64,
    
    // Audio Processing Settings
    pub sample_rate: u32,
    pub channels: u16,
    pub bit_depth: u16,
    pub max_audio_duration: u32,
    
    // External API Configuration
    pub openai_api_key: Option<String>,
    pub google_speech_api_key: Option<String>,
    pub azure_speech_key: Option<String>,
    pub azure_speech_region: Option<String>,
    
    // Processing Queue Configuration
    pub max_concurrent_jobs: u32,
    pub job_timeout_seconds: u64,
    pub cleanup_temp_files_after_hours: u32,
    
    // Security Configuration
    pub jwt_secret: String,
    pub session_timeout_minutes: u32,
    pub rate_limit_requests_per_minute: u32,
    
    // CORS Configuration
    pub cors_origins: Vec<String>,
    pub cors_allow_credentials: bool,
}

impl Config {
    /// Create a new Config instance with default values
    pub fn new() -> Self {
        Self {
            port: 3000,
            host: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            log_level: "info".to_string(),
            database_url: String::new(),
            database_max_connections: 10,
            database_acquire_timeout: 30,
            redis_url: String::new(),
            redis_max_connections: 10,
            redis_connection_timeout: 5,
            r2_credentials: None,
            max_audio_file_size: 52_428_800, // 50MB
            allowed_audio_formats: vec!["wav".to_string(), "mp3".to_string(), "flac".to_string()],
            audio_upload_path: "./uploads/audio".to_string(),
            temp_audio_path: "./temp/audio".to_string(),
            model_path: "./models".to_string(),
            default_model_name: "whisper-base".to_string(),
            default_language: "en-US".to_string(),
            confidence_threshold: 0.5,
            sample_rate: 16000,
            channels: 1,
            bit_depth: 16,
            max_audio_duration: 300, // 5 minutes
            openai_api_key: None,
            google_speech_api_key: None,
            azure_speech_key: None,
            azure_speech_region: None,
            max_concurrent_jobs: 5,
            job_timeout_seconds: 300,
            cleanup_temp_files_after_hours: 24,
            jwt_secret: String::new(),
            session_timeout_minutes: 60,
            rate_limit_requests_per_minute: 100,
            cors_origins: vec!["*".to_string()],
            cors_allow_credentials: false,
        }
    }

    /// Load configuration from environment variables
    pub fn load() -> Result<Self, ConfigError> {
        // Load .env file
        dotenvy::dotenv().map_err(|e| ConfigError::IoError(e.to_string()))?;

        let mut config = Self::new();

        // Server Configuration
        let port_str = get_env_var("PORT")?;
        config.port = port_str.parse::<u16>()
            .map_err(|_| ConfigError::InvalidPort(port_str))?;
        
        let host_str = get_env_var("HOST")?;
        config.host = host_str.parse::<IpAddr>()
            .map_err(|_| ConfigError::InvalidIpAddress(host_str))?;
        
        config.log_level = get_env_var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

        // Database Configuration
        config.database_url = get_env_var("DATABASE_URL")?;
        config.database_max_connections = get_env_var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<u32>()
            .map_err(|_| ConfigError::InvalidInteger("DATABASE_MAX_CONNECTIONS".to_string()))?;
        
        config.database_acquire_timeout = get_env_var("DATABASE_ACQUIRE_TIMEOUT")
            .unwrap_or_else(|_| "30".to_string())
            .parse::<u64>()
            .map_err(|_| ConfigError::InvalidInteger("DATABASE_ACQUIRE_TIMEOUT".to_string()))?;

        // Redis Configuration
        config.redis_url = get_env_var("REDIS_URL")?;
        config.redis_max_connections = get_env_var("REDIS_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<u32>()
            .map_err(|_| ConfigError::InvalidInteger("REDIS_MAX_CONNECTIONS".to_string()))?;
        
        config.redis_connection_timeout = get_env_var("REDIS_CONNECTION_TIMEOUT")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u64>()
            .map_err(|_| ConfigError::InvalidInteger("REDIS_CONNECTION_TIMEOUT".to_string()))?;

        // R2 Storage Configuration (Optional)
        if let Ok(endpoint) = get_env_var("R2_ENDPOINT") {
            let r2_creds = R2Credentials::new(
                endpoint,
                get_env_var("R2_ACCESS_KEY_ID").map_err(|e| e)?,
                get_env_var("R2_SECRET_ACCESS_KEY").map_err(|e| e)?,
                get_env_var("R2_BUCKET_NAME").map_err(|e| e)?,
                get_env_var("R2_REGION").map_err(|e| e)?,
            );
            config.r2_credentials = Some(r2_creds);
        }

        // Audio Processing Configuration
        config.max_audio_file_size = get_env_var("MAX_AUDIO_FILE_SIZE")
            .unwrap_or_else(|_| "52428800".to_string())
            .parse::<u64>()
            .map_err(|_| ConfigError::InvalidInteger("MAX_AUDIO_FILE_SIZE".to_string()))?;
        
        config.allowed_audio_formats = get_env_var("ALLOWED_AUDIO_FORMATS")
            .unwrap_or_else(|_| "wav,mp3,flac,m4a,aac,ogg".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        
        config.audio_upload_path = get_env_var("AUDIO_UPLOAD_PATH")
            .unwrap_or_else(|_| "./uploads/audio".to_string());
        
        config.temp_audio_path = get_env_var("TEMP_AUDIO_PATH")
            .unwrap_or_else(|_| "./temp/audio".to_string());

        // Machine Learning Configuration
        config.model_path = get_env_var("MODEL_PATH")
            .unwrap_or_else(|_| "./models".to_string());
        
        config.default_model_name = get_env_var("DEFAULT_MODEL_NAME")
            .unwrap_or_else(|_| "whisper-base".to_string());
        
        config.default_language = get_env_var("DEFAULT_LANGUAGE")
            .unwrap_or_else(|_| "en-US".to_string());
        
        config.confidence_threshold = get_env_var("CONFIDENCE_THRESHOLD")
            .unwrap_or_else(|_| "0.5".to_string())
            .parse::<f64>()
            .map_err(|_| ConfigError::InvalidFloat("CONFIDENCE_THRESHOLD".to_string()))?;

        // Audio Processing Settings
        config.sample_rate = get_env_var("SAMPLE_RATE")
            .unwrap_or_else(|_| "16000".to_string())
            .parse::<u32>()
            .map_err(|_| ConfigError::InvalidInteger("SAMPLE_RATE".to_string()))?;
        
        config.channels = get_env_var("CHANNELS")
            .unwrap_or_else(|_| "1".to_string())
            .parse::<u16>()
            .map_err(|_| ConfigError::InvalidInteger("CHANNELS".to_string()))?;
        
        config.bit_depth = get_env_var("BIT_DEPTH")
            .unwrap_or_else(|_| "16".to_string())
            .parse::<u16>()
            .map_err(|_| ConfigError::InvalidInteger("BIT_DEPTH".to_string()))?;
        
        config.max_audio_duration = get_env_var("MAX_AUDIO_DURATION")
            .unwrap_or_else(|_| "300".to_string())
            .parse::<u32>()
            .map_err(|_| ConfigError::InvalidInteger("MAX_AUDIO_DURATION".to_string()))?;

        // External API Configuration (Optional)
        config.openai_api_key = get_env_var("OPENAI_API_KEY").ok();
        config.google_speech_api_key = get_env_var("GOOGLE_SPEECH_API_KEY").ok();
        config.azure_speech_key = get_env_var("AZURE_SPEECH_KEY").ok();
        config.azure_speech_region = get_env_var("AZURE_SPEECH_REGION").ok();

        // Processing Queue Configuration
        config.max_concurrent_jobs = get_env_var("MAX_CONCURRENT_JOBS")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u32>()
            .map_err(|_| ConfigError::InvalidInteger("MAX_CONCURRENT_JOBS".to_string()))?;
        
        config.job_timeout_seconds = get_env_var("JOB_TIMEOUT_SECONDS")
            .unwrap_or_else(|_| "300".to_string())
            .parse::<u64>()
            .map_err(|_| ConfigError::InvalidInteger("JOB_TIMEOUT_SECONDS".to_string()))?;
        
        config.cleanup_temp_files_after_hours = get_env_var("CLEANUP_TEMP_FILES_AFTER_HOURS")
            .unwrap_or_else(|_| "24".to_string())
            .parse::<u32>()
            .map_err(|_| ConfigError::InvalidInteger("CLEANUP_TEMP_FILES_AFTER_HOURS".to_string()))?;

        // Security Configuration
        config.jwt_secret = get_env_var("JWT_SECRET")?;
        config.session_timeout_minutes = get_env_var("SESSION_TIMEOUT_MINUTES")
            .unwrap_or_else(|_| "60".to_string())
            .parse::<u32>()
            .map_err(|_| ConfigError::InvalidInteger("SESSION_TIMEOUT_MINUTES".to_string()))?;
        
        config.rate_limit_requests_per_minute = get_env_var("RATE_LIMIT_REQUESTS_PER_MINUTE")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<u32>()
            .map_err(|_| ConfigError::InvalidInteger("RATE_LIMIT_REQUESTS_PER_MINUTE".to_string()))?;

        // CORS Configuration
        config.cors_origins = get_env_var("CORS_ORIGINS")
            .unwrap_or_else(|_| "*".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        
        config.cors_allow_credentials = get_env_var("CORS_ALLOW_CREDENTIALS")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .map_err(|_| ConfigError::InvalidBoolean("CORS_ALLOW_CREDENTIALS".to_string()))?;

        // Validate the configuration
        if let Err(e) = config.validate() {
            return Err(ConfigError::ValidationError(format!("{:?}", e)));
        }

        Ok(config)
    }

    /// Get a masked version of the configuration for safe logging
    pub fn masked(&self) -> MaskedConfig {
        MaskedConfig {
            port: self.port,
            host: self.host,
            log_level: self.log_level.clone(),
            database_url: mask_url(&self.database_url),
            database_max_connections: self.database_max_connections,
            database_acquire_timeout: self.database_acquire_timeout,
            redis_url: mask_url(&self.redis_url),
            redis_max_connections: self.redis_max_connections,
            redis_connection_timeout: self.redis_connection_timeout,
            r2_credentials: self.r2_credentials.as_ref().map(|r2| r2.masked()),
            max_audio_file_size: self.max_audio_file_size,
            allowed_audio_formats: self.allowed_audio_formats.clone(),
            audio_upload_path: self.audio_upload_path.clone(),
            temp_audio_path: self.temp_audio_path.clone(),
            model_path: self.model_path.clone(),
            default_model_name: self.default_model_name.clone(),
            default_language: self.default_language.clone(),
            confidence_threshold: self.confidence_threshold,
            sample_rate: self.sample_rate,
            channels: self.channels,
            bit_depth: self.bit_depth,
            max_audio_duration: self.max_audio_duration,
            openai_api_key: self.openai_api_key.as_ref().map(|k| mask_string(k, 4)),
            google_speech_api_key: self.google_speech_api_key.as_ref().map(|k| mask_string(k, 4)),
            azure_speech_key: self.azure_speech_key.as_ref().map(|k| mask_string(k, 4)),
            azure_speech_region: self.azure_speech_region.clone(),
            max_concurrent_jobs: self.max_concurrent_jobs,
            job_timeout_seconds: self.job_timeout_seconds,
            cleanup_temp_files_after_hours: self.cleanup_temp_files_after_hours,
            jwt_secret: mask_string(&self.jwt_secret, 4),
            session_timeout_minutes: self.session_timeout_minutes,
            rate_limit_requests_per_minute: self.rate_limit_requests_per_minute,
            cors_origins: self.cors_origins.clone(),
            cors_allow_credentials: self.cors_allow_credentials,
        }
    }
}

/// Masked version of Config for safe logging
#[derive(Debug, Clone, Serialize)]
pub struct MaskedConfig {
    pub port: u16,
    pub host: IpAddr,
    pub log_level: String,
    pub database_url: String,
    pub database_max_connections: u32,
    pub database_acquire_timeout: u64,
    pub redis_url: String,
    pub redis_max_connections: u32,
    pub redis_connection_timeout: u64,
    pub r2_credentials: Option<MaskedR2Credentials>,
    pub max_audio_file_size: u64,
    pub allowed_audio_formats: Vec<String>,
    pub audio_upload_path: String,
    pub temp_audio_path: String,
    pub model_path: String,
    pub default_model_name: String,
    pub default_language: String,
    pub confidence_threshold: f64,
    pub sample_rate: u32,
    pub channels: u16,
    pub bit_depth: u16,
    pub max_audio_duration: u32,
    pub openai_api_key: Option<String>,
    pub google_speech_api_key: Option<String>,
    pub azure_speech_key: Option<String>,
    pub azure_speech_region: Option<String>,
    pub max_concurrent_jobs: u32,
    pub job_timeout_seconds: u64,
    pub cleanup_temp_files_after_hours: u32,
    pub jwt_secret: String,
    pub session_timeout_minutes: u32,
    pub rate_limit_requests_per_minute: u32,
    pub cors_origins: Vec<String>,
    pub cors_allow_credentials: bool,
}

/// Get environment variable with error handling
fn get_env_var(key: &str) -> Result<String, ConfigError> {
    env::var(key).map_err(|_| ConfigError::MissingVariable(key.to_string()))
}

/// Validate IP address format
fn validate_ip_address(_ip: &IpAddr) -> Result<(), ValidationError> {
    // Basic validation - IP addresses are already validated by FromStr
    Ok(())
}

/// Mask sensitive string values for logging
fn mask_string(value: &str, visible_chars: usize) -> String {
    if value.len() <= visible_chars {
        "*".repeat(value.len())
    } else {
        format!("{}{}", &value[..visible_chars], "*".repeat(value.len() - visible_chars))
    }
}

/// Mask URL credentials for logging
fn mask_url(url: &str) -> String {
    if let Ok(parsed) = url::Url::parse(url) {
        if let Some(password) = parsed.password() {
            url.replace(password, "***")
        } else {
            url.to_string()
        }
    } else {
        url.to_string()
    }
}

/// Load and validate configuration
pub fn load_config() -> Result<Config, ConfigError> {
    let config = Config::load()?;
    
    // Log configuration (masked)
    log::info!("Configuration loaded successfully: {:?}", config.masked());
    
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_validation() {
        // Set required environment variables
        env::set_var("DATABASE_URL", "postgresql://user:pass@localhost/db");
        env::set_var("REDIS_URL", "redis://localhost:6379");
        env::set_var("JWT_SECRET", "test-secret");
        
        let config = Config::load();
        assert!(config.is_ok());
    }

    #[test]
    fn test_missing_required_vars() {
        // Clear environment
        env::remove_var("DATABASE_URL");
        env::remove_var("REDIS_URL");
        env::remove_var("JWT_SECRET");
        
        let config = Config::load();
        assert!(config.is_err());
    }

    #[test]
    fn test_mask_string() {
        assert_eq!(mask_string("secret", 2), "se****");
        assert_eq!(mask_string("ab", 2), "ab");
        assert_eq!(mask_string("a", 2), "*");
    }
}
