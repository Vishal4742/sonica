#!/bin/bash

# Sonica Development Environment Setup Script

set -e

echo "🚀 Setting up Sonica development environment..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
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

# Check if Docker is installed
check_docker() {
    print_status "Checking Docker installation..."
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed. Please install Docker first."
        exit 1
    fi
    
    if ! command -v docker-compose &> /dev/null; then
        print_error "Docker Compose is not installed. Please install Docker Compose first."
        exit 1
    fi
    
    print_success "Docker and Docker Compose are installed"
}

# Check if required tools are installed
check_tools() {
    print_status "Checking required tools..."
    
    # Check Rust
    if ! command -v cargo &> /dev/null; then
        print_warning "Rust is not installed. Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source $HOME/.cargo/env
    else
        print_success "Rust is installed"
    fi
    
    # Check Python
    if ! command -v python3 &> /dev/null; then
        print_error "Python 3 is not installed. Please install Python 3.11+ first."
        exit 1
    else
        print_success "Python 3 is installed"
    fi
    
    # Check Node.js (for frontend development)
    if ! command -v node &> /dev/null; then
        print_warning "Node.js is not installed. Please install Node.js 18+ for frontend development."
    else
        print_success "Node.js is installed"
    fi
}

# Create environment files
create_env_files() {
    print_status "Creating environment files..."
    
    # Backend .env file
    if [ ! -f "backend/.env" ]; then
        cat > backend/.env << EOF
# Database
DATABASE_URL=postgresql://sonica:password@localhost:5432/sonica

# Redis
REDIS_URL=redis://localhost:6379

# Audio Engine
AUDIO_ENGINE_URL=http://localhost:8080

# Security
SECRET_KEY=your-secret-key-change-in-production-$(openssl rand -hex 32)

# Debug
DEBUG=true

# Vector Database
VECTOR_DB_PROVIDER=pinecone
VECTOR_DB_API_KEY=your-pinecone-api-key
VECTOR_DB_ENVIRONMENT=us-west1-gcp
VECTOR_DB_INDEX_NAME=sonica-music

# External APIs
SPOTIFY_CLIENT_ID=your-spotify-client-id
SPOTIFY_CLIENT_SECRET=your-spotify-client-secret
YOUTUBE_API_KEY=your-youtube-api-key
EOF
        print_success "Created backend/.env file"
    else
        print_warning "backend/.env already exists, skipping..."
    fi
    
    # Audio Engine .env file
    if [ ! -f "audio-engine/.env" ]; then
        cat > audio-engine/.env << EOF
# Database
DATABASE_URL=postgresql://sonica:password@localhost:5432/sonica

# Redis
REDIS_URL=redis://localhost:6379

# Logging
RUST_LOG=info

# Vector Database
VECTOR_DB_PROVIDER=pinecone
VECTOR_DB_API_KEY=your-pinecone-api-key
VECTOR_DB_ENVIRONMENT=us-west1-gcp
VECTOR_DB_INDEX_NAME=sonica-music
EOF
        print_success "Created audio-engine/.env file"
    else
        print_warning "audio-engine/.env already exists, skipping..."
    fi
}

# Build Rust audio engine
build_audio_engine() {
    print_status "Building Rust audio engine..."
    cd audio-engine
    
    # Install dependencies
    cargo build --release
    
    print_success "Rust audio engine built successfully"
    cd ..
}

# Install Python dependencies
install_python_deps() {
    print_status "Installing Python dependencies..."
    cd backend
    
    # Create virtual environment if it doesn't exist
    if [ ! -d "venv" ]; then
        python3 -m venv venv
    fi
    
    # Activate virtual environment
    source venv/bin/activate
    
    # Install dependencies
    pip install --upgrade pip
    pip install -r requirements.txt
    
    print_success "Python dependencies installed"
    cd ..
}

# Start development environment
start_services() {
    print_status "Starting development services..."
    
    # Start database and cache services
    docker-compose up -d postgres redis
    
    # Wait for services to be ready
    print_status "Waiting for services to be ready..."
    sleep 10
    
    # Run database migrations
    print_status "Running database migrations..."
    cd audio-engine
    cargo run --bin migrate
    cd ..
    
    print_success "Development environment is ready!"
}

# Main setup function
main() {
    echo "🎵 Welcome to Sonica Development Setup!"
    echo "======================================"
    
    check_docker
    check_tools
    create_env_files
    build_audio_engine
    install_python_deps
    start_services
    
    echo ""
    echo "🎉 Setup completed successfully!"
    echo ""
    echo "Next steps:"
    echo "1. Update the .env files with your API keys"
    echo "2. Start the full development environment:"
    echo "   docker-compose up -d"
    echo "3. Access the services:"
    echo "   - Backend API: http://localhost:8000"
    echo "   - Audio Engine: http://localhost:8080"
    echo "   - Database: localhost:5432"
    echo "   - Redis: localhost:6379"
    echo "   - Prometheus: http://localhost:9090"
    echo "   - Grafana: http://localhost:3000"
    echo ""
    echo "Happy coding! 🚀"
}

# Run main function
main "$@"
