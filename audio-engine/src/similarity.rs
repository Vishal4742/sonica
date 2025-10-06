//! Similarity calculation algorithms for audio fingerprints

use crate::fingerprint::Fingerprint;
use ndarray::{Array1, Array2};
use std::collections::HashMap;

/// Calculate similarity between two fingerprints
pub fn calculate_similarity(fingerprint1: &Fingerprint, fingerprint2: &Fingerprint) -> f32 {
    // Use multiple similarity metrics and combine them
    let hash_similarity = calculate_hash_similarity(fingerprint1, fingerprint2);
    let peak_similarity = calculate_peak_similarity(fingerprint1, fingerprint2);
    let spectral_similarity = calculate_spectral_similarity(fingerprint1, fingerprint2);
    
    // Weighted combination of similarities
    let weights = (0.5, 0.3, 0.2); // hash, peak, spectral
    hash_similarity * weights.0 + peak_similarity * weights.1 + spectral_similarity * weights.2
}

/// Calculate hash-based similarity
fn calculate_hash_similarity(fingerprint1: &Fingerprint, fingerprint2: &Fingerprint) -> f32 {
    if fingerprint1.hashes.is_empty() || fingerprint2.hashes.is_empty() {
        return 0.0;
    }
    
    // Create hash sets for fast lookup
    let hash_set1: std::collections::HashSet<u64> = fingerprint1.hashes.iter().cloned().collect();
    let hash_set2: std::collections::HashSet<u64> = fingerprint2.hashes.iter().cloned().collect();
    
    // Calculate Jaccard similarity
    let intersection = hash_set1.intersection(&hash_set2).count();
    let union = hash_set1.union(&hash_set2).count();
    
    if union == 0 {
        0.0
    } else {
        intersection as f32 / union as f32
    }
}

/// Calculate peak-based similarity
fn calculate_peak_similarity(fingerprint1: &Fingerprint, fingerprint2: &Fingerprint) -> f32 {
    if fingerprint1.peaks.is_empty() || fingerprint2.peaks.is_empty() {
        return 0.0;
    }
    
    let mut matches = 0;
    let mut total_peaks = 0;
    
    // Find matching peaks within tolerance
    let freq_tolerance = 50.0; // Hz
    let time_tolerance = 0.1; // seconds
    
    for peak1 in &fingerprint1.peaks {
        total_peaks += 1;
        
        for peak2 in &fingerprint2.peaks {
            let freq_diff = (peak1.frequency - peak2.frequency).abs();
            let time_diff = (peak1.time - peak2.time).abs();
            
            if freq_diff <= freq_tolerance && time_diff <= time_tolerance {
                matches += 1;
                break;
            }
        }
    }
    
    if total_peaks == 0 {
        0.0
    } else {
        matches as f32 / total_peaks as f32
    }
}

/// Calculate spectral similarity
fn calculate_spectral_similarity(fingerprint1: &Fingerprint, fingerprint2: &Fingerprint) -> f32 {
    // Compare spectral characteristics
    let spectral_features1 = extract_spectral_features(fingerprint1);
    let spectral_features2 = extract_spectral_features(fingerprint2);
    
    // Calculate cosine similarity
    cosine_similarity(&spectral_features1, &spectral_features2)
}

