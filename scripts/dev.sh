#!/bin/bash

# Sonica Development Script

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

# Function to show help
show_help() {
    echo "Sonica Development Script"
    echo ""
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  start       Start all development services"
    echo "  stop        Stop all development services"
    echo "  restart     Restart all development services"
    echo "  logs        Show logs for all services"
    echo "  build       Build all services"
    echo "  test        Run tests for all services"
    echo "  clean       Clean up containers and volumes"
    echo "  status      Show status of all services"
    echo "  backend     Start only backend services"
    echo "  audio       Start only audio engine"
    echo "  db          Start only database services"
    echo "  help        Show this help message"
    echo ""
}

# Start all services
start_services() {
    print_status "Starting all Sonica development services..."
    docker-compose up -d
    print_success "All services started"
    show_status
}

# Stop all services
stop_services() {
    print_status "Stopping all Sonica development services..."
    docker-compose down
    print_success "All services stopped"
}

# Restart all services
restart_services() {
    print_status "Restarting all Sonica development services..."
    docker-compose restart
    print_success "All services restarted"
}

# Show logs
show_logs() {
    print_status "Showing logs for all services..."
    docker-compose logs -f
}

# Build all services
build_services() {
    print_status "Building all Sonica services..."
    
    # Build Rust audio engine
    print_status "Building Rust audio engine..."
    cd audio-engine
    cargo build --release
    cd ..
    
    # Build Docker images
    print_status "Building Docker images..."
    docker-compose build
    
    print_success "All services built"
}

# Run tests
run_tests() {
    print_status "Running tests for all services..."
    
    # Test Rust audio engine
    print_status "Testing Rust audio engine..."
    cd audio-engine
    cargo test
    cd ..
    
    # Test Python backend
    print_status "Testing Python backend..."
    cd backend
    if [ -d "venv" ]; then
        source venv/bin/activate
        python -m pytest tests/ -v
    else
        print_warning "Virtual environment not found, skipping Python tests"
    fi
    cd ..
    
    print_success "All tests completed"
}

# Clean up
clean_up() {
    print_status "Cleaning up containers and volumes..."
    docker-compose down -v --remove-orphans
    docker system prune -f
    print_success "Cleanup completed"
}

# Show status
show_status() {
    print_status "Service Status:"
    echo ""
    docker-compose ps
    echo ""
    
    # Check health endpoints
    print_status "Health Checks:"
    
    # Backend health
    if curl -s http://localhost:8000/health > /dev/null 2>&1; then
        print_success "Backend API: Healthy"
    else
        print_warning "Backend API: Not responding"
    fi
    
    # Audio engine health
    if curl -s http://localhost:8080/health > /dev/null 2>&1; then
        print_success "Audio Engine: Healthy"
    else
        print_warning "Audio Engine: Not responding"
    fi
    
    # Database health
    if docker-compose exec -T postgres pg_isready -U sonica -d sonica > /dev/null 2>&1; then
        print_success "PostgreSQL: Healthy"
    else
        print_warning "PostgreSQL: Not responding"
    fi
    
    # Redis health
    if docker-compose exec -T redis redis-cli ping > /dev/null 2>&1; then
        print_success "Redis: Healthy"
    else
        print_warning "Redis: Not responding"
    fi
}

# Start backend services only
start_backend() {
    print_status "Starting backend services..."
    docker-compose up -d postgres redis backend
    print_success "Backend services started"
}

# Start audio engine only
start_audio() {
    print_status "Starting audio engine..."
    docker-compose up -d postgres redis audio-engine
    print_success "Audio engine started"
}

# Start database services only
start_db() {
    print_status "Starting database services..."
    docker-compose up -d postgres redis
    print_success "Database services started"
}

# Main function
main() {
    case "${1:-help}" in
        start)
            start_services
            ;;
        stop)
            stop_services
            ;;
        restart)
            restart_services
            ;;
        logs)
            show_logs
            ;;
        build)
            build_services
            ;;
        test)
            run_tests
            ;;
        clean)
            clean_up
            ;;
        status)
            show_status
            ;;
        backend)
            start_backend
            ;;
        audio)
            start_audio
            ;;
        db)
            start_db
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            print_error "Unknown command: $1"
            show_help
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
