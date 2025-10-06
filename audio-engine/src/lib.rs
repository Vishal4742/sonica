//! Sonica Audio Engine - Ultra-fast audio fingerprinting for music recognition
//! 
//! This crate provides high-performance audio processing capabilities optimized
//! for music recognition with SIMD operations and parallel processing.
//! 
//! # WebAssembly Support
//! 
//! This crate can be compiled to WebAssembly for client-side audio processing:
//! 
//! ```bash
//! wasm-pack build --target web
//! ```
//! 
//! # Usage
//! 
//! ## Server-side (Rust)
//! 
//! ```rust
//! use sonica_audio_engine::AudioEngine;
//! 
//! let engine = AudioEngine::new().await?;
//! let fingerprint = engine.process_audio(&audio_data).await?;
//! let result = engine.recognize(&fingerprint).await?;
//! ```
//! 
//! ## Client-side (WebAssembly)
//! 
//! ```javascript
//! import { WasmAudioProcessor } from './sonica-audio-engine.js';
//! 
//! const processor = new WasmAudioProcessor(44100, 4096, 0.5);
//! const fingerprint = processor.process_audio(audioData);
//! ```

pub mod audio;
pub mod fingerprint;
pub mod advanced_fingerprint;
pub mod optimized_fingerprint;
pub mod similarity;
pub mod database;
pub mod config;
pub mod error;
pub mod vector_db;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};
use crate::vector_db::VectorDatabase;

/// Main audio engine for music recognition
pub struct AudioEngine {
    config: config::Config,
    database: Arc<database::Database>,
    vector_db: Arc<VectorDatabase>,
    fingerprint_cache: Arc<RwLock<lru::LruCache<String, fingerprint::Fingerprint>>>,
}

impl AudioEngine {
    /// Create a new audio engine instance
    pub async fn new() -> Result<Self, error::AudioEngineError> {
        let config = config::Config::load()?;
        let database = Arc::new(database::Database::new(&config).await?);
        
        // Initialize vector database
        let vector_db = Arc::new(VectorDatabase::new(
            config.vector_db.api_key.clone(),
            config.vector_db.environment.clone(),
            config.vector_db.index_name.clone(),
            config.vector_db.dimensions,
        ));
        
        // Initialize vector database connection
        vector_db.initialize().await?;
        
        // Initialize fingerprint cache with 10,000 entries
        let fingerprint_cache = Arc::new(RwLock::new(
            lru::LruCache::new(std::num::NonZeroUsize::new(10_000).unwrap())
        ));

        info!("Audio engine initialized successfully with vector database");
        
        Ok(Self {
            config,
            database,
            vector_db,
            fingerprint_cache,
        })
    }

    /// Process audio data and generate fingerprint
    pub async fn process_audio(&self, audio_data: &[f32]) -> Result<fingerprint::Fingerprint, error::AudioEngineError> {
        let start_time = std::time::Instant::now();
        
        // Preprocess audio
        let processed_audio = self.preprocess_audio(audio_data)?;
        
        // Generate fingerprint
        let fingerprint = self.generate_fingerprint(&processed_audio).await?;
        
        let processing_time = start_time.elapsed();
        info!(
            audio_size = audio_data.len(),
            processing_time_ms = processing_time.as_millis(),
            "Audio processing completed"
        );
        
        Ok(fingerprint)
    }

    /// Recognize song from audio fingerprint using vector database
    pub async fn recognize(&self, fingerprint: &fingerprint::Fingerprint) -> Result<Option<database::Song>, error::AudioEngineError> {
        let start_time = std::time::Instant::now();
        
        // Search for similar fingerprints using vector database
        let vector_results = self.vector_db.search_similar_fingerprints(
            fingerprint,
            self.config.recognition.max_candidates,
            None, // language filter
            None, // genre filter
        ).await?;
        
        if vector_results.is_empty() {
            return Ok(None);
        }
        
        // Get song details for the best matches
        let mut best_match: Option<database::Song> = None;
        let mut best_score = 0.0;
        
        for result in vector_results {
            if result.score > best_score && result.score > self.config.recognition.threshold {
                if let Some(song_id_str) = result.metadata.get("song_id") {
                    if let Some(song_id) = song_id_str.as_str() {
                        if let Ok(uuid) = uuid::Uuid::parse_str(song_id) {
                            if let Some(song) = self.database.get_song(&uuid).await? {
                                best_score = result.score;
                                best_match = Some(song);
                            }
                        }
                    }
                }
            }
        }
        
        let recognition_time = start_time.elapsed();
        info!(
            vector_results_found = vector_results.len(),
            recognition_time_ms = recognition_time.as_millis(),
            best_match_id = best_match.as_ref().map(|s| s.id.to_string()),
            best_score = best_score,
            "Vector-based recognition completed"
        );
        
        Ok(best_match)
    }

