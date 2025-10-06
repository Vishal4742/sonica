//! Vector database integration for ultra-fast similarity search
//! 
//! This module provides integration with Pinecone vector database for
//! sub-millisecond similarity search of audio fingerprints.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use uuid::Uuid;

/// Vector database client for similarity search
pub struct VectorDatabase {
    api_key: String,
    environment: String,
    index_name: String,
    dimensions: u32,
    client: reqwest::Client,
    base_url: String,
}

/// Vector search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchResult {
    pub id: String,
    pub score: f32,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Vector upsert request
#[derive(Debug, Serialize)]
pub struct VectorUpsertRequest {
    pub vectors: Vec<VectorData>,
    pub namespace: Option<String>,
}

/// Vector data for upsert
#[derive(Debug, Serialize)]
pub struct VectorData {
    pub id: String,
    pub values: Vec<f32>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Vector query request
#[derive(Debug, Serialize)]
pub struct VectorQueryRequest {
    pub vector: Vec<f32>,
    pub top_k: u32,
    pub include_metadata: bool,
    pub namespace: Option<String>,
    pub filter: Option<HashMap<String, serde_json::Value>>,
}

/// Vector query response
#[derive(Debug, Deserialize)]
pub struct VectorQueryResponse {
    pub matches: Vec<VectorMatch>,
    pub namespace: String,
}

/// Vector match result
#[derive(Debug, Deserialize)]
pub struct VectorMatch {
    pub id: String,
    pub score: f32,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Index statistics
#[derive(Debug, Deserialize)]
pub struct IndexStats {
    pub total_vector_count: u64,
    pub dimension: u32,
    pub index_fullness: f32,
}

impl VectorDatabase {
    /// Create a new vector database client
    pub fn new(
        api_key: String,
        environment: String,
        index_name: String,
        dimensions: u32,
    ) -> Self {
        let base_url = format!("https://{}-{}.svc.pinecone.io", index_name, environment);
        
        Self {
            api_key,
            environment,
            index_name,
            dimensions,
            client: reqwest::Client::new(),
            base_url,
        }
    }

    /// Initialize the vector database connection
    pub async fn initialize(&self) -> Result<()> {
        // Check if index exists and is ready
        let stats = self.get_index_stats().await?;
        tracing::info!(
            "Vector database initialized: {} vectors, {} dimensions",
            stats.total_vector_count,
            stats.dimension
        );
        
        Ok(())
    }

    /// Get index statistics
    pub async fn get_index_stats(&self) -> Result<IndexStats> {
        let url = format!("{}/describe_index_stats", self.base_url);
        
        let response = self.client
            .post(&url)
            .header("Api-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Failed to get index stats: {}", error_text));
        }

        let stats: IndexStats = response.json().await?;
        Ok(stats)
    }

