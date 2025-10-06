#!/bin/bash

# Setup script for Pinecone vector database

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

# Check if Python is installed
check_python() {
    print_status "Checking Python installation..."
    if ! command -v python3 &> /dev/null; then
        print_error "Python 3 is not installed. Please install Python 3.8 or higher."
        exit 1
    fi
    
    python_version=$(python3 --version | cut -d' ' -f2)
    print_success "Python $python_version is installed"
}

# Install required Python packages
install_dependencies() {
    print_status "Installing Python dependencies..."
    
    pip3 install --upgrade pip
    pip3 install pinecone-client httpx asyncio
    
    print_success "Dependencies installed successfully"
}

# Create Pinecone index
create_pinecone_index() {
    print_status "Creating Pinecone index..."
    
    # Check if environment variables are set
    if [ -z "$VECTOR_DB_API_KEY" ]; then
        print_error "VECTOR_DB_API_KEY environment variable is not set"
        print_warning "Please set your Pinecone API key:"
        print_warning "export VECTOR_DB_API_KEY=your-api-key"
        exit 1
    fi
    
    if [ -z "$VECTOR_DB_ENVIRONMENT" ]; then
        print_error "VECTOR_DB_ENVIRONMENT environment variable is not set"
        print_warning "Please set your Pinecone environment:"
        print_warning "export VECTOR_DB_ENVIRONMENT=us-west1-gcp"
        exit 1
    fi
    
    if [ -z "$VECTOR_DB_INDEX_NAME" ]; then
        print_error "VECTOR_DB_INDEX_NAME environment variable is not set"
        print_warning "Please set your Pinecone index name:"
        print_warning "export VECTOR_DB_INDEX_NAME=sonica-music"
        exit 1
    fi
    
    # Create Python script to setup index
    cat > setup_pinecone.py << 'EOF'
import os
import time
import pinecone
from pinecone import Pinecone, ServerlessSpec

def setup_pinecone_index():
    # Initialize Pinecone
    pc = Pinecone(api_key=os.getenv('VECTOR_DB_API_KEY'))
    
    index_name = os.getenv('VECTOR_DB_INDEX_NAME', 'sonica-music')
    environment = os.getenv('VECTOR_DB_ENVIRONMENT', 'us-west1-gcp')
    dimensions = int(os.getenv('VECTOR_DB_DIMENSIONS', '1024'))
    
    print(f"Setting up Pinecone index: {index_name}")
    print(f"Environment: {environment}")
    print(f"Dimensions: {dimensions}")
    
    # Check if index already exists
    if index_name in pc.list_indexes().names():
        print(f"Index {index_name} already exists")
        
        # Get index stats
        index = pc.Index(index_name)
        stats = index.describe_index_stats()
        print(f"Index stats: {stats}")
        
        return True
    
    # Create new index
    try:
        pc.create_index(
            name=index_name,
            dimension=dimensions,
            metric='cosine',
            spec=ServerlessSpec(
                cloud='aws',
                region=environment
            )
        )
        
        print(f"Index {index_name} created successfully")
        
        # Wait for index to be ready
        print("Waiting for index to be ready...")
        while not pc.describe_index(index_name).status['ready']:
            time.sleep(1)
        
        print("Index is ready!")
        return True
        
    except Exception as e:
        print(f"Error creating index: {e}")
        return False

if __name__ == "__main__":
    success = setup_pinecone_index()
    if success:
        print("Pinecone setup completed successfully!")
    else:
        print("Pinecone setup failed!")
        exit(1)
EOF

    # Run the setup script
    python3 setup_pinecone.py
    
    if [ $? -eq 0 ]; then
        print_success "Pinecone index created successfully"
    else
        print_error "Failed to create Pinecone index"
        exit 1
    fi
    
    # Clean up
    rm setup_pinecone.py
}

# Test vector database connection
test_connection() {
    print_status "Testing vector database connection..."
    
    # Create test script
    cat > test_vector_db.py << 'EOF'
import os
import asyncio
import httpx
import json

async def test_vector_db():
    api_key = os.getenv('VECTOR_DB_API_KEY')
    environment = os.getenv('VECTOR_DB_ENVIRONMENT')
    index_name = os.getenv('VECTOR_DB_INDEX_NAME')
    
    base_url = f"https://{index_name}-{environment}.svc.pinecone.io"
    
    async with httpx.AsyncClient() as client:
        try:
            # Test index stats
            response = await client.post(
                f"{base_url}/describe_index_stats",
                headers={
                    "Api-Key": api_key,
                    "Content-Type": "application/json"
                }
            )
            
            if response.status_code == 200:
                stats = response.json()
                print(f"✅ Connection successful!")
                print(f"Index stats: {stats}")
                return True
            else:
                print(f"❌ Connection failed: {response.status_code}")
                print(f"Response: {response.text}")
                return False
                
        except Exception as e:
            print(f"❌ Connection error: {e}")
            return False

if __name__ == "__main__":
    success = asyncio.run(test_vector_db())
    if not success:
        exit(1)
EOF

    # Run test
    python3 test_vector_db.py
    
    if [ $? -eq 0 ]; then
        print_success "Vector database connection test passed"
    else
        print_error "Vector database connection test failed"
        exit 1
    fi
    
    # Clean up
    rm test_vector_db.py
}