    /// Add new song to database and vector database
    pub async fn add_song(&self, song: database::Song, audio_data: &[f32]) -> Result<(), error::AudioEngineError> {
        // Generate fingerprint for new song
        let fingerprint = self.process_audio(audio_data).await?;
        
        // Store in PostgreSQL database
        self.database.add_song(song.clone(), fingerprint.clone()).await?;
        
        // Prepare metadata for vector database
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("title".to_string(), serde_json::Value::String(song.title));
        metadata.insert("artist".to_string(), serde_json::Value::String(song.artist));
        metadata.insert("language".to_string(), serde_json::Value::String(song.language));
        if let Some(genre) = song.genre {
            metadata.insert("genre".to_string(), serde_json::Value::String(genre));
        }
        if let Some(album) = song.album {
            metadata.insert("album".to_string(), serde_json::Value::String(album));
        }
        metadata.insert("popularity_score".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(song.popularity_score as f64).unwrap()));
        
        // Store in vector database
        self.vector_db.add_fingerprint(song.id, &fingerprint, metadata).await?;
        
        info!("New song added to database and vector database");
        Ok(())
    }

    /// Preprocess audio data for fingerprinting
    fn preprocess_audio(&self, audio_data: &[f32]) -> Result<Vec<f32>, error::AudioEngineError> {
        // Normalize audio
        let normalized = audio::normalize_audio(audio_data);
        
        // Apply noise reduction
        let denoised = audio::reduce_noise(&normalized);
        
        // Resample if needed
        let resampled = audio::resample_audio(&denoised, 44100)?;
        
        Ok(resampled)
    }

    /// Generate optimized fingerprint from processed audio
    async fn generate_fingerprint(&self, audio_data: &[f32]) -> Result<fingerprint::Fingerprint, error::AudioEngineError> {
        // Check cache first
        let cache_key = self.generate_cache_key(audio_data);
        {
            let mut cache = self.fingerprint_cache.write().await;
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }
        
        // Generate optimized fingerprint
        let optimized_fingerprint = optimized_fingerprint::OptimizedFingerprint::generate(audio_data, 44100)?;
        
        // Log performance metrics
        let metrics = optimized_fingerprint.get_performance_metrics();
        info!(
            optimized_processing_time = metrics.get("processing_time_ms").unwrap_or(&0.0),
            memory_usage_mb = metrics.get("memory_usage_mb").unwrap_or(&0.0),
            simd_operations = metrics.get("simd_operations").unwrap_or(&0.0),
            cache_hit_ratio = metrics.get("cache_hit_ratio").unwrap_or(&0.0),
            overall_confidence = metrics.get("overall_confidence").unwrap_or(&0.0),
            "Optimized fingerprint generation completed"
        );
        
        // Extract base fingerprint for compatibility
        let fingerprint = optimized_fingerprint.hash_fingerprint;
        
        // Cache the result
        {
            let mut cache = self.fingerprint_cache.write().await;
            cache.put(cache_key, fingerprint.clone());
        }
        
        Ok(fingerprint)
    }

    /// Find best matching song from candidates
    async fn find_best_match(
        &self,
        query_fingerprint: &fingerprint::Fingerprint,
        candidates: &[database::Song],
    ) -> Result<Option<database::Song>, error::AudioEngineError> {
        let mut best_match: Option<(database::Song, f32)> = None;
        let mut best_score = 0.0;
        
        for candidate in candidates {
            // Get candidate fingerprint
            let candidate_fingerprint = self.database.get_fingerprint(&candidate.id).await?;
            
            // Calculate similarity score
            let score = similarity::calculate_similarity(query_fingerprint, &candidate_fingerprint);
            
            if score > best_score && score > self.config.recognition_threshold {
                best_score = score;
                best_match = Some((candidate.clone(), score));
            }
        }
        
        Ok(best_match.map(|(song, _)| song))
    }

    /// Generate cache key for audio data
    fn generate_cache_key(&self, audio_data: &[f32]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        audio_data.len().hash(&mut hasher);
        // Hash first and last few samples for quick comparison
        if audio_data.len() > 100 {
            audio_data[..50].hash(&mut hasher);
            audio_data[audio_data.len()-50..].hash(&mut hasher);
        } else {
            audio_data.hash(&mut hasher);
        }
        
        format!("audio_{}", hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audio_engine_initialization() {
        let engine = AudioEngine::new().await;
        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_audio_processing() {
        let engine = AudioEngine::new().await.unwrap();
        
        // Generate test audio data (1 second of sine wave)
        let sample_rate = 44100;
        let duration = 1.0;
        let frequency = 440.0; // A4 note
        
        let mut audio_data = Vec::new();
        for i in 0..(sample_rate as f32 * duration) as usize {
            let sample = (2.0 * std::f32::consts::PI * frequency * i as f32 / sample_rate as f32).sin();
            audio_data.push(sample);
        }
        
        let fingerprint = engine.process_audio(&audio_data).await;
        assert!(fingerprint.is_ok());
    }
}