    /// Upsert vectors to the database
    pub async fn upsert_vectors(
        &self,
        vectors: Vec<(String, Vec<f32>, HashMap<String, serde_json::Value>)>,
        namespace: Option<String>,
    ) -> Result<()> {
        let url = format!("{}/vectors/upsert", self.base_url);
        
        let vector_data: Vec<VectorData> = vectors
            .into_iter()
            .map(|(id, values, metadata)| VectorData {
                id,
                values,
                metadata,
            })
            .collect();

        let request = VectorUpsertRequest {
            vectors: vector_data,
            namespace,
        };

        let response = self.client
            .post(&url)
            .header("Api-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Failed to upsert vectors: {}", error_text));
        }

        tracing::info!("Successfully upserted {} vectors", request.vectors.len());
        Ok(())
    }

    /// Query similar vectors
    pub async fn query_similar(
        &self,
        query_vector: Vec<f32>,
        top_k: u32,
        namespace: Option<String>,
        filter: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<VectorSearchResult>> {
        let url = format!("{}/query", self.base_url);
        
        let request = VectorQueryRequest {
            vector: query_vector,
            top_k,
            include_metadata: true,
            namespace,
            filter,
        };

        let response = self.client
            .post(&url)
            .header("Api-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Failed to query vectors: {}", error_text));
        }

        let query_response: VectorQueryResponse = response.json().await?;
        
        let results: Vec<VectorSearchResult> = query_response
            .matches
            .into_iter()
            .map(|match_| VectorSearchResult {
                id: match_.id,
                score: match_.score,
                metadata: match_.metadata.unwrap_or_default(),
            })
            .collect();

        tracing::debug!("Found {} similar vectors", results.len());
        Ok(results)
    }

    /// Delete vectors by IDs
    pub async fn delete_vectors(
        &self,
        ids: Vec<String>,
        namespace: Option<String>,
    ) -> Result<()> {
        let url = format!("{}/vectors/delete", self.base_url);
        
        let mut request_body = serde_json::json!({
            "ids": ids
        });
        
        if let Some(ns) = namespace {
            request_body["namespace"] = serde_json::Value::String(ns);
        }

        let response = self.client
            .post(&url)
            .header("Api-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Failed to delete vectors: {}", error_text));
        }

        tracing::info!("Successfully deleted {} vectors", ids.len());
        Ok(())
    }

    /// Add audio fingerprint to vector database
    pub async fn add_fingerprint(
        &self,
        song_id: Uuid,
        fingerprint: &crate::fingerprint::Fingerprint,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        // Convert fingerprint to vector representation
        let vector = self.fingerprint_to_vector(fingerprint)?;
        
        let mut full_metadata = metadata;
        full_metadata.insert("song_id".to_string(), serde_json::Value::String(song_id.to_string()));
        full_metadata.insert("fingerprint_id".to_string(), serde_json::Value::String(Uuid::new_v4().to_string()));
        full_metadata.insert("created_at".to_string(), serde_json::Value::String(chrono::Utc::now().to_rfc3339()));

        let vector_id = format!("fingerprint_{}", song_id);
        
        self.upsert_vectors(
            vec![(vector_id, vector, full_metadata)],
            None,
        ).await?;

        Ok(())
    }

    /// Search for similar fingerprints
    pub async fn search_similar_fingerprints(
        &self,
        query_fingerprint: &crate::fingerprint::Fingerprint,
        top_k: u32,
        language_filter: Option<String>,
        genre_filter: Option<String>,
    ) -> Result<Vec<VectorSearchResult>> {
        // Convert fingerprint to vector
        let query_vector = self.fingerprint_to_vector(query_fingerprint)?;
        
        // Build filter if needed
        let mut filter = HashMap::new();
        if let Some(lang) = language_filter {
            filter.insert("language".to_string(), serde_json::Value::String(lang));
        }
        if let Some(genre) = genre_filter {
            filter.insert("genre".to_string(), serde_json::Value::String(genre));
        }
        
        let filter_option = if filter.is_empty() { None } else { Some(filter) };

        let results = self.query_similar(
            query_vector,
            top_k,
            None,
            filter_option,
        ).await?;

        Ok(results)
    }

    /// Convert fingerprint to vector representation
    fn fingerprint_to_vector(&self, fingerprint: &crate::fingerprint::Fingerprint) -> Result<Vec<f32>> {
        // Use spectral features as vector representation
        let mut vector = Vec::new();
        
        // Add spectral features
        if !fingerprint.peaks.is_empty() {
            // Frequency distribution (20 bins)
            let freq_bins = 20;
            let mut freq_histogram = vec![0.0; freq_bins];
            
            for peak in &fingerprint.peaks {
                let bin = ((peak.frequency / 20000.0) * freq_bins as f32) as usize;
                if bin < freq_bins {
                    freq_histogram[bin] += peak.magnitude;
                }
            }
            
            // Normalize frequency histogram
            let max_freq = freq_histogram.iter().fold(0.0, |a, &b| a.max(b));
            if max_freq > 0.0 {
                for val in &mut freq_histogram {
                    *val /= max_freq;
                }
            }
            
            vector.extend(freq_histogram);
        }
        
        // Add time distribution (10 bins)
        let time_bins = 10;
        let mut time_histogram = vec![0.0; time_bins];
        
        for peak in &fingerprint.peaks {
            let bin = ((peak.time / fingerprint.metadata.duration) * time_bins as f32) as usize;
            if bin < time_bins {
                time_histogram[bin] += peak.magnitude;
            }
        }
        
        // Normalize time histogram
        let max_time = time_histogram.iter().fold(0.0, |a, &b| a.max(b));
        if max_time > 0.0 {
            for val in &mut time_histogram {
                *val /= max_time;
            }
        }
        
        vector.extend(time_histogram);
        
        // Add statistical features
        if !fingerprint.peaks.is_empty() {
            let magnitudes: Vec<f32> = fingerprint.peaks.iter().map(|p| p.magnitude).collect();
            let mean_magnitude = magnitudes.iter().sum::<f32>() / magnitudes.len() as f32;
            let max_magnitude = magnitudes.iter().fold(0.0, |a, &b| a.max(b));
            let min_magnitude = magnitudes.iter().fold(f32::INFINITY, |a, &b| a.min(b));
            
            vector.push(mean_magnitude);
            vector.push(max_magnitude);
            vector.push(min_magnitude);
        }
        
        // Pad or truncate to target dimensions
        while vector.len() < self.dimensions as usize {
            vector.push(0.0);
        }
        
        if vector.len() > self.dimensions as usize {
            vector.truncate(self.dimensions as usize);
        }
        
        Ok(vector)
    }

    /// Batch upsert fingerprints
    pub async fn batch_upsert_fingerprints(
        &self,
        fingerprints: Vec<(Uuid, crate::fingerprint::Fingerprint, HashMap<String, serde_json::Value>)>,
    ) -> Result<()> {
        let batch_size = 100; // Pinecone batch limit
        
        for chunk in fingerprints.chunks(batch_size) {
            let vectors: Vec<(String, Vec<f32>, HashMap<String, serde_json::Value>)> = chunk
                .iter()
                .map(|(song_id, fingerprint, metadata)| {
                    let vector = self.fingerprint_to_vector(fingerprint).unwrap_or_default();
                    let mut full_metadata = metadata.clone();
                    full_metadata.insert("song_id".to_string(), serde_json::Value::String(song_id.to_string()));
                    full_metadata.insert("fingerprint_id".to_string(), serde_json::Value::String(Uuid::new_v4().to_string()));
                    full_metadata.insert("created_at".to_string(), serde_json::Value::String(chrono::Utc::now().to_rfc3339()));
                    
                    (format!("fingerprint_{}", song_id), vector, full_metadata)
                })
                .collect();
            
            self.upsert_vectors(vectors, None).await?;
        }
        
        Ok(())
    }

    /// Get vector database health status
    pub async fn health_check(&self) -> Result<bool> {
        match self.get_index_stats().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fingerprint::{Fingerprint, SpectralPeak, FingerprintMetadata};

    fn create_test_fingerprint() -> Fingerprint {
        let mut peaks = Vec::new();
        for i in 0..100 {
            peaks.push(SpectralPeak {
                frequency: i as f32 * 100.0,
                time: i as f32 * 0.1,
                magnitude: 1.0,
            });
        }

        Fingerprint {
            hashes: vec![1, 2, 3, 4, 5],
            time_offsets: vec![0.0, 0.1, 0.2, 0.3, 0.4],
            peaks,
            metadata: FingerprintMetadata {
                sample_rate: 44100,
                duration: 10.0,
                num_bins: 2048,
                window_size: 4096,
                overlap: 0.5,
            },
        }
    }

    #[tokio::test]
    async fn test_fingerprint_to_vector() {
        let db = VectorDatabase::new(
            "test-key".to_string(),
            "test-env".to_string(),
            "test-index".to_string(),
            1024,
        );
        
        let fingerprint = create_test_fingerprint();
        let vector = db.fingerprint_to_vector(&fingerprint).unwrap();
        
        assert_eq!(vector.len(), 1024);
        assert!(vector.iter().all(|&x| x >= 0.0 && x <= 1.0));
    }

    #[test]
    fn test_vector_database_creation() {
        let db = VectorDatabase::new(
            "test-key".to_string(),
            "test-env".to_string(),
            "test-index".to_string(),
            1024,
        );
        
        assert_eq!(db.dimensions, 1024);
        assert_eq!(db.index_name, "test-index");
    }
}
