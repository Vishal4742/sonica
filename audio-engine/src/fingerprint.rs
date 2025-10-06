//! Audio fingerprinting algorithms for music recognition

use std::collections::HashMap;
use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};
use anyhow::Result;

/// Audio fingerprint for music recognition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fingerprint {
    /// Hash values representing audio characteristics
    pub hashes: Vec<u64>,
    /// Time offsets for each hash
    pub time_offsets: Vec<f32>,
    /// Spectral peaks used for fingerprinting
    pub peaks: Vec<SpectralPeak>,
    /// Metadata about the fingerprint
    pub metadata: FingerprintMetadata,
}

/// Spectral peak in frequency domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectralPeak {
    /// Frequency in Hz
    pub frequency: f32,
    /// Time in seconds
    pub time: f32,
    /// Magnitude of the peak
    pub magnitude: f32,
}

/// Fingerprint metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintMetadata {
    /// Sample rate of the original audio
    pub sample_rate: u32,
    /// Duration of the audio in seconds
    pub duration: f32,
    /// Number of frequency bins
    pub num_bins: usize,
    /// Window size used for analysis
    pub window_size: usize,
    /// Overlap between windows
    pub overlap: f32,
}

impl Fingerprint {
    /// Generate fingerprint from audio data
    pub fn generate(audio_data: &[f32]) -> Result<Self> {
        let sample_rate = 44100;
        let window_size = 4096;
        let overlap = 0.5;
        let hop_size = (window_size as f32 * (1.0 - overlap)) as usize;
        
        // Compute spectrogram
        let spectrogram = compute_spectrogram(audio_data, window_size, hop_size, sample_rate)?;
        
        // Find spectral peaks
        let peaks = find_spectral_peaks(&spectrogram, sample_rate, hop_size)?;
        
        // Generate hash pairs
        let (hashes, time_offsets) = generate_hash_pairs(&peaks)?;
        
        let duration = audio_data.len() as f32 / sample_rate as f32;
        let num_bins = spectrogram.nrows();
        
        let metadata = FingerprintMetadata {
            sample_rate,
            duration,
            num_bins,
            window_size,
            overlap,
        };
        
        Ok(Fingerprint {
            hashes,
            time_offsets,
            peaks,
            metadata,
        })
    }
    
    /// Calculate similarity with another fingerprint
    pub fn similarity(&self, other: &Fingerprint) -> f32 {
        if self.hashes.is_empty() || other.hashes.is_empty() {
            return 0.0;
        }
        
        // Find matching hashes
        let mut matches = 0;
        let mut time_diffs = Vec::new();
        
        for (i, &hash1) in self.hashes.iter().enumerate() {
            if let Some(j) = other.hashes.iter().position(|&hash2| hash1 == hash2) {
                matches += 1;
                let time_diff = (self.time_offsets[i] - other.time_offsets[j]).abs();
                time_diffs.push(time_diff);
            }
        }
        
        if matches == 0 {
            return 0.0;
        }
        
        // Calculate similarity based on matches and time consistency
        let hash_similarity = matches as f32 / self.hashes.len().max(other.hashes.len()) as f32;
        
        // Time consistency bonus
        let time_consistency = if !time_diffs.is_empty() {
            let avg_time_diff = time_diffs.iter().sum::<f32>() / time_diffs.len() as f32;
            if avg_time_diff < 0.1 {
                1.0
            } else if avg_time_diff < 0.5 {
                0.8
            } else {
                0.5
            }
        } else {
            0.0
        };
        
        hash_similarity * time_consistency
    }
    
    /// Get fingerprint as bytes for storage
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(self)?)
    }
    
    /// Create fingerprint from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        Ok(bincode::deserialize(data)?)
    }
}

/// Compute spectrogram from audio data
fn compute_spectrogram(
    audio_data: &[f32],
    window_size: usize,
    hop_size: usize,
    sample_rate: u32,
) -> Result<Array2<f32>> {
    use rustfft::{FftPlanner, num_complex::Complex};
    
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(window_size);
    
    let num_frames = (audio_data.len() - window_size) / hop_size + 1;
    let num_bins = window_size / 2 + 1;
    
    let mut spectrogram = Array2::zeros((num_bins, num_frames));
    
    for (frame_idx, frame_start) in (0..audio_data.len() - window_size + 1)
        .step_by(hop_size)
        .enumerate()
    {
        // Extract window
        let window_data: Vec<f32> = audio_data[frame_start..frame_start + window_size]
            .iter()
            .enumerate()
            .map(|(i, &sample)| {
                // Apply Hamming window
                let window_value = 0.54 - 0.46 * (2.0 * std::f32::consts::PI * i as f32 / (window_size - 1) as f32).cos();
                sample * window_value
            })
            .collect();
        
        // Convert to complex numbers
        let mut complex_data: Vec<Complex<f32>> = window_data
            .iter()
            .map(|&x| Complex::new(x, 0.0))
            .collect();
        
        // Perform FFT
        fft.process(&mut complex_data);
        
        // Store magnitude spectrum
        for (bin_idx, &complex_val) in complex_data.iter().take(num_bins).enumerate() {
            spectrogram[[bin_idx, frame_idx]] = complex_val.norm();
        }
    }
    
    Ok(spectrogram)
}

