# 🔌 Sonica API Specification

## 📋 Overview

The Sonica API provides lightning-fast music recognition services with sub-second response times. Built with RESTful principles and optimized for performance.

**Base URL**: `https://api.sonica.com/v1`  
**Authentication**: Bearer Token (JWT)  
**Rate Limits**: 100 requests/minute (free), 1000 requests/minute (premium)

## 🔐 Authentication

### Get Access Token
```http
POST /auth/token
Content-Type: application/json

{
  "grant_type": "client_credentials",
  "client_id": "your_client_id",
  "client_secret": "your_client_secret"
}
```

**Response:**
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "scope": "recognition:read recognition:write"
}
```

### Refresh Token
```http
POST /auth/refresh
Content-Type: application/json

{
  "refresh_token": "your_refresh_token"
}
```

## 🎵 Music Recognition

### Recognize Audio
```http
POST /recognize
Authorization: Bearer {access_token}
Content-Type: multipart/form-data

{
  "audio": <audio_file>,
  "format": "mp3|wav|flac|aac|ogg",
  "duration": 30,
  "language": "hi|en|auto"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "song": {
      "id": "song_12345",
      "title": "Tum Hi Ho",
      "artist": "Arijit Singh",
      "album": "Aashiqui 2",
      "language": "hi",
      "genre": "bollywood",
      "duration": 240,
      "release_year": 2013,
      "popularity_score": 0.95
    },
    "recognition": {
      "confidence": 0.98,
      "processing_time_ms": 245,
      "audio_quality": "high",
      "matched_segments": [
        {
          "start_time": 15.2,
          "end_time": 30.0,
          "confidence": 0.98
        }
      ]
    },
    "metadata": {
      "request_id": "req_67890",
      "timestamp": "2025-01-06T10:30:00Z",
      "api_version": "1.0"
    }
  }
}
```

### Streaming Recognition
```http
POST /recognize/stream
Authorization: Bearer {access_token}
Content-Type: application/json

{
  "stream_url": "wss://api.sonica.com/stream/audio",
  "format": "pcm",
  "sample_rate": 44100,
  "channels": 2
}
```

**WebSocket Response:**
```json
{
  "type": "recognition_result",
  "data": {
    "song": { /* song object */ },
    "confidence": 0.95,
    "timestamp": "2025-01-06T10:30:15Z"
  }
}
```

## 🔍 Search & Discovery

### Search Songs
```http
GET /songs/search?q=tum+hi+ho&language=hi&limit=10&offset=0
Authorization: Bearer {access_token}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "songs": [
      {
        "id": "song_12345",
        "title": "Tum Hi Ho",
        "artist": "Arijit Singh",
        "album": "Aashiqui 2",
        "language": "hi",
        "genre": "bollywood",
        "duration": 240,
        "release_year": 2013,
        "popularity_score": 0.95,
        "preview_url": "https://cdn.sonica.com/previews/song_12345.mp3"
      }
    ],
    "pagination": {
      "total": 1,
      "limit": 10,
      "offset": 0,
      "has_more": false
    }
  }
}
```

### Get Song Details
```http
GET /songs/{song_id}
Authorization: Bearer {access_token}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "song": {
      "id": "song_12345",
      "title": "Tum Hi Ho",
      "artist": "Arijit Singh",
      "album": "Aashiqui 2",
      "language": "hi",
      "genre": "bollywood",
      "duration": 240,
      "release_year": 2013,
      "popularity_score": 0.95,
      "lyrics": "Tum hi ho, tum hi ho...",
      "preview_url": "https://cdn.sonica.com/previews/song_12345.mp3",
      "full_audio_url": "https://cdn.sonica.com/audio/song_12345.mp3",
      "artwork_url": "https://cdn.sonica.com/artwork/song_12345.jpg",
      "related_songs": [
        {
          "id": "song_12346",
          "title": "Chahun Main Ya Naa",
          "artist": "Arijit Singh",
          "similarity": 0.85
        }
      ]
    }
  }
}
```

### Get Popular Songs
```http
GET /songs/popular?language=hi&genre=bollywood&limit=20
Authorization: Bearer {access_token}
```

## 👤 User Management

### Get User Profile
```http
GET /users/me
Authorization: Bearer {access_token}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "user": {
      "id": "user_12345",
      "email": "user@example.com",
      "name": "John Doe",
      "plan": "premium",
      "preferences": {
        "language": "hi",
        "genre": ["bollywood", "bhojpuri"],
        "notifications": true
      },
      "stats": {
        "total_recognitions": 1250,
        "favorite_songs": 45,
        "created_at": "2024-01-01T00:00:00Z"
      }
    }
  }
}
```

### Update User Preferences
```http
PUT /users/me/preferences
Authorization: Bearer {access_token}
Content-Type: application/json

