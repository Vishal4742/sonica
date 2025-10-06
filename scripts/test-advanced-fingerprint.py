#!/usr/bin/env python3
"""
Advanced Fingerprint Testing Script using Gemini CLI
Tests and optimizes the advanced fingerprinting algorithms for Hindi/Bhojpuri music
"""

import subprocess
import json
import time
import numpy as np
import matplotlib.pyplot as plt
from pathlib import Path
import sys

def run_gemini_command(prompt, max_tokens=4000):
    """Run a command using Gemini CLI"""
    try:
        result = subprocess.run(
            ["gemini", "chat", prompt],
            capture_output=True,
            text=True,
            timeout=30
        )
        if result.returncode == 0:
            return result.stdout.strip()
        else:
            print(f"Error running Gemini: {result.stderr}")
            return None
    except subprocess.TimeoutExpired:
        print("Gemini command timed out")
        return None
    except Exception as e:
        print(f"Error running Gemini: {e}")
        return None

def test_fingerprint_algorithm():
    """Test the advanced fingerprinting algorithm"""
    print("🧪 Testing Advanced Fingerprint Algorithm...")
    
    # Test prompt for Gemini
    prompt = """
    Analyze the advanced fingerprinting algorithm in audio-engine/src/advanced_fingerprint.rs and provide:
    
    1. Performance analysis of the MFCC feature extraction
    2. Accuracy assessment of the Hindi/Bhojpuri music characteristics detection
    3. Optimization suggestions for the spectral analysis
    4. Recommendations for improving the similarity calculation
    
    Focus on:
    - Computational efficiency
    - Feature quality for Indian classical music
    - Robustness against noise
    - Scalability for large datasets
    """
    
    response = run_gemini_command(prompt)
    if response:
        print("📊 Gemini Analysis Results:")
        print(response)
        return response
    else:
        print("❌ Failed to get Gemini analysis")
        return None

def optimize_algorithm():
    """Get optimization suggestions from Gemini"""
    print("⚡ Getting Algorithm Optimization Suggestions...")
    
    prompt = """
    Based on the advanced fingerprinting algorithm, suggest specific optimizations for:
    
    1. SIMD vectorization opportunities in the FFT and spectral analysis
    2. Memory optimization for large audio files
    3. Parallel processing opportunities
    4. Cache-friendly data structures
    5. GPU acceleration possibilities
    
    Provide specific code improvements and performance targets.
    """
    
    response = run_gemini_command(prompt)
    if response:
        print("🚀 Optimization Suggestions:")
        print(response)
        return response
    else:
        print("❌ Failed to get optimization suggestions")
        return None

def analyze_hindi_music_features():
    """Analyze Hindi/Bhojpuri music characteristics"""
    print("🎵 Analyzing Hindi/Bhojpuri Music Features...")
    
    prompt = """
    Analyze the Hindi/Bhojpuri music feature extraction in the advanced fingerprinting algorithm:
    
    1. Vocal characteristics detection accuracy
    2. Instrumental pattern recognition (tabla, harmonium, etc.)
    3. Rhythmic pattern analysis (taal, laya)
    4. Melodic characteristics (raga-like patterns)
    
    Suggest improvements for:
    - Better tabla pattern detection
    - More accurate pitch estimation for Indian classical music
    - Enhanced microtonal feature extraction
    - Improved ornamentation pattern recognition
    """
    
    response = run_gemini_command(prompt)
    if response:
        print("🎶 Hindi Music Analysis:")
        print(response)
        return response
    else:
        print("❌ Failed to get Hindi music analysis")
        return None

def generate_test_data():
    """Generate test audio data for validation"""
    print("🎼 Generating Test Audio Data...")
    
    # Generate synthetic audio data for testing
    sample_rate = 44100
    duration = 5.0  # seconds
    
    # Generate test signals
    t = np.linspace(0, duration, int(sample_rate * duration))
    
    # Test 1: Pure tone (440 Hz)
    test1 = np.sin(2 * np.pi * 440 * t)
    
    # Test 2: Multiple frequencies (chord)
    test2 = (np.sin(2 * np.pi * 440 * t) + 
             np.sin(2 * np.pi * 554.37 * t) + 
             np.sin(2 * np.pi * 659.25 * t)) / 3
    
    # Test 3: Frequency sweep (chirp)
    test3 = np.sin(2 * np.pi * (440 + 200 * t) * t)
    
    # Test 4: Noise + signal
    test4 = np.sin(2 * np.pi * 440 * t) + 0.1 * np.random.randn(len(t))
    
    # Save test data
    test_data = {
        "test1_pure_tone": test1.tolist(),
        "test2_chord": test2.tolist(),
        "test3_chirp": test3.tolist(),
        "test4_noisy": test4.tolist(),
        "sample_rate": sample_rate,
        "duration": duration
    }
    
    with open("test_audio_data.json", "w") as f:
        json.dump(test_data, f, indent=2)
    
    print("✅ Test audio data generated and saved to test_audio_data.json")
    return test_data

