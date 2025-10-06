#!/bin/bash

# Build WebAssembly module for Sonica audio engine

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if wasm-pack is installed
check_wasm_pack() {
    print_status "Checking wasm-pack installation..."
    if ! command -v wasm-pack &> /dev/null; then
        print_warning "wasm-pack is not installed. Installing..."
        curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
    else
        print_success "wasm-pack is installed"
    fi
}

# Build WebAssembly module
build_wasm() {
    print_status "Building WebAssembly module..."
    
    cd audio-engine
    
    # Clean previous builds
    if [ -d "pkg" ]; then
        print_status "Cleaning previous build..."
        rm -rf pkg
    fi
    
    # Build with wasm-pack
    print_status "Building with wasm-pack..."
    wasm-pack build --target web --out-dir pkg --out-name sonica-audio-engine
    
    if [ $? -eq 0 ]; then
        print_success "WebAssembly module built successfully"
    else
        print_error "WebAssembly build failed"
        exit 1
    fi
    
    cd ..
}

# Copy WASM files to frontend
copy_wasm_files() {
    print_status "Copying WASM files to frontend..."
    
    # Create frontend public/wasm directory if it doesn't exist
    mkdir -p frontend/public/wasm
    
    # Copy WASM files
    cp audio-engine/pkg/sonica-audio-engine.wasm frontend/public/wasm/
    cp audio-engine/pkg/sonica-audio-engine.js frontend/public/wasm/
    cp audio-engine/pkg/sonica-audio-engine_bg.wasm.d.ts frontend/public/wasm/
    
    print_success "WASM files copied to frontend"
}

# Optimize WASM file size
optimize_wasm() {
    print_status "Optimizing WASM file size..."
    
    # Check if wasm-opt is available
    if command -v wasm-opt &> /dev/null; then
        print_status "Running wasm-opt optimization..."
        wasm-opt -Oz frontend/public/wasm/sonica-audio-engine.wasm -o frontend/public/wasm/sonica-audio-engine.wasm
        print_success "WASM optimization completed"
    else
        print_warning "wasm-opt not found. Install binaryen for better optimization:"
        print_warning "  npm install -g binaryen"
    fi
}

# Generate TypeScript definitions
generate_types() {
    print_status "Generating TypeScript definitions..."
    
    # Create TypeScript definition file
    cat > frontend/src/types/sonica-wasm.d.ts << 'EOF'
declare module 'sonica-audio-engine' {
  export class WasmAudioProcessor {
    constructor(sampleRate: number, windowSize: number, overlap: number);
    process_audio(audioData: Float32Array): any;
    extract_features(audioData: Float32Array): any;
    calculate_similarity(fingerprint1: any, fingerprint2: any): number;
    normalize_audio(audioData: Float32Array): Float32Array;
    reduce_noise(audioData: Float32Array): Float32Array;
    get_config(): any;
  }

  export class WasmAudioVisualizer {
    constructor(fftSize: number, sampleRate: number);
    compute_spectrum(audioData: Float32Array): Float32Array;
    get_frequency_bins(): Float32Array;
  }

  export class WasmPerformanceMonitor {
    constructor();
    start(): void;
    end(): number;
    static now(): number;
  }

  export function get_version(): string;
  export function get_build_info(): string;
}
EOF

    print_success "TypeScript definitions generated"
}

# Test WASM module
test_wasm() {
    print_status "Testing WASM module..."
    
    # Create a simple test HTML file
    cat > test-wasm.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>Sonica WASM Test</title>
</head>
<body>
    <h1>Sonica WebAssembly Test</h1>
    <div id="output"></div>
    
    <script type="module">
        import init, { WasmAudioProcessor, get_version } from './frontend/public/wasm/sonica-audio-engine.js';
        
        async function test() {
            try {
                await init();
                console.log('WASM module loaded successfully');
                console.log('Version:', get_version());
                
                const processor = new WasmAudioProcessor(44100, 4096, 0.5);
                console.log('Audio processor created');
                
                // Test with dummy data
                const testData = new Float32Array(44100).fill(0.1);
                const result = processor.process_audio(testData);
                console.log('Audio processing test passed');
                
                document.getElementById('output').innerHTML = 
                    '<p style="color: green;">✅ WASM module test passed!</p>' +
                    '<p>Version: ' + get_version() + '</p>';
                    
            } catch (error) {
                console.error('WASM test failed:', error);
                document.getElementById('output').innerHTML = 
                    '<p style="color: red;">❌ WASM module test failed: ' + error.message + '</p>';
            }
        }
        
        test();
    </script>
</body>
</html>
EOF

    print_success "Test file created: test-wasm.html"
    print_warning "Open test-wasm.html in a browser to test the WASM module"
}

# Main build function
main() {
    echo "🎵 Building Sonica WebAssembly Module"
    echo "====================================="
    
    check_wasm_pack
    build_wasm
    copy_wasm_files
    optimize_wasm
    generate_types
    test_wasm
    
    echo ""
    echo "🎉 WebAssembly build completed successfully!"
    echo ""
    echo "Files created:"
    echo "  - frontend/public/wasm/sonica-audio-engine.wasm"
    echo "  - frontend/public/wasm/sonica-audio-engine.js"
    echo "  - frontend/src/types/sonica-wasm.d.ts"
    echo "  - test-wasm.html"
    echo ""
    echo "Next steps:"
    echo "  1. Test the WASM module by opening test-wasm.html"
    echo "  2. Integrate with the frontend React components"
    echo "  3. Deploy to production with proper MIME types"
    echo ""
}

# Run main function
main "$@"