{
  "language": "hi",
  "genre": ["bollywood", "bhojpuri"],
  "notifications": true
}
```

### Get Recognition History
```http
GET /users/me/history?limit=50&offset=0
Authorization: Bearer {access_token}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "recognitions": [
      {
        "id": "rec_12345",
        "song": {
          "id": "song_12345",
          "title": "Tum Hi Ho",
          "artist": "Arijit Singh"
        },
        "confidence": 0.98,
        "timestamp": "2025-01-06T10:30:00Z",
        "audio_duration": 15.2
      }
    ],
    "pagination": {
      "total": 1250,
      "limit": 50,
      "offset": 0,
      "has_more": true
    }
  }
}
```

### Add to Favorites
```http
POST /users/me/favorites
Authorization: Bearer {access_token}
Content-Type: application/json

{
  "song_id": "song_12345"
}
```

### Get Favorites
```http
GET /users/me/favorites?limit=50&offset=0
Authorization: Bearer {access_token}
```

## 📊 Analytics & Insights

### Get User Analytics
```http
GET /analytics/user
Authorization: Bearer {access_token}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "analytics": {
      "total_recognitions": 1250,
      "successful_recognitions": 1180,
      "accuracy_rate": 0.944,
      "average_confidence": 0.92,
      "most_recognized_genres": [
        {
          "genre": "bollywood",
          "count": 650,
          "percentage": 52.0
        },
        {
          "genre": "bhojpuri",
          "count": 400,
          "percentage": 32.0
        }
      ],
      "recognition_trends": {
        "daily": [
          {
            "date": "2025-01-01",
            "count": 25
          }
        ],
        "weekly": [
          {
            "week": "2025-W01",
            "count": 175
          }
        ]
      }
    }
  }
}
```

## 🎯 Recommendations

### Get Personalized Recommendations
```http
GET /recommendations?limit=20
Authorization: Bearer {access_token}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "recommendations": [
      {
        "song": {
          "id": "song_12346",
          "title": "Chahun Main Ya Naa",
          "artist": "Arijit Singh",
          "album": "Aashiqui 2",
          "language": "hi",
          "genre": "bollywood"
        },
        "reason": "Similar to your recent recognitions",
        "confidence": 0.92
      }
    ]
  }
}
```

## 🔧 Admin APIs

### Upload New Song
```http
POST /admin/songs
Authorization: Bearer {admin_token}
Content-Type: multipart/form-data