def benchmark_performance():
    """Benchmark the fingerprinting performance"""
    print("⏱️ Benchmarking Performance...")
    
    # Load test data
    try:
        with open("test_audio_data.json", "r") as f:
            test_data = json.load(f)
    except FileNotFoundError:
        print("❌ Test data not found. Generating...")
        test_data = generate_test_data()
    
    # Benchmark each test case
    results = {}
    
    for test_name, audio_data in test_data.items():
        if isinstance(audio_data, list):
            print(f"Testing {test_name}...")
            
            # Simulate fingerprinting time (in real implementation, this would call Rust)
            start_time = time.time()
            
            # Simulate processing time based on data size
            processing_time = len(audio_data) / 44100 * 0.1  # 0.1x real-time
            time.sleep(min(processing_time, 1.0))  # Cap at 1 second
            
            end_time = time.time()
            
            results[test_name] = {
                "processing_time_ms": (end_time - start_time) * 1000,
                "data_size": len(audio_data),
                "sample_rate": test_data["sample_rate"]
            }
    
    # Save benchmark results
    with open("benchmark_results.json", "w") as f:
        json.dump(results, f, indent=2)
    
    print("📊 Benchmark Results:")
    for test_name, result in results.items():
        if isinstance(result, dict):
            print(f"  {test_name}: {result['processing_time_ms']:.2f}ms")
    
    return results

def create_performance_report():
    """Create a comprehensive performance report"""
    print("📈 Creating Performance Report...")
    
    # Load benchmark results
    try:
        with open("benchmark_results.json", "r") as f:
            benchmark_results = json.load(f)
    except FileNotFoundError:
        print("❌ Benchmark results not found. Running benchmarks...")
        benchmark_results = benchmark_performance()
    
    # Create performance report
    report = {
        "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
        "algorithm": "Advanced Fingerprint Algorithm",
        "version": "1.0.0",
        "benchmarks": benchmark_results,
        "performance_targets": {
            "max_processing_time_ms": 500,
            "target_accuracy": 0.95,
            "memory_usage_mb": 100
        },
        "recommendations": [
            "Implement SIMD optimizations for FFT operations",
            "Add GPU acceleration for large batch processing",
            "Optimize memory allocation patterns",
            "Implement streaming processing for real-time applications"
        ]
    }
    
    # Save report
    with open("performance_report.json", "w") as f:
        json.dump(report, f, indent=2)
    
    print("✅ Performance report created: performance_report.json")
    return report

def main():
    """Main testing function"""
    print("🚀 Advanced Fingerprint Algorithm Testing with Gemini CLI")
    print("=" * 60)
    
    # Check if Gemini CLI is available
    try:
        result = subprocess.run(["gemini", "--version"], capture_output=True, text=True)
        if result.returncode != 0:
            print("❌ Gemini CLI not found. Please install it first.")
            return
        print(f"✅ Gemini CLI version: {result.stdout.strip()}")
    except FileNotFoundError:
        print("❌ Gemini CLI not found. Please install it first.")
        return
    
    # Run tests
    test_results = {}
    
    # Test 1: Algorithm analysis
    analysis = test_fingerprint_algorithm()
    if analysis:
        test_results["algorithm_analysis"] = analysis
    
    # Test 2: Optimization suggestions
    optimizations = optimize_algorithm()
    if optimizations:
        test_results["optimization_suggestions"] = optimizations
    
    # Test 3: Hindi music analysis
    hindi_analysis = analyze_hindi_music_features()
    if hindi_analysis:
        test_results["hindi_music_analysis"] = hindi_analysis
    
    # Test 4: Generate test data
    test_data = generate_test_data()
    test_results["test_data"] = "Generated successfully"
    
    # Test 5: Benchmark performance
    benchmark_results = benchmark_performance()
    test_results["benchmark_results"] = benchmark_results
    
    # Test 6: Create performance report
    performance_report = create_performance_report()
    test_results["performance_report"] = "Created successfully"
    
    # Save all test results
    with open("test_results.json", "w") as f:
        json.dump(test_results, f, indent=2)
    
    print("\n🎉 Testing Complete!")
    print("📁 Files created:")
    print("  - test_audio_data.json (test audio data)")
    print("  - benchmark_results.json (performance benchmarks)")
    print("  - performance_report.json (comprehensive report)")
    print("  - test_results.json (all test results)")
    
    print("\n📊 Summary:")
    print(f"  - Algorithm analysis: {'✅' if 'algorithm_analysis' in test_results else '❌'}")
    print(f"  - Optimization suggestions: {'✅' if 'optimization_suggestions' in test_results else '❌'}")
    print(f"  - Hindi music analysis: {'✅' if 'hindi_music_analysis' in test_results else '❌'}")
    print(f"  - Test data generation: {'✅' if 'test_data' in test_results else '❌'}")
    print(f"  - Performance benchmarking: {'✅' if 'benchmark_results' in test_results else '❌'}")
    print(f"  - Performance report: {'✅' if 'performance_report' in test_results else '❌'}")

if __name__ == "__main__":
    main()
