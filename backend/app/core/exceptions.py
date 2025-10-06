"""
Custom exceptions for Sonica API
"""

from typing import Optional, Dict, Any


class SonicaException(Exception):
    """Base exception for Sonica API"""
    
    def __init__(
        self,
        message: str,
        status_code: int = 500,
        error_code: str = "INTERNAL_ERROR",
        details: Optional[Dict[str, Any]] = None
    ):
        self.message = message
        self.status_code = status_code
        self.error_code = error_code
        self.details = details or {}
        super().__init__(self.message)


class AudioProcessingError(SonicaException):
    """Audio processing related errors"""
    
    def __init__(self, message: str, details: Optional[Dict[str, Any]] = None):
        super().__init__(
            message=message,
            status_code=422,
            error_code="AUDIO_PROCESSING_ERROR",
            details=details
        )


class InvalidAudioFormatError(SonicaException):
    """Invalid audio format error"""
    
    def __init__(self, format: str, supported_formats: list):
        super().__init__(
            message=f"Unsupported audio format: {format}",
            status_code=400,
            error_code="INVALID_AUDIO_FORMAT",
            details={
                "provided_format": format,
                "supported_formats": supported_formats
            }
        )


class AudioTooShortError(SonicaException):
    """Audio duration too short error"""
    
    def __init__(self, duration: float, minimum: float):
        super().__init__(
            message=f"Audio duration too short: {duration}s (minimum: {minimum}s)",
            status_code=400,
            error_code="AUDIO_TOO_SHORT",
            details={
                "duration": duration,
                "minimum": minimum
            }
        )


class AudioTooLongError(SonicaException):
    """Audio duration too long error"""
    
    def __init__(self, duration: float, maximum: float):
        super().__init__(
            message=f"Audio duration too long: {duration}s (maximum: {maximum}s)",
            status_code=400,
            error_code="AUDIO_TOO_LONG",
            details={
                "duration": duration,
                "maximum": maximum
            }
        )


class RecognitionFailedError(SonicaException):
    """Recognition failed error"""
    
    def __init__(self, reason: str):
        super().__init__(
            message=f"Recognition failed: {reason}",
            status_code=422,
            error_code="RECOGNITION_FAILED",
            details={"reason": reason}
        )


class SongNotFoundError(SonicaException):
    """Song not found error"""
    
    def __init__(self, song_id: str):
        super().__init__(
            message=f"Song not found: {song_id}",
            status_code=404,
            error_code="SONG_NOT_FOUND",
            details={"song_id": song_id}
        )


class RateLimitExceededError(SonicaException):
    """Rate limit exceeded error"""
    
    def __init__(self, limit: int, window: str):
        super().__init__(
            message=f"Rate limit exceeded: {limit} requests per {window}",
            status_code=429,
            error_code="RATE_LIMIT_EXCEEDED",
            details={
                "limit": limit,
                "window": window
            }
        )


class AuthenticationError(SonicaException):
    """Authentication error"""
    
    def __init__(self, reason: str = "Authentication failed"):
        super().__init__(
            message=reason,
            status_code=401,
            error_code="AUTHENTICATION_FAILED",
            details={"reason": reason}
        )


class AuthorizationError(SonicaException):
    """Authorization error"""
    
    def __init__(self, reason: str = "Insufficient permissions"):
        super().__init__(
            message=reason,
            status_code=403,
            error_code="AUTHORIZATION_FAILED",
            details={"reason": reason}
        )


class DatabaseError(SonicaException):
    """Database error"""
    
    def __init__(self, message: str, details: Optional[Dict[str, Any]] = None):
        super().__init__(
            message=message,
            status_code=500,
            error_code="DATABASE_ERROR",
            details=details
        )


class CacheError(SonicaException):
    """Cache error"""
    
    def __init__(self, message: str, details: Optional[Dict[str, Any]] = None):
        super().__init__(
            message=message,
            status_code=500,
            error_code="CACHE_ERROR",
            details=details
        )


class VectorDatabaseError(SonicaException):
    """Vector database error"""
    
    def __init__(self, message: str, details: Optional[Dict[str, Any]] = None):
        super().__init__(
            message=message,
            status_code=500,
            error_code="VECTOR_DATABASE_ERROR",
            details=details
        )


class ExternalAPIError(SonicaException):
    """External API error"""
    
    def __init__(self, service: str, message: str, details: Optional[Dict[str, Any]] = None):
        super().__init__(
            message=f"{service} API error: {message}",
            status_code=502,
            error_code="EXTERNAL_API_ERROR",
            details={
                "service": service,
                "message": message,
                **(details or {})
            }
        )
