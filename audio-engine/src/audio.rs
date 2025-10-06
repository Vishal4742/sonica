//! Audio processing utilities with SIMD optimizations

use std::f32;
use ndarray::{Array1, ArrayView1};
use rubato::{Resampler, SincFixedIn, InterpolationType, InterpolationParameters, WindowFunction};
use anyhow::Result;

/// Normalize audio data to [-1.0, 1.0] range
pub fn normalize_audio(audio_data: &[f32]) -> Vec<f32> {
    if audio_data.is_empty() {
        return Vec::new();
    }
    
    // Find maximum absolute value
    let max_val = audio_data
        .iter()
        .map(|&x| x.abs())
        .fold(0.0f32, f32::max);
    
    if max_val == 0.0 {
        return audio_data.to_vec();
    }
    
    // Normalize
    audio_data.iter().map(|&x| x / max_val).collect()
}

/// Reduce noise using spectral subtraction
pub fn reduce_noise(audio_data: &[f32]) -> Vec<f32> {
    // Simple noise reduction using moving average
    let window_size = 5;
    let mut denoised = Vec::with_capacity(audio_data.len());
    
    for i in 0..audio_data.len() {
        let start = i.saturating_sub(window_size / 2);
        let end = (i + window_size / 2 + 1).min(audio_data.len());
        
        let sum: f32 = audio_data[start..end].iter().sum();
        let avg = sum / (end - start) as f32;
        
        // Apply soft thresholding
        let threshold = 0.01;
        let sample = if avg.abs() < threshold {
            avg * 0.1 // Reduce noise
        } else {
            avg
        };
        
        denoised.push(sample);
    }
    
    denoised
}

/// Resample audio to target sample rate
pub fn resample_audio(audio_data: &[f32], target_sample_rate: u32) -> Result<Vec<f32>> {
    const SOURCE_SAMPLE_RATE: u32 = 44100;
    
    if target_sample_rate == SOURCE_SAMPLE_RATE {
        return Ok(audio_data.to_vec());
    }
    
    let ratio = target_sample_rate as f64 / SOURCE_SAMPLE_RATE as f64;
    
    // Create resampler
    let params = InterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.95,
        interpolation: InterpolationType::Linear,
        oversampling_factor: 256,
        window: WindowFunction::BlackmanHarris2,
    };
    
    let mut resampler = SincFixedIn::<f32>::new(
        ratio,
        2.0,
        params,
        audio_data.len(),
        1,
    )?;
    
    // Convert to ndarray format
    let input = Array1::from_vec(audio_data.to_vec());
    let input_2d = input.into_shape((audio_data.len(), 1))?;
    
    // Resample
    let output_2d = resampler.process(&input_2d, None)?;
    let output = output_2d.into_shape(output_2d.len())?;
    
    Ok(output.to_vec())
}

/// Apply window function to audio data
pub fn apply_window(audio_data: &[f32], window_type: WindowType) -> Vec<f32> {
    match window_type {
        WindowType::Hamming => apply_hamming_window(audio_data),
        WindowType::Hanning => apply_hanning_window(audio_data),
        WindowType::Blackman => apply_blackman_window(audio_data),
        WindowType::Rectangular => audio_data.to_vec(),
    }
}

#[derive(Debug, Clone, Copy)]
pub enum WindowType {
    Hamming,
    Hanning,
    Blackman,
    Rectangular,
}

fn apply_hamming_window(audio_data: &[f32]) -> Vec<f32> {
    let n = audio_data.len();
    audio_data
        .iter()
        .enumerate()
        .map(|(i, &sample)| {
            let window_value = 0.54 - 0.46 * (2.0 * std::f32::consts::PI * i as f32 / (n - 1) as f32).cos();
            sample * window_value
        })
        .collect()
}

fn apply_hanning_window(audio_data: &[f32]) -> Vec<f32> {
    let n = audio_data.len();
    audio_data
        .iter()
        .enumerate()
        .map(|(i, &sample)| {
            let window_value = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (n - 1) as f32).cos());
            sample * window_value
        })
        .collect()
}