/// Find spectral peaks in spectrogram
fn find_spectral_peaks(
    spectrogram: &Array2<f32>,
    sample_rate: u32,
    hop_size: usize,
) -> Result<Vec<SpectralPeak>> {
    let mut peaks = Vec::new();
    let num_bins = spectrogram.nrows();
    let num_frames = spectrogram.ncols();
    
    // Convert bin indices to frequencies
    let bin_to_freq = |bin: usize| -> f32 {
        bin as f32 * sample_rate as f32 / (2.0 * (num_bins - 1) as f32)
    };
    
    // Convert frame indices to time
    let frame_to_time = |frame: usize| -> f32 {
        frame as f32 * hop_size as f32 / sample_rate as f32
    };
    
    // Find peaks in each frame
    for frame_idx in 0..num_frames {
        let frame_spectrum = spectrogram.column(frame_idx);
        
        // Find local maxima
        for bin_idx in 1..num_bins - 1 {
            let magnitude = frame_spectrum[bin_idx];
            
            // Check if it's a local maximum
            if magnitude > frame_spectrum[bin_idx - 1] && magnitude > frame_spectrum[bin_idx + 1] {
                // Check if magnitude is above threshold
                let threshold = calculate_adaptive_threshold(&frame_spectrum, bin_idx);
                
                if magnitude > threshold {
                    peaks.push(SpectralPeak {
                        frequency: bin_to_freq(bin_idx),
                        time: frame_to_time(frame_idx),
                        magnitude,
                    });
                }
            }
        }
    }
    
    // Sort peaks by magnitude (strongest first)
    peaks.sort_by(|a, b| b.magnitude.partial_cmp(&a.magnitude).unwrap());
    
    // Keep only top peaks to avoid too many hash pairs
    peaks.truncate(1000);
    
    Ok(peaks)
}

/// Calculate adaptive threshold for peak detection
fn calculate_adaptive_threshold(spectrum: &Array1<f32>, bin_idx: usize) -> f32 {
    let window_size = 10;
    let start = bin_idx.saturating_sub(window_size / 2);
    let end = (bin_idx + window_size / 2 + 1).min(spectrum.len());
    
    let local_spectrum = &spectrum.slice(ndarray::s![start..end]);
    let mean = local_spectrum.mean().unwrap_or(0.0);
    let std = local_spectrum.std(1.0);
    
    mean + 2.0 * std
}

/// Generate hash pairs from spectral peaks
fn generate_hash_pairs(peaks: &[SpectralPeak]) -> Result<(Vec<u64>, Vec<f32>)> {
    let mut hashes = Vec::new();
    let mut time_offsets = Vec::new();
    
    // Generate hash pairs using time-frequency combinations
    for i in 0..peaks.len() {
        for j in (i + 1)..peaks.len() {
            let peak1 = &peaks[i];
            let peak2 = &peaks[j];
            
            // Skip if peaks are too far apart in time
            let time_diff = (peak2.time - peak1.time).abs();
            if time_diff > 10.0 {
                break;
            }
            
            // Create hash from frequency and time differences
            let freq_diff = peak2.frequency - peak1.frequency;
            let hash = create_hash(peak1.frequency, freq_diff, time_diff);
            
            hashes.push(hash);
            time_offsets.push(peak1.time);
        }
    }
    
    Ok((hashes, time_offsets))
}

/// Create hash from frequency and time information
fn create_hash(freq1: f32, freq_diff: f32, time_diff: f32) -> u64 {
    // Quantize values to reduce noise sensitivity
    let freq1_quantized = (freq1 / 10.0).round() as i32;
    let freq_diff_quantized = (freq_diff / 10.0).round() as i32;
    let time_diff_quantized = (time_diff * 100.0).round() as i32;
    
    // Combine into hash
    let hash = ((freq1_quantized as u64) << 32) 
        | ((freq_diff_quantized as u64) << 16) 
        | (time_diff_quantized as u64);
    
    hash
}

/// Fast fingerprint matching using hash tables
pub struct FingerprintMatcher {
    hash_table: HashMap<u64, Vec<(usize, f32)>>, // hash -> [(song_id, time_offset)]
}

impl FingerprintMatcher {
    /// Create new fingerprint matcher
    pub fn new() -> Self {
        Self {
            hash_table: HashMap::new(),
        }
    }
    
