"""
Recognition service for music identification using vector database
"""

import time
import logging
from typing import Optional, List, Dict, Any
from uuid import UUID
import asyncio

from app.core.config import settings
from app.core.exceptions import RecognitionError, AudioProcessingError, VectorDatabaseError
from app.models.recognition import RecognitionResult, SongInfo
from app.services.audio_service import AudioService
from app.services.database_service import DatabaseService
from app.services.vector_service import get_vector_service, VectorDatabaseService

logger = logging.getLogger(__name__)


class RecognitionService:
    """Service for music recognition using vector database"""
    
    def __init__(
        self,
        audio_service: AudioService,
        database_service: DatabaseService,
        vector_service: VectorDatabaseService
    ):
        self.audio_service = audio_service
        self.database_service = database_service
        self.vector_service = vector_service
        self.recognition_cache = {}  # Simple in-memory cache
    
    async def recognize(
        self,
        audio_data: bytes,
        language: Optional[str] = None,
        request_id: Optional[str] = None
    ) -> Optional[RecognitionResult]:
        """
        Recognize music from audio data using vector database
        
        Args:
            audio_data: Raw audio data
            language: Language hint for recognition
            request_id: Request ID for tracking
            
        Returns:
            RecognitionResult if match found, None otherwise
        """
        start_time = time.time()
        
        try:
            # Extract audio features and generate fingerprint
            audio_features = await self.audio_service.extract_features(audio_data)
            fingerprint_data = await self.audio_service.generate_fingerprint(audio_data)
            
            # Search for similar fingerprints using vector database
            vector_results = await self.vector_service.search_similar_fingerprints(
                fingerprint_data=fingerprint_data,
                top_k=settings.RECOGNITION_MAX_CANDIDATES,
                language_filter=language,
                genre_filter=None
            )
            
            if not vector_results:
                logger.info(f"No similar fingerprints found for request {request_id}")
                return None
            
            # Find the best match above threshold
            best_match = None
            best_score = 0.0
            
            for result in vector_results:
                score = result.get("score", 0.0)
                
                if score > settings.RECOGNITION_THRESHOLD and score > best_score:
                    # Extract song ID from metadata
                    metadata = result.get("metadata", {})
                    song_id_str = metadata.get("song_id")
                    
                    if song_id_str:
                        try:
                            song_id = UUID(song_id_str)
                            song = await self.database_service.get_song(song_id)
                            
                            if song:
                                best_score = score
                                best_match = RecognitionResult(
                                    song=SongInfo(
                                        id=song.id,
                                        title=song.title,
                                        artist=song.artist,
                                        album=song.album,
                                        genre=song.genre,
                                        language=song.language,
                                        duration=song.duration,
                                        release_date=song.release_date,
                                        popularity_score=song.popularity_score,
                                        spotify_id=song.spotify_id,
                                        youtube_id=song.youtube_id
                                    ),
                                    confidence=score,
                                    match_type="vector_similarity",
                                    processing_time_ms=int((time.time() - start_time) * 1000)
                                )
                        except (ValueError, TypeError) as e:
                            logger.warning(f"Invalid song ID in vector result: {song_id_str}, error: {e}")
                            continue
            
            if best_match:
                logger.info(
                    f"Recognition successful for request {request_id}: "
                    f"song_id={best_match.song.id}, confidence={best_match.confidence:.3f}"
                )
                
                # Cache the result
                if request_id:
                    self.recognition_cache[request_id] = best_match
            
            return best_match
            
        except VectorDatabaseError as e:
            logger.error(f"Vector database error during recognition: {e}")
            raise RecognitionError(f"Vector database error: {str(e)}")
        except Exception as e:
            logger.error(f"Recognition failed for request {request_id}: {e}")
            raise RecognitionError(f"Recognition failed: {str(e)}")
    
    async def recognize_streaming(
        self,
        audio_data: bytes,
        language: Optional[str] = None,
        request_id: Optional[str] = None
    ) -> Optional[RecognitionResult]:
        """
        Recognize music from streaming audio data
        
        This method is optimized for real-time streaming recognition
        """
        # For streaming, we use a smaller window and faster processing
        try:
            # Process audio in chunks for streaming
            chunk_size = 1024 * 1024  # 1MB chunks
            audio_chunks = [audio_data[i:i+chunk_size] for i in range(0, len(audio_data), chunk_size)]
            
            # Process the first chunk for quick recognition
            if audio_chunks:
                return await self.recognize(audio_chunks[0], language, request_id)
            
            return None
            
        except Exception as e:
            logger.error(f"Streaming recognition failed for request {request_id}: {e}")
            raise RecognitionError(f"Streaming recognition failed: {str(e)}")
    
    async def add_song_to_database(
        self,
        song_data: Dict[str, Any],
        audio_data: bytes
    ) -> UUID:
        """
        Add a new song to the database and vector database
        
        Args:
            song_data: Song metadata
            audio_data: Raw audio data
            
        Returns:
            UUID of the created song
        """
        try:
            # Generate fingerprint
            fingerprint_data = await self.audio_service.generate_fingerprint(audio_data)
            
            # Create song in database
            song_id = await self.database_service.create_song(song_data)
            
            # Prepare metadata for vector database
            metadata = {
                "title": song_data.get("title", ""),
                "artist": song_data.get("artist", ""),
                "language": song_data.get("language", "en"),
                "genre": song_data.get("genre", ""),
                "album": song_data.get("album", ""),
                "popularity_score": song_data.get("popularity_score", 0.0)
            }
            
            # Add to vector database
            await self.vector_service.add_fingerprint(
                song_id=song_id,
                fingerprint_data=fingerprint_data,
                metadata=metadata
            )
            
            logger.info(f"Successfully added song {song_id} to database and vector database")
            return song_id
            
        except Exception as e:
            logger.error(f"Failed to add song to database: {e}")
            raise RecognitionError(f"Failed to add song: {str(e)}")
    
    async def batch_add_songs(
        self,
        songs_data: List[Dict[str, Any]]
    ) -> List[UUID]:
        """
        Batch add multiple songs to the database
        
        Args:
            songs_data: List of song data with audio_data
            
        Returns:
            List of created song UUIDs
        """
        try:
            created_songs = []
            
            # Prepare batch data for vector database
            vector_batch = []
            
            for song_data in songs_data:
                # Create song in database
                song_id = await self.database_service.create_song(song_data)
                created_songs.append(song_id)
                
                # Generate fingerprint
                audio_data = song_data.get("audio_data")
                if audio_data:
                    fingerprint_data = await self.audio_service.generate_fingerprint(audio_data)
                    
                    # Prepare metadata
                    metadata = {
                        "title": song_data.get("title", ""),
                        "artist": song_data.get("artist", ""),
                        "language": song_data.get("language", "en"),
                        "genre": song_data.get("genre", ""),
                        "album": song_data.get("album", ""),
                        "popularity_score": song_data.get("popularity_score", 0.0)
                    }
                    
                    vector_batch.append((song_id, fingerprint_data, metadata))
            
            # Batch upsert to vector database
            if vector_batch:
                await self.vector_service.batch_upsert_fingerprints(vector_batch)
            
            logger.info(f"Successfully batch added {len(created_songs)} songs")
            return created_songs
            
        except Exception as e:
            logger.error(f"Failed to batch add songs: {e}")
            raise RecognitionError(f"Batch add failed: {str(e)}")
    
    async def get_recognition_history(
        self,
        limit: int = 50,
        offset: int = 0
    ) -> List[Dict[str, Any]]:
        """Get recognition history from database"""
        try:
            return await self.database_service.get_recognition_history(
                limit=limit,
                offset=offset
            )
        except Exception as e:
            logger.error(f"Failed to get recognition history: {e}")
            raise RecognitionError(f"Failed to get history: {str(e)}")
    
    async def delete_recognition_history(self, recognition_id: str) -> None:
        """Delete recognition from history"""
        try:
            await self.database_service.delete_recognition_history(recognition_id)
        except Exception as e:
            logger.error(f"Failed to delete recognition history: {e}")
            raise RecognitionError(f"Failed to delete history: {str(e)}")
    
    async def log_recognition(
        self,
        song_id: UUID,
        confidence: float,
        processing_time_ms: int,
        request_id: Optional[str] = None
    ) -> None:
        """Log recognition result to database"""
        try:
            await self.database_service.log_recognition(
                song_id=song_id,
                confidence=confidence,
                processing_time_ms=processing_time_ms,
                request_id=request_id
            )
        except Exception as e:
            logger.error(f"Failed to log recognition: {e}")
    
    async def log_error(
        self,
        error: str,
        processing_time_ms: int,
        request_id: Optional[str] = None
    ) -> None:
        """Log recognition error to database"""
        try:
            await self.database_service.log_recognition_error(
                error=error,
                processing_time_ms=processing_time_ms,
                request_id=request_id
            )
        except Exception as e:
            logger.error(f"Failed to log error: {e}")
    
    async def get_recognition_stats(self) -> Dict[str, Any]:
        """Get recognition statistics"""
        try:
            stats = await self.database_service.get_recognition_stats()
            
            # Add vector database stats
            vector_stats = await self.vector_service.get_index_stats()
            stats["vector_database"] = {
                "total_vectors": vector_stats.get("total_vector_count", 0),
                "dimensions": vector_stats.get("dimension", 0),
                "index_fullness": vector_stats.get("index_fullness", 0.0)
            }
            
            return stats
        except Exception as e:
            logger.error(f"Failed to get recognition stats: {e}")
            raise RecognitionError(f"Failed to get stats: {str(e)}")
    
    async def health_check(self) -> Dict[str, bool]:
        """Check service health"""
        try:
            return {
                "audio_service": await self.audio_service.health_check(),
                "database_service": await self.database_service.health_check(),
                "vector_service": await self.vector_service.health_check()
            }
        except Exception as e:
            logger.error(f"Health check failed: {e}")
            return {
                "audio_service": False,
                "database_service": False,
                "vector_service": False
            }


# Dependency injection
async def get_recognition_service() -> RecognitionService:
    """Get recognition service instance with dependencies"""
    from app.services.audio_service import get_audio_service
    from app.services.database_service import get_database_service
    
    audio_service = await get_audio_service()
    database_service = await get_database_service()
    vector_service = await get_vector_service()
    
    return RecognitionService(
        audio_service=audio_service,
        database_service=database_service,
        vector_service=vector_service
    )
