//! WebAssembly bindings for Sonica audio engine

use wasm_bindgen::prelude::*;
use web_sys::console;
use crate::audio::AudioFeatures;
use crate::fingerprint::Fingerprint;
use crate::similarity::calculate_similarity;

// Import the `console.log` function from the `console` object
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Define a macro to make logging easier
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

/// WebAssembly audio processor for client-side processing
#[wasm_bindgen]
pub struct WasmAudioProcessor {
    sample_rate: u32,
    window_size: usize,
    overlap: f32,
}

#[wasm_bindgen]
impl WasmAudioProcessor {
    /// Create a new WebAssembly audio processor
    #[wasm_bindgen(constructor)]
    pub fn new(sample_rate: u32, window_size: usize, overlap: f32) -> WasmAudioProcessor {
        console_log!("Initializing WasmAudioProcessor with sample_rate: {}, window_size: {}, overlap: {}", 
                    sample_rate, window_size, overlap);
        
        WasmAudioProcessor {
            sample_rate,
            window_size,
            overlap,
        }
    }
    
    /// Process audio data and return fingerprint
    #[wasm_bindgen]
    pub fn process_audio(&self, audio_data: &[f32]) -> Result<JsValue, JsValue> {
        console_log!("Processing audio data with {} samples", audio_data.len());
        
        // Validate input
        if audio_data.is_empty() {
            return Err(JsValue::from_str("Audio data is empty"));
        }
        
        if audio_data.len() < self.window_size {
            return Err(JsValue::from_str("Audio data too short for processing"));
        }
        
        // Generate fingerprint
        match Fingerprint::generate(audio_data) {
            Ok(fingerprint) => {
                console_log!("Generated fingerprint with {} hashes", fingerprint.hashes.len());
                
                // Convert to JavaScript object
                let result = serde_wasm_bindgen::to_value(&fingerprint)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;
                
                Ok(result)
            }
            Err(e) => {
                console_log!("Error generating fingerprint: {}", e);
                Err(JsValue::from_str(&format!("Fingerprint generation failed: {}", e)))
            }
        }
    }
    
    /// Extract audio features
    #[wasm_bindgen]
    pub fn extract_features(&self, audio_data: &[f32]) -> Result<JsValue, JsValue> {
        console_log!("Extracting features from {} samples", audio_data.len());
        
        match crate::audio::extract_features(audio_data, self.sample_rate) {
            Ok(features) => {
                console_log!("Extracted features: spectral_centroid={}, spectral_rolloff={}", 
                           features.spectral_centroid, features.spectral_rolloff);
                
                serde_wasm_bindgen::to_value(&features)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
            }
            Err(e) => {
                console_log!("Error extracting features: {}", e);
                Err(JsValue::from_str(&format!("Feature extraction failed: {}", e)))
            }
        }
    }
    
    /// Calculate similarity between two fingerprints
    #[wasm_bindgen]
    pub fn calculate_similarity(&self, fingerprint1: &JsValue, fingerprint2: &JsValue) -> Result<f32, JsValue> {
        console_log!("Calculating similarity between fingerprints");
        
        // Deserialize fingerprints
        let fp1: Fingerprint = serde_wasm_bindgen::from_value(fingerprint1.clone())
            .map_err(|e| JsValue::from_str(&format!("Deserialization error: {}", e)))?;
        
        let fp2: Fingerprint = serde_wasm_bindgen::from_value(fingerprint2.clone())
            .map_err(|e| JsValue::from_str(&format!("Deserialization error: {}", e)))?;
        
        // Calculate similarity
        let similarity = calculate_similarity(&fp1, &fp2);
        console_log!("Similarity calculated: {}", similarity);
        
        Ok(similarity)
    }
    
    /// Normalize audio data
    #[wasm_bindgen]
    pub fn normalize_audio(&self, audio_data: &[f32]) -> Vec<f32> {
        console_log!("Normalizing audio data");
        crate::audio::normalize_audio(audio_data)
    }
    
    /// Reduce noise in audio data
    #[wasm_bindgen]
    pub fn reduce_noise(&self, audio_data: &[f32]) -> Vec<f32> {
        console_log!("Reducing noise in audio data");
        crate::audio::reduce_noise(audio_data)
    }
    
    /// Get processor configuration
    #[wasm_bindgen]
    pub fn get_config(&self) -> JsValue {
        let config = serde_json::json!({
            "sample_rate": self.sample_rate,
            "window_size": self.window_size,
            "overlap": self.overlap
        });
        
        serde_wasm_bindgen::to_value(&config).unwrap_or(JsValue::NULL)
    }
}

/// WebAssembly audio visualizer for real-time visualization
#[wasm_bindgen]
pub struct WasmAudioVisualizer {
    fft_size: usize,
    sample_rate: u32,
}