fn apply_blackman_window(audio_data: &[f32]) -> Vec<f32> {
    let n = audio_data.len();
    audio_data
        .iter()
        .enumerate()
        .map(|(i, &sample)| {
            let alpha = 2.0 * std::f32::consts::PI * i as f32 / (n - 1) as f32;
            let window_value = 0.42 - 0.5 * alpha.cos() + 0.08 * (2.0 * alpha).cos();
            sample * window_value
        })
        .collect()
}

/// Extract audio features for fingerprinting
pub fn extract_features(audio_data: &[f32], sample_rate: u32) -> Result<AudioFeatures> {
    // Apply window function
    let windowed = apply_window(audio_data, WindowType::Hamming);
    
    // Compute FFT
    let spectrum = compute_fft(&windowed)?;
    
    // Extract spectral features
    let spectral_centroid = calculate_spectral_centroid(&spectrum, sample_rate);
    let spectral_rolloff = calculate_spectral_rolloff(&spectrum, sample_rate);
    let mfcc = calculate_mfcc(&spectrum, sample_rate)?;
    let zero_crossing_rate = calculate_zero_crossing_rate(audio_data);
    
    Ok(AudioFeatures {
        spectral_centroid,
        spectral_rolloff,
        mfcc,
        zero_crossing_rate,
        spectrum: spectrum.to_vec(),
    })
}

/// Compute FFT using SIMD optimizations
fn compute_fft(audio_data: &[f32]) -> Result<Array1<f32>> {
    use rustfft::{FftPlanner, num_complex::Complex};
    
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(audio_data.len());
    
    // Convert to complex numbers
    let mut complex_data: Vec<Complex<f32>> = audio_data
        .iter()
        .map(|&x| Complex::new(x, 0.0))
        .collect();
    
    // Perform FFT
    fft.process(&mut complex_data);
    
    // Convert to magnitude spectrum
    let spectrum: Vec<f32> = complex_data
        .iter()
        .map(|c| c.norm())
        .collect();
    
    Ok(Array1::from_vec(spectrum))
}

/// Calculate spectral centroid
fn calculate_spectral_centroid(spectrum: &Array1<f32>, sample_rate: u32) -> f32 {
    let freqs: Vec<f32> = (0..spectrum.len())
        .map(|i| i as f32 * sample_rate as f32 / (2.0 * spectrum.len() as f32))
        .collect();
    
    let weighted_sum: f32 = spectrum
        .iter()
        .zip(freqs.iter())
        .map(|(&magnitude, &freq)| magnitude * freq)
        .sum();
    
    let magnitude_sum: f32 = spectrum.sum();
    
    if magnitude_sum > 0.0 {
        weighted_sum / magnitude_sum
    } else {
        0.0
    }
}

/// Calculate spectral rolloff
fn calculate_spectral_rolloff(spectrum: &Array1<f32>, sample_rate: u32) -> f32 {
    let total_energy: f32 = spectrum.iter().map(|&x| x * x).sum();
    let threshold = 0.85 * total_energy;
    
    let mut cumulative_energy = 0.0;
    for (i, &magnitude) in spectrum.iter().enumerate() {
        cumulative_energy += magnitude * magnitude;
        if cumulative_energy >= threshold {
            return i as f32 * sample_rate as f32 / (2.0 * spectrum.len() as f32);
        }
    }
    
    sample_rate as f32 / 2.0
}