# Create sample data
create_sample_data() {
    print_status "Creating sample data for testing..."
    
    # Create sample script
    cat > create_sample_data.py << 'EOF'
import os
import asyncio
import httpx
import json
import uuid
import time

async def create_sample_data():
    api_key = os.getenv('VECTOR_DB_API_KEY')
    environment = os.getenv('VECTOR_DB_ENVIRONMENT')
    index_name = os.getenv('VECTOR_DB_INDEX_NAME')
    
    base_url = f"https://{index_name}-{environment}.svc.pinecone.io"
    
    # Sample fingerprint vectors (1024 dimensions)
    sample_vectors = []
    
    for i in range(5):
        # Create a sample vector with some pattern
        vector = [0.1] * 1024
        # Add some variation
        for j in range(0, 1024, 100):
            vector[j] = 0.5 + (i * 0.1)
        
        sample_vectors.append({
            "id": f"sample_fingerprint_{i}",
            "values": vector,
            "metadata": {
                "song_id": str(uuid.uuid4()),
                "title": f"Sample Song {i+1}",
                "artist": f"Sample Artist {i+1}",
                "language": "en",
                "genre": "pop",
                "album": f"Sample Album {i+1}",
                "popularity_score": 0.8,
                "created_at": time.time()
            }
        })
    
    async with httpx.AsyncClient() as client:
        try:
            # Upsert sample vectors
            response = await client.post(
                f"{base_url}/vectors/upsert",
                headers={
                    "Api-Key": api_key,
                    "Content-Type": "application/json"
                },
                json={
                    "vectors": sample_vectors
                }
            )
            
            if response.status_code == 200:
                print(f"✅ Sample data created successfully!")
                print(f"Created {len(sample_vectors)} sample vectors")
                return True
            else:
                print(f"❌ Failed to create sample data: {response.status_code}")
                print(f"Response: {response.text}")
                return False
                
        except Exception as e:
            print(f"❌ Error creating sample data: {e}")
            return False

if __name__ == "__main__":
    success = asyncio.run(create_sample_data())
    if not success:
        exit(1)
EOF

    # Run sample data creation
    python3 create_sample_data.py
    
    if [ $? -eq 0 ]; then
        print_success "Sample data created successfully"
    else
        print_error "Failed to create sample data"
        exit 1
    fi
    
    # Clean up
    rm create_sample_data.py
}

# Main setup function
main() {
    echo "🎵 Setting up Sonica Vector Database (Pinecone)"
    echo "=============================================="
    
    # Check environment variables
    if [ -z "$VECTOR_DB_API_KEY" ] || [ -z "$VECTOR_DB_ENVIRONMENT" ] || [ -z "$VECTOR_DB_INDEX_NAME" ]; then
        print_warning "Environment variables not set. Please set:"
        print_warning "  export VECTOR_DB_API_KEY=your-pinecone-api-key"
        print_warning "  export VECTOR_DB_ENVIRONMENT=us-west1-gcp"
        print_warning "  export VECTOR_DB_INDEX_NAME=sonica-music"
        print_warning "  export VECTOR_DB_DIMENSIONS=1024"
        echo ""
        print_warning "You can get your Pinecone API key from: https://app.pinecone.io/"
        exit 1
    fi
    
    check_python
    install_dependencies
    create_pinecone_index
    test_connection
    create_sample_data
    
    echo ""
    echo "🎉 Vector database setup completed successfully!"
    echo ""
    echo "Next steps:"
    echo "  1. Update your .env file with the vector database configuration"
    echo "  2. Start the backend service: python -m uvicorn main:app --reload"
    echo "  3. Test the recognition API with sample audio"
    echo ""
    echo "Configuration:"
    echo "  Index Name: $VECTOR_DB_INDEX_NAME"
    echo "  Environment: $VECTOR_DB_ENVIRONMENT"
    echo "  Dimensions: ${VECTOR_DB_DIMENSIONS:-1024}"
    echo ""
}

# Run main function
main "$@"