#[wasm_bindgen]
impl WasmAudioVisualizer {
    /// Create a new audio visualizer
    #[wasm_bindgen(constructor)]
    pub fn new(fft_size: usize, sample_rate: u32) -> WasmAudioVisualizer {
        console_log!("Initializing WasmAudioVisualizer with fft_size: {}, sample_rate: {}", 
                    fft_size, sample_rate);
        
        WasmAudioVisualizer {
            fft_size,
            sample_rate,
        }
    }
    
    /// Compute frequency spectrum for visualization
    #[wasm_bindgen]
    pub fn compute_spectrum(&self, audio_data: &[f32]) -> Result<Vec<f32>, JsValue> {
        console_log!("Computing spectrum for {} samples", audio_data.len());
        
        if audio_data.len() < self.fft_size {
            return Err(JsValue::from_str("Audio data too short for FFT"));
        }
        
        // Take the first fft_size samples
        let window_data: Vec<f32> = audio_data[..self.fft_size].to_vec();
        
        // Apply window function
        let windowed = crate::audio::apply_window(&window_data, crate::audio::WindowType::Hamming);
        
        // Compute FFT (simplified version for WASM)
        let spectrum = self.compute_fft_simple(&windowed);
        
        console_log!("Computed spectrum with {} bins", spectrum.len());
        Ok(spectrum)
    }
    
    /// Get frequency bins for visualization
    #[wasm_bindgen]
    pub fn get_frequency_bins(&self) -> Vec<f32> {
        let mut bins = Vec::new();
        let bin_width = self.sample_rate as f32 / (2.0 * self.fft_size as f32);
        
        for i in 0..self.fft_size / 2 {
            bins.push(i as f32 * bin_width);
        }
        
        bins
    }
    
    /// Simple FFT implementation for WebAssembly
    fn compute_fft_simple(&self, data: &[f32]) -> Vec<f32> {
        // Simplified FFT for demonstration
        // In production, you would use a proper FFT library
        let mut spectrum = vec![0.0; self.fft_size / 2];
        
        for i in 0..self.fft_size / 2 {
            let mut real = 0.0;
            let mut imag = 0.0;
            
            for j in 0..data.len() {
                let angle = -2.0 * std::f32::consts::PI * i as f32 * j as f32 / data.len() as f32;
                real += data[j] * angle.cos();
                imag += data[j] * angle.sin();
            }
            
            spectrum[i] = (real * real + imag * imag).sqrt();
        }
        
        spectrum
    }
}

/// WebAssembly performance monitor
#[wasm_bindgen]
pub struct WasmPerformanceMonitor {
    start_time: f64,
}

#[wasm_bindgen]
impl WasmPerformanceMonitor {
    /// Create a new performance monitor
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmPerformanceMonitor {
        WasmPerformanceMonitor {
            start_time: js_sys::Date::now(),
        }
    }
    
    /// Start timing
    #[wasm_bindgen]
    pub fn start(&mut self) {
        self.start_time = js_sys::Date::now();
    }
    
    /// End timing and return elapsed time in milliseconds
    #[wasm_bindgen]
    pub fn end(&self) -> f64 {
        js_sys::Date::now() - self.start_time
    }
    
    /// Get current timestamp
    #[wasm_bindgen]
    pub fn now() -> f64 {
        js_sys::Date::now()
    }
}

/// Initialize WebAssembly module
#[wasm_bindgen(start)]
pub fn main() {
    console_log!("Sonica WebAssembly module initialized");
    
    // Set up panic hook for better error messages
    console_error_panic_hook::set_once();
}

/// Utility functions for JavaScript integration
#[wasm_bindgen]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[wasm_bindgen]
pub fn get_build_info() -> String {
    format!("Sonica WASM v{} built with Rust", env!("CARGO_PKG_VERSION"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_wasm_audio_processor() {
        let processor = WasmAudioProcessor::new(44100, 4096, 0.5);
        
        // Test with dummy audio data
        let audio_data = vec![0.1; 44100]; // 1 second of audio
        let result = processor.process_audio(&audio_data);
        
        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    fn test_wasm_audio_visualizer() {
        let visualizer = WasmAudioVisualizer::new(1024, 44100);
        
        // Test with dummy audio data
        let audio_data = vec![0.1; 1024];
        let spectrum = visualizer.compute_spectrum(&audio_data);
        
        assert!(spectrum.is_ok());
        assert_eq!(spectrum.unwrap().len(), 512);
    }

    #[wasm_bindgen_test]
    fn test_performance_monitor() {
        let mut monitor = WasmPerformanceMonitor::new();
        monitor.start();
        
        // Simulate some work
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let elapsed = monitor.end();
        assert!(elapsed > 0.0);
    }
}