{
  "audio_file": <audio_file>,
  "title": "New Song",
  "artist": "Artist Name",
  "album": "Album Name",
  "language": "hi",
  "genre": "bollywood",
  "release_year": 2025
}
```

### Get System Statistics
```http
GET /admin/stats
Authorization: Bearer {admin_token}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "stats": {
      "total_songs": 100000,
      "total_users": 50000,
      "total_recognitions": 1000000,
      "average_recognition_time_ms": 245,
      "system_uptime": "99.9%",
      "active_users_24h": 5000
    }
  }
}
```

## ⚠️ Error Handling

### Error Response Format
```json
{
  "success": false,
  "error": {
    "code": "INVALID_AUDIO_FORMAT",
    "message": "Unsupported audio format. Supported formats: mp3, wav, flac, aac, ogg",
    "details": {
      "provided_format": "wma",
      "supported_formats": ["mp3", "wav", "flac", "aac", "ogg"]
    },
    "request_id": "req_67890",
    "timestamp": "2025-01-06T10:30:00Z"
  }
}
```

### Error Codes
| Code | HTTP Status | Description |
|------|-------------|-------------|
| `INVALID_AUDIO_FORMAT` | 400 | Unsupported audio format |
| `AUDIO_TOO_SHORT` | 400 | Audio duration less than 3 seconds |
| `AUDIO_TOO_LONG` | 400 | Audio duration more than 30 seconds |
| `AUDIO_QUALITY_LOW` | 400 | Audio quality too low for recognition |
| `RATE_LIMIT_EXCEEDED` | 429 | API rate limit exceeded |
| `UNAUTHORIZED` | 401 | Invalid or missing authentication token |
| `FORBIDDEN` | 403 | Insufficient permissions |
| `SONG_NOT_FOUND` | 404 | Song not found in database |
| `RECOGNITION_FAILED` | 422 | Audio recognition failed |
| `INTERNAL_ERROR` | 500 | Internal server error |

## 📈 Rate Limits

### Free Tier
- **Recognition**: 10 requests/day
- **Search**: 100 requests/day
- **User APIs**: 50 requests/day

### Premium Tier
- **Recognition**: 1000 requests/day
- **Search**: 10000 requests/day
- **User APIs**: 1000 requests/day

### Rate Limit Headers
```http
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1640995200
```

## 🔄 Webhooks

### Recognition Webhook
```http
POST /webhooks/recognition
Content-Type: application/json
X-Sonica-Signature: sha256=...

{
  "event": "recognition.completed",
  "data": {
    "user_id": "user_12345",
    "song_id": "song_12345",
    "confidence": 0.98,
    "processing_time_ms": 245,
    "timestamp": "2025-01-06T10:30:00Z"
  }
}
```

## 📱 SDK Examples

### JavaScript/TypeScript
```typescript
import { SonicaClient } from '@sonica/sdk';

const client = new SonicaClient({
  apiKey: 'your_api_key',
  baseUrl: 'https://api.sonica.com/v1'
});

// Recognize audio
const result = await client.recognize({
  audio: audioFile,
  format: 'mp3',
  language: 'hi'
});

console.log(result.song.title);
```

### Python
```python
from sonica import SonicaClient

client = SonicaClient(api_key='your_api_key')

# Recognize audio
result = client.recognize(
    audio_file='path/to/audio.mp3',
    format='mp3',
    language='hi'
)

print(result.song.title)
```

### React Native
```typescript
import { SonicaSDK } from '@sonica/react-native';

const sdk = new SonicaSDK({
  apiKey: 'your_api_key'
});

// Record and recognize
const result = await sdk.recordAndRecognize({
  duration: 15,
  language: 'hi'
});
```

## 🧪 Testing

### Test Audio Files
- **High Quality**: 320kbps MP3, 44.1kHz
- **Medium Quality**: 128kbps MP3, 44.1kHz
- **Low Quality**: 64kbps MP3, 22kHz

### Test Endpoints
```bash
# Test recognition
curl -X POST https://api.sonica.com/v1/recognize \
  -H "Authorization: Bearer your_token" \
  -F "audio=@test_audio.mp3" \
  -F "format=mp3" \
  -F "language=hi"

# Test search
curl -X GET "https://api.sonica.com/v1/songs/search?q=tum+hi+ho" \
  -H "Authorization: Bearer your_token"
```

This API specification ensures Sonica provides a comprehensive, fast, and reliable music recognition service.
