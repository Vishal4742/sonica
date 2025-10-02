# SONICA Audio Recognition System

A high-performance audio recognition and speech-to-text system built with Rust and Actix Web.

## Features

- **Audio Recognition**: Advanced speech-to-text processing with multiple model support
- **Fast & Secure**: Built with Rust for maximum performance and memory safety
- **Modern Web Framework**: Powered by Actix Web for async HTTP handling
- **Multiple Audio Formats**: Support for WAV, MP3, FLAC, M4A, AAC, OGG
- **Database Support**: PostgreSQL with SQLx for type-safe database operations
- **Caching**: Redis integration for high-performance caching
- **File Storage**: AWS S3 integration for scalable audio file storage
- **Authentication**: JWT-based authentication system
- **CORS Support**: Built-in CORS middleware for cross-origin requests
- **Logging**: Comprehensive logging with env_logger
- **ML Integration**: Support for Whisper, OpenAI, Google Speech, and Azure Speech APIs

## Project Structure

```
sonica/
├── backend/           # Rust backend application
│   ├── src/
│   │   └── main.rs   # Main application entry point with audio recognition endpoints
│   └── Cargo.toml    # Rust dependencies including audio processing libraries
├── database/
│   └── schema.sql    # PostgreSQL database schema for audio recognition system
├── env.example       # Environment variables template for audio processing
├── .gitignore        # Git ignore rules for Rust projects
└── README.md         # This file
```

## Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- [PostgreSQL](https://www.postgresql.org/) (12 or later)
- [Redis](https://redis.io/) (6 or later)
- [AWS Account](https://aws.amazon.com/) (for S3 storage)
- Audio processing libraries (FFmpeg recommended for format conversion)
- Machine Learning models (Whisper, OpenAI API, or other STT services)

## Quick Start

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd sonica
   ```

2. **Set up environment variables**
   ```bash
   cp env.example .env
   # Edit .env with your configuration
   ```

3. **Set up the database**
   ```bash
   # Create PostgreSQL database
   createdb sonica_audio_db
   
   # Run the schema
   psql sonica_audio_db < database/schema.sql
   ```

4. **Start Redis server**
   ```bash
   redis-server
   ```

5. **Run the application**
   ```bash
   cd backend
   cargo run
   ```

The server will start on `http://localhost:8080` by default.

## API Endpoints

### Core Endpoints
- `GET /` - Welcome message with API information
- `GET /health` - Health check endpoint

### Audio Recognition Endpoints
- `POST /api/audio/upload` - Upload audio file for processing
- `POST /api/audio/recognize` - Perform speech-to-text recognition
- `GET /api/audio/status/{id}` - Check recognition job status

### Example Usage

**Upload Audio File:**
```bash
curl -X POST http://localhost:8080/api/audio/upload \
  -F "file=@audio_sample.wav"
```

**Recognize Audio:**
```bash
curl -X POST http://localhost:8080/api/audio/recognize \
  -F "file=@audio_sample.wav"
```

**Check Status:**
```bash
curl http://localhost:8080/api/audio/status/{recognition_id}
```

## Environment Variables

Copy `env.example` to `.env` and configure the following variables:

### Required
- `DATABASE_URL` - PostgreSQL connection string
- `REDIS_URL` - Redis connection string
- `JWT_SECRET` - Secret key for JWT tokens

### Audio Processing
- `MAX_AUDIO_FILE_SIZE` - Maximum audio file size in bytes (default: 50MB)
- `ALLOWED_AUDIO_FORMATS` - Comma-separated list of allowed formats
- `AUDIO_UPLOAD_PATH` - Directory for uploaded audio files
- `DEFAULT_MODEL_NAME` - Default ML model for recognition
- `DEFAULT_LANGUAGE` - Default language for recognition

### Optional
- `HOST` - Server host (default: 127.0.0.1)
- `PORT` - Server port (default: 8080)
- `AWS_*` - AWS configuration for S3 storage
- `OPENAI_API_KEY` - OpenAI API key for speech recognition
- `GOOGLE_SPEECH_API_KEY` - Google Speech API key
- `AZURE_SPEECH_*` - Azure Speech Services configuration

## Development

### Running in Development Mode

```bash
cd backend
cargo run
```

### Building for Production

```bash
cd backend
cargo build --release
```

### Running Tests

```bash
cd backend
cargo test
```

## Database Schema

The application includes a comprehensive database schema designed for audio recognition with the following main entities:

- **Users** - User accounts and authentication
- **Audio Files** - Audio file metadata and storage information
- **Recognition Jobs** - Audio recognition processing jobs and status
- **Recognition Results** - Speech-to-text results with timestamps and confidence scores
- **Audio Features** - Extracted audio features for ML model training
- **Model Configs** - Machine learning model configurations and versions
- **Processing Logs** - Detailed logging for audio processing operations

See `database/schema.sql` for the complete schema definition.

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For support and questions, please open an issue in the GitHub repository.