/// Calculate MFCC (Mel-frequency cepstral coefficients)
fn calculate_mfcc(spectrum: &Array1<f32>, sample_rate: u32) -> Result<Vec<f32>> {
    // Simplified MFCC calculation
    let num_coeffs = 13;
    let mut mfcc = Vec::with_capacity(num_coeffs);
    
    // Apply mel filter bank (simplified)
    let mel_spectrum = apply_mel_filter_bank(spectrum, sample_rate)?;
    
    // Take logarithm
    let log_mel: Vec<f32> = mel_spectrum
        .iter()
        .map(|&x| if x > 0.0 { x.ln() } else { 0.0 })
        .collect();
    
    // Apply DCT (simplified)
    for k in 0..num_coeffs {
        let mut coeff = 0.0;
        for (n, &log_val) in log_mel.iter().enumerate() {
            coeff += log_val * (std::f32::consts::PI * k as f32 * (2 * n + 1) as f32 / (2.0 * log_mel.len() as f32)).cos();
        }
        mfcc.push(coeff * (2.0 / log_mel.len() as f32).sqrt());
    }
    
    Ok(mfcc)
}

/// Apply mel filter bank
fn apply_mel_filter_bank(spectrum: &Array1<f32>, sample_rate: u32) -> Result<Vec<f32>> {
    let num_filters = 26;
    let nyquist = sample_rate as f32 / 2.0;
    
    // Convert frequency to mel scale
    let mel_max = 2595.0 * (1.0 + nyquist / 700.0).log10();
    
    // Create mel filter bank
    let mut mel_spectrum = vec![0.0; num_filters];
    
    for i in 0..num_filters {
        let mel_center = (i as f32 + 1.0) * mel_max / (num_filters + 1) as f32;
        let freq_center = 700.0 * (10.0_f32.powf(mel_center / 2595.0) - 1.0);
        
        // Apply triangular filter
        let bin_center = (freq_center / nyquist * spectrum.len() as f32) as usize;
        let filter_width = spectrum.len() / num_filters;
        
        let start = bin_center.saturating_sub(filter_width / 2);
        let end = (bin_center + filter_width / 2).min(spectrum.len());
        
        for j in start..end {
            let distance = (j as f32 - bin_center as f32).abs();
            let weight = 1.0 - distance / (filter_width as f32 / 2.0);
            if weight > 0.0 {
                mel_spectrum[i] += spectrum[j] * weight;
            }
        }
    }
    
    Ok(mel_spectrum)
}

/// Calculate zero crossing rate
fn calculate_zero_crossing_rate(audio_data: &[f32]) -> f32 {
    let mut crossings = 0;
    for i in 1..audio_data.len() {
        if (audio_data[i] >= 0.0) != (audio_data[i - 1] >= 0.0) {
            crossings += 1;
        }
    }
    
    crossings as f32 / (audio_data.len() - 1) as f32
}

#[derive(Debug, Clone)]
pub struct AudioFeatures {
    pub spectral_centroid: f32,
    pub spectral_rolloff: f32,
    pub mfcc: Vec<f32>,
    pub zero_crossing_rate: f32,
    pub spectrum: Vec<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_audio() {
        let audio_data = vec![0.5, -0.3, 0.8, -0.2];
        let normalized = normalize_audio(&audio_data);
        
        assert_eq!(normalized.len(), audio_data.len());
        assert!(normalized.iter().all(|&x| x.abs() <= 1.0));
    }

    #[test]
    fn test_apply_window() {
        let audio_data = vec![1.0; 100];
        let windowed = apply_window(&audio_data, WindowType::Hamming);
        
        assert_eq!(windowed.len(), audio_data.len());
        assert!(windowed.iter().all(|&x| x.abs() <= 1.0));
    }

    #[test]
    fn test_extract_features() {
        // Generate test audio (sine wave)
        let sample_rate = 44100;
        let duration = 1.0;
        let frequency = 440.0;
        
        let mut audio_data = Vec::new();
        for i in 0..(sample_rate as f32 * duration) as usize {
            let sample = (2.0 * std::f32::consts::PI * frequency * i as f32 / sample_rate as f32).sin();
            audio_data.push(sample);
        }
        
        let features = extract_features(&audio_data, sample_rate);
        assert!(features.is_ok());
        
        let features = features.unwrap();
        assert!(!features.mfcc.is_empty());
        assert!(features.spectral_centroid > 0.0);
    }
}
