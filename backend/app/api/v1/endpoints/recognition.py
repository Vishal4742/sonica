"""
Recognition endpoints for music identification
"""

import time
import uuid
from typing import Optional
from fastapi import APIRouter, Depends, HTTPException, UploadFile, File, Form
from fastapi.responses import JSONResponse

from app.core.config import settings
from app.core.exceptions import (
    AudioProcessingError,
    InvalidAudioFormatError,
    AudioTooShortError,
    AudioTooLongError,
    RecognitionFailedError
)
from app.models.recognition import RecognitionRequest, RecognitionResponse, RecognitionResult
from app.services.audio_service import AudioService
from app.services.recognition_service import RecognitionService
from app.utils.audio_utils import validate_audio_file, get_audio_duration

router = APIRouter()


@router.post("/", response_model=RecognitionResponse)
async def recognize_music(
    audio_file: UploadFile = File(...),
    format: Optional[str] = Form(None),
    duration: Optional[int] = Form(None),
    language: Optional[str] = Form("auto"),
    audio_service: AudioService = Depends(),
    recognition_service: RecognitionService = Depends()
):
    """
    Recognize music from uploaded audio file
    
    - **audio_file**: Audio file to recognize (MP3, WAV, FLAC, AAC, OGG)
    - **format**: Audio format (auto-detected if not provided)
    - **duration**: Recording duration in seconds
    - **language**: Language hint for recognition (hi, en, auto)
    """
    start_time = time.time()
    request_id = str(uuid.uuid4())
    
    try:
        # Validate audio file
        if not audio_file.content_type or not audio_file.content_type.startswith("audio/"):
            raise InvalidAudioFormatError(
                format=audio_file.content_type or "unknown",
                supported_formats=["audio/mpeg", "audio/wav", "audio/flac", "audio/aac", "audio/ogg"]
            )
        
        # Read audio data
        audio_data = await audio_file.read()
        
        if len(audio_data) > settings.MAX_AUDIO_SIZE:
            raise HTTPException(
                status_code=413,
                detail=f"Audio file too large. Maximum size: {settings.MAX_AUDIO_SIZE / (1024*1024):.1f}MB"
            )
        
        # Validate audio format and duration
        audio_info = await validate_audio_file(audio_data, audio_file.content_type)
        
        if audio_info.duration < settings.MIN_AUDIO_DURATION:
            raise AudioTooShortError(
                duration=audio_info.duration,
                minimum=settings.MIN_AUDIO_DURATION
            )
        
        if audio_info.duration > settings.MAX_AUDIO_DURATION:
            raise AudioTooLongError(
                duration=audio_info.duration,
                maximum=settings.MAX_AUDIO_DURATION
            )
        
        # Process audio for recognition
        processed_audio = await audio_service.process_audio(audio_data, audio_info)
        
        # Perform recognition
        recognition_result = await recognition_service.recognize(
            processed_audio,
            language=language,
            request_id=request_id
        )
        
        processing_time = (time.time() - start_time) * 1000  # Convert to milliseconds
        
        if not recognition_result:
            raise RecognitionFailedError("No matching song found")
        
        # Log recognition result
        await recognition_service.log_recognition(
            song_id=recognition_result.song.id,
            confidence=recognition_result.confidence,
            processing_time_ms=int(processing_time),
            request_id=request_id
        )
        
        return RecognitionResponse(
            success=True,
            data=recognition_result,
            metadata={
                "request_id": request_id,
                "processing_time_ms": int(processing_time),
                "audio_duration": audio_info.duration,
                "audio_quality": audio_info.quality,
                "api_version": "1.0"
            }
        )
        
    except Exception as e:
        processing_time = (time.time() - start_time) * 1000
        
        # Log error
        await recognition_service.log_error(
            error=str(e),
            processing_time_ms=int(processing_time),
            request_id=request_id
        )
        
        raise


@router.post("/stream")
async def recognize_streaming_audio(
    request: RecognitionRequest,
    recognition_service: RecognitionService = Depends()
):
    """
    Recognize music from streaming audio data
    
    This endpoint is designed for real-time audio streaming recognition
    """
    request_id = str(uuid.uuid4())
    start_time = time.time()
    
    try:
        # Process streaming audio
        recognition_result = await recognition_service.recognize_streaming(
            request.audio_data,
            request.language,
            request_id
        )
        
        processing_time = (time.time() - start_time) * 1000
        
        return RecognitionResponse(
            success=True,
            data=recognition_result,
            metadata={
                "request_id": request_id,
                "processing_time_ms": int(processing_time),
                "streaming": True,
                "api_version": "1.0"
            }
        )
        
    except Exception as e:
        processing_time = (time.time() - start_time) * 1000
        
        await recognition_service.log_error(
            error=str(e),
            processing_time_ms=int(processing_time),
            request_id=request_id
        )
        
        raise


from app.core.auth import get_current_user
from app.models.user import User # Assuming you have a user model

@router.get("/history")
async def get_recognition_history(
    limit: int = 50,
    offset: int = 0,
    recognition_service: RecognitionService = Depends(),
    current_user: User = Depends(get_current_user)
):
    """
    Get recognition history for the current user
    """
    try:
        history = await recognition_service.get_recognition_history(
            user_id=current_user.id, # Pass user_id to the service
            limit=limit,
            offset=offset
        )
        
        return {
            "success": True,
            "data": {
                "recognitions": history,
                "pagination": {
                    "limit": limit,
                    "offset": offset,
                    "has_more": len(history) == limit
                }
            }
        }
        
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@router.delete("/history/{recognition_id}")
async def delete_recognition_history(
    recognition_id: str,
    recognition_service: RecognitionService = Depends(),
    current_user: User = Depends(get_current_user)
):
    """
    Delete a specific recognition from history
    """
    try:
        await recognition_service.delete_recognition_history(
            recognition_id=recognition_id,
            user_id=current_user.id # Pass user_id to prevent IDOR
        )
        
        return {
            "success": True,
            "message": "Recognition deleted successfully"
        }
        
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))