/// Extract spectral features from fingerprint
fn extract_spectral_features(fingerprint: &Fingerprint) -> Vec<f32> {
    let mut features = Vec::new();
    
    // Frequency distribution
    let freq_bins = 20;
    let mut freq_histogram = vec![0.0; freq_bins];
    
    for peak in &fingerprint.peaks {
        let bin = ((peak.frequency / 20000.0) * freq_bins as f32) as usize;
        if bin < freq_bins {
            freq_histogram[bin] += peak.magnitude;
        }
    }
    
    features.extend(freq_histogram);
    
    // Time distribution
    let time_bins = 10;
    let mut time_histogram = vec![0.0; time_bins];
    
    for peak in &fingerprint.peaks {
        let bin = ((peak.time / fingerprint.metadata.duration) * time_bins as f32) as usize;
        if bin < time_bins {
            time_histogram[bin] += peak.magnitude;
        }
    }
    
    features.extend(time_histogram);
    
    // Statistical features
    if !fingerprint.peaks.is_empty() {
        let magnitudes: Vec<f32> = fingerprint.peaks.iter().map(|p| p.magnitude).collect();
        let mean_magnitude = magnitudes.iter().sum::<f32>() / magnitudes.len() as f32;
        let max_magnitude = magnitudes.iter().fold(0.0, |a, &b| a.max(b));
        let min_magnitude = magnitudes.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        
        features.push(mean_magnitude);
        features.push(max_magnitude);
        features.push(min_magnitude);
    }
    
    features
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(vec1: &[f32], vec2: &[f32]) -> f32 {
    if vec1.len() != vec2.len() || vec1.is_empty() {
        return 0.0;
    }
    
    let dot_product: f32 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
    let norm1: f32 = vec1.iter().map(|&x| x * x).sum::<f32>().sqrt();
    let norm2: f32 = vec2.iter().map(|&x| x * x).sum::<f32>().sqrt();
    
    if norm1 == 0.0 || norm2 == 0.0 {
        0.0
    } else {
        dot_product / (norm1 * norm2)
    }
}

/// Advanced similarity calculation with time alignment
pub fn calculate_aligned_similarity(fingerprint1: &Fingerprint, fingerprint2: &Fingerprint) -> f32 {
    // Find best time alignment between fingerprints
    let time_offsets = find_time_offsets(fingerprint1, fingerprint2);
    
    if time_offsets.is_empty() {
        return 0.0;
    }
    
    // Calculate similarity for each time offset and take the best
    let mut best_similarity = 0.0;
    
    for time_offset in time_offsets {
        let aligned_similarity = calculate_time_aligned_similarity(fingerprint1, fingerprint2, time_offset);
        best_similarity = best_similarity.max(aligned_similarity);
    }
    
    best_similarity
}

/// Find potential time offsets between fingerprints
fn find_time_offsets(fingerprint1: &Fingerprint, fingerprint2: &Fingerprint) -> Vec<f32> {
    let mut time_offsets = Vec::new();
    let time_tolerance = 0.1; // seconds
    
    // Find matching hash pairs and their time differences
    let mut time_diffs = Vec::new();
    
    for (i, &hash1) in fingerprint1.hashes.iter().enumerate() {
        if let Some(j) = fingerprint2.hashes.iter().position(|&hash2| hash1 == hash2) {
            let time_diff = fingerprint1.time_offsets[i] - fingerprint2.time_offsets[j];
            time_diffs.push(time_diff);
        }
    }
    
    if time_diffs.is_empty() {
        return time_offsets;
    }
    
    // Group similar time differences
    time_diffs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    let mut current_offset = time_diffs[0];
    let mut count = 1;
    
    for &time_diff in &time_diffs[1..] {
        if (time_diff - current_offset).abs() <= time_tolerance {
            count += 1;
        } else {
            // If we have enough matches for this offset, add it
            if count >= 3 {
                time_offsets.push(current_offset);
            }
            current_offset = time_diff;
            count = 1;
        }
    }
    
    // Add the last offset if it has enough matches
    if count >= 3 {
        time_offsets.push(current_offset);
    }
    
    time_offsets
}

/// Calculate similarity with time alignment
fn calculate_time_aligned_similarity(
    fingerprint1: &Fingerprint,
    fingerprint2: &Fingerprint,
    time_offset: f32,
) -> f32 {
    let mut matches = 0;
    let mut total_hashes = 0;
    
    for (i, &hash1) in fingerprint1.hashes.iter().enumerate() {
        total_hashes += 1;
        let time1 = fingerprint1.time_offsets[i];
        
        for (j, &hash2) in fingerprint2.hashes.iter().enumerate() {
            if hash1 == hash2 {
                let time2 = fingerprint2.time_offsets[j] + time_offset;
                let time_diff = (time1 - time2).abs();
                
                if time_diff <= 0.1 {
                    matches += 1;
                    break;
                }
            }
        }
    }
    
    if total_hashes == 0 {
        0.0
    } else {
        matches as f32 / total_hashes as f32
    }
}

/// Batch similarity calculation for multiple fingerprints
pub fn calculate_batch_similarity(
    query_fingerprint: &Fingerprint,
    candidate_fingerprints: &[Fingerprint],
) -> Vec<(usize, f32)> {
    candidate_fingerprints
        .iter()
        .enumerate()
        .map(|(i, candidate)| {
            let similarity = calculate_similarity(query_fingerprint, candidate);
            (i, similarity)
        })
        .filter(|(_, similarity)| *similarity > 0.1) // Filter out very low similarities
        .collect()
}

/// Fast similarity calculation using approximate methods
pub fn calculate_fast_similarity(fingerprint1: &Fingerprint, fingerprint2: &Fingerprint) -> f32 {
    // Use only hash similarity for speed
    calculate_hash_similarity(fingerprint1, fingerprint2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fingerprint::{Fingerprint, SpectralPeak, FingerprintMetadata};

    fn create_test_fingerprint(hashes: Vec<u64>, peaks: Vec<SpectralPeak>) -> Fingerprint {
        Fingerprint {
            hashes,
            time_offsets: vec![0.0, 0.1, 0.2, 0.3, 0.4],
            peaks,
            metadata: FingerprintMetadata {
                sample_rate: 44100,
                duration: 1.0,
                num_bins: 2048,
                window_size: 4096,
                overlap: 0.5,
            },
        }
    }

    #[test]
    fn test_hash_similarity() {
        let fingerprint1 = create_test_fingerprint(vec![1, 2, 3, 4, 5], Vec::new());
        let fingerprint2 = create_test_fingerprint(vec![1, 2, 3, 6, 7], Vec::new());
        
        let similarity = calculate_hash_similarity(&fingerprint1, &fingerprint2);
        assert!(similarity > 0.0);
        assert!(similarity <= 1.0);
    }

    #[test]
    fn test_cosine_similarity() {
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![1.0, 2.0, 3.0];
        let vec3 = vec![0.0, 0.0, 0.0];
        
        assert_eq!(cosine_similarity(&vec1, &vec2), 1.0);
        assert_eq!(cosine_similarity(&vec1, &vec3), 0.0);
    }

    #[test]
    fn test_batch_similarity() {
        let query = create_test_fingerprint(vec![1, 2, 3], Vec::new());
        let candidates = vec![
            create_test_fingerprint(vec![1, 2, 3], Vec::new()),
            create_test_fingerprint(vec![4, 5, 6], Vec::new()),
            create_test_fingerprint(vec![1, 2, 7], Vec::new()),
        ];
        
        let results = calculate_batch_similarity(&query, &candidates);
        assert!(!results.is_empty());
        assert_eq!(results[0].0, 0); // First candidate should have highest similarity
    }
}