    /// Add fingerprint to matcher
    pub fn add_fingerprint(&mut self, song_id: usize, fingerprint: &Fingerprint) {
        for (hash, &time_offset) in fingerprint.hashes.iter().zip(fingerprint.time_offsets.iter()) {
            self.hash_table
                .entry(*hash)
                .or_insert_with(Vec::new)
                .push((song_id, time_offset));
        }
    }
    
    /// Find best matching songs
    pub fn find_matches(&self, query_fingerprint: &Fingerprint, min_matches: usize) -> Vec<(usize, f32, usize)> {
        let mut song_matches: HashMap<usize, Vec<f32>> = HashMap::new();
        
        // Find matching hashes
        for (hash, &query_time) in query_fingerprint.hashes.iter().zip(query_fingerprint.time_offsets.iter()) {
            if let Some(matches) = self.hash_table.get(hash) {
                for &(song_id, song_time) in matches {
                    let time_diff = (query_time - song_time).abs();
                    song_matches
                        .entry(song_id)
                        .or_insert_with(Vec::new)
                        .push(time_diff);
                }
            }
        }
        
        // Calculate scores for each song
        let mut results = Vec::new();
        for (song_id, time_diffs) in song_matches {
            if time_diffs.len() >= min_matches {
                // Calculate score based on number of matches and time consistency
                let num_matches = time_diffs.len();
                let avg_time_diff = time_diffs.iter().sum::<f32>() / num_matches as f32;
                let time_consistency = if avg_time_diff < 0.1 { 1.0 } else { 0.5 };
                let score = num_matches as f32 * time_consistency;
                
                results.push((song_id, score, num_matches));
            }
        }
        
        // Sort by score (highest first)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_generation() {
        // Generate test audio (sine wave)
        let sample_rate = 44100;
        let duration = 1.0;
        let frequency = 440.0;
        
        let mut audio_data = Vec::new();
        for i in 0..(sample_rate as f32 * duration) as usize {
            let sample = (2.0 * std::f32::consts::PI * frequency * i as f32 / sample_rate as f32).sin();
            audio_data.push(sample);
        }
        
        let fingerprint = Fingerprint::generate(&audio_data);
        assert!(fingerprint.is_ok());
        
        let fingerprint = fingerprint.unwrap();
        assert!(!fingerprint.hashes.is_empty());
        assert!(!fingerprint.peaks.is_empty());
    }

    #[test]
    fn test_fingerprint_similarity() {
        // Create two similar fingerprints
        let mut fingerprint1 = Fingerprint {
            hashes: vec![1, 2, 3, 4, 5],
            time_offsets: vec![0.0, 0.1, 0.2, 0.3, 0.4],
            peaks: Vec::new(),
            metadata: FingerprintMetadata {
                sample_rate: 44100,
                duration: 1.0,
                num_bins: 2048,
                window_size: 4096,
                overlap: 0.5,
            },
        };
        
        let mut fingerprint2 = Fingerprint {
            hashes: vec![1, 2, 3, 6, 7], // 3 matches
            time_offsets: vec![0.0, 0.1, 0.2, 0.5, 0.6],
            peaks: Vec::new(),
            metadata: FingerprintMetadata {
                sample_rate: 44100,
                duration: 1.0,
                num_bins: 2048,
                window_size: 4096,
                overlap: 0.5,
            },
        };
        
        let similarity = fingerprint1.similarity(&fingerprint2);
        assert!(similarity > 0.0);
        assert!(similarity <= 1.0);
    }

    #[test]
    fn test_fingerprint_matcher() {
        let mut matcher = FingerprintMatcher::new();
        
        let fingerprint1 = Fingerprint {
            hashes: vec![1, 2, 3],
            time_offsets: vec![0.0, 0.1, 0.2],
            peaks: Vec::new(),
            metadata: FingerprintMetadata {
                sample_rate: 44100,
                duration: 1.0,
                num_bins: 2048,
                window_size: 4096,
                overlap: 0.5,
            },
        };
        
        let fingerprint2 = Fingerprint {
            hashes: vec![4, 5, 6],
            time_offsets: vec![0.0, 0.1, 0.2],
            peaks: Vec::new(),
            metadata: FingerprintMetadata {
                sample_rate: 44100,
                duration: 1.0,
                num_bins: 2048,
                window_size: 4096,
                overlap: 0.5,
            },
        };
        
        matcher.add_fingerprint(1, &fingerprint1);
        matcher.add_fingerprint(2, &fingerprint2);
        
        let query = Fingerprint {
            hashes: vec![1, 2, 7], // 2 matches with song 1
            time_offsets: vec![0.0, 0.1, 0.3],
            peaks: Vec::new(),
            metadata: FingerprintMetadata {
                sample_rate: 44100,
                duration: 1.0,
                num_bins: 2048,
                window_size: 4096,
                overlap: 0.5,
            },
        };
        
        let matches = matcher.find_matches(&query, 1);
        assert!(!matches.is_empty());
        assert_eq!(matches[0].0, 1); // Should match song 1
    }
}
