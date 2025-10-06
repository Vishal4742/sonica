"""
Vector database service for ultra-fast similarity search
"""

import asyncio
import logging
from typing import List, Optional, Dict, Any
import httpx
import json
from uuid import UUID
from app.core.config import settings
from app.core.exceptions import VectorDatabaseError

logger = logging.getLogger(__name__)


class VectorDatabaseService:
    """Service for managing vector database operations"""
    
    def __init__(self):
        self.api_key = settings.VECTOR_DB_API_KEY
        self.environment = settings.VECTOR_DB_ENVIRONMENT
        self.index_name = settings.VECTOR_DB_INDEX_NAME
        self.dimensions = settings.VECTOR_DB_DIMENSIONS
        self.base_url = f"https://{self.index_name}-{self.environment}.svc.pinecone.io"
        
        self.client = httpx.AsyncClient(
            timeout=30.0,
            headers={
                "Api-Key": self.api_key,
                "Content-Type": "application/json"
            }
        )
    
    async def initialize(self) -> None:
        """Initialize vector database connection"""
        try:
            stats = await self.get_index_stats()
            logger.info(
                f"Vector database initialized: {stats['total_vector_count']} vectors, "
                f"{stats['dimension']} dimensions"
            )
        except Exception as e:
            logger.error(f"Failed to initialize vector database: {e}")
            raise VectorDatabaseError(f"Vector database initialization failed: {str(e)}")
    
    async def get_index_stats(self) -> Dict[str, Any]:
        """Get index statistics"""
        try:
            response = await self.client.post(f"{self.base_url}/describe_index_stats")
            response.raise_for_status()
            return response.json()
        except httpx.HTTPError as e:
            raise VectorDatabaseError(f"Failed to get index stats: {str(e)}")
    
    async def upsert_vectors(
        self,
        vectors: List[Dict[str, Any]],
        namespace: Optional[str] = None
    ) -> None:
        """Upsert vectors to the database"""
        try:
            payload = {
                "vectors": vectors,
                "namespace": namespace
            }
            
            response = await self.client.post(
                f"{self.base_url}/vectors/upsert",
                json=payload
            )
            response.raise_for_status()
            
            logger.info(f"Successfully upserted {len(vectors)} vectors")
            
        except httpx.HTTPError as e:
            raise VectorDatabaseError(f"Failed to upsert vectors: {str(e)}")
    
    async def query_similar(
        self,
        query_vector: List[float],
        top_k: int = 10,
        namespace: Optional[str] = None,
        filter: Optional[Dict[str, Any]] = None
    ) -> List[Dict[str, Any]]:
        """Query similar vectors"""
        try:
            payload = {
                "vector": query_vector,
                "top_k": top_k,
                "include_metadata": True,
                "namespace": namespace,
                "filter": filter
            }
            
            response = await self.client.post(
                f"{self.base_url}/query",
                json=payload
            )
            response.raise_for_status()
            
            result = response.json()
            matches = result.get("matches", [])
            
            logger.debug(f"Found {len(matches)} similar vectors")
            return matches
            
        except httpx.HTTPError as e:
            raise VectorDatabaseError(f"Failed to query vectors: {str(e)}")
    
    async def delete_vectors(
        self,
        ids: List[str],
        namespace: Optional[str] = None
    ) -> None:
        """Delete vectors by IDs"""
        try:
            payload = {
                "ids": ids,
                "namespace": namespace
            }
            
            response = await self.client.post(
                f"{self.base_url}/vectors/delete",
                json=payload
            )
            response.raise_for_status()
            
            logger.info(f"Successfully deleted {len(ids)} vectors")
            
        except httpx.HTTPError as e:
            raise VectorDatabaseError(f"Failed to delete vectors: {str(e)}")
    
    def fingerprint_to_vector(self, fingerprint_data: Dict[str, Any]) -> List[float]:
        """Convert fingerprint to vector representation"""
        vector = []
        
        # Extract peaks from fingerprint
        peaks = fingerprint_data.get("peaks", [])
        
        if peaks:
            # Frequency distribution (20 bins)
            freq_bins = 20
            freq_histogram = [0.0] * freq_bins
            
            for peak in peaks:
                frequency = peak.get("frequency", 0.0)
                magnitude = peak.get("magnitude", 0.0)
                
                bin_idx = min(int((frequency / 20000.0) * freq_bins), freq_bins - 1)
                freq_histogram[bin_idx] += magnitude
            
            # Normalize frequency histogram
            max_freq = max(freq_histogram) if freq_histogram else 1.0
            if max_freq > 0:
                freq_histogram = [val / max_freq for val in freq_histogram]
            
            vector.extend(freq_histogram)
        
        # Time distribution (10 bins)
        time_bins = 10
        time_histogram = [0.0] * time_bins
        
        metadata = fingerprint_data.get("metadata", {})
        duration = metadata.get("duration", 1.0)
        
        for peak in peaks:
            time = peak.get("time", 0.0)
            magnitude = peak.get("magnitude", 0.0)
            
            bin_idx = min(int((time / duration) * time_bins), time_bins - 1)
            time_histogram[bin_idx] += magnitude
        
        # Normalize time histogram
        max_time = max(time_histogram) if time_histogram else 1.0
        if max_time > 0:
            time_histogram = [val / max_time for val in time_histogram]
        
        vector.extend(time_histogram)
        
        # Statistical features
        if peaks:
            magnitudes = [peak.get("magnitude", 0.0) for peak in peaks]
            mean_magnitude = sum(magnitudes) / len(magnitudes)
            max_magnitude = max(magnitudes)
            min_magnitude = min(magnitudes)
            
            vector.extend([mean_magnitude, max_magnitude, min_magnitude])
        
        # Pad or truncate to target dimensions
        while len(vector) < self.dimensions:
            vector.append(0.0)
        
        if len(vector) > self.dimensions:
            vector = vector[:self.dimensions]
        
        return vector
    
    async def add_fingerprint(
        self,
        song_id: UUID,
        fingerprint_data: Dict[str, Any],
        metadata: Dict[str, Any]
    ) -> None:
        """Add audio fingerprint to vector database"""
        try:
            # Convert fingerprint to vector
            vector = self.fingerprint_to_vector(fingerprint_data)
            
            # Prepare metadata
            full_metadata = metadata.copy()
            full_metadata["song_id"] = str(song_id)
            full_metadata["fingerprint_id"] = str(UUID.uuid4())
            full_metadata["created_at"] = asyncio.get_event_loop().time()
            
            # Prepare vector data
            vector_data = {
                "id": f"fingerprint_{song_id}",
                "values": vector,
                "metadata": full_metadata
            }
            
            # Upsert to vector database
            await self.upsert_vectors([vector_data])
            
            logger.info(f"Added fingerprint for song {song_id} to vector database")
            
        except Exception as e:
            raise VectorDatabaseError(f"Failed to add fingerprint: {str(e)}")
    
    async def search_similar_fingerprints(
        self,
        fingerprint_data: Dict[str, Any],
        top_k: int = 10,
        language_filter: Optional[str] = None,
        genre_filter: Optional[str] = None
    ) -> List[Dict[str, Any]]:
        """Search for similar fingerprints"""
        try:
            # Convert fingerprint to vector
            query_vector = self.fingerprint_to_vector(fingerprint_data)
            
            # Build filter if needed
            filter_dict = {}
            if language_filter:
                filter_dict["language"] = language_filter
            if genre_filter:
                filter_dict["genre"] = genre_filter
            
            filter_option = filter_dict if filter_dict else None
            
            # Query similar vectors
            matches = await self.query_similar(
                query_vector=query_vector,
                top_k=top_k,
                filter=filter_option
            )
            
            return matches
            
        except Exception as e:
            raise VectorDatabaseError(f"Failed to search similar fingerprints: {str(e)}")
    
    async def batch_upsert_fingerprints(
        self,
        fingerprints: List[tuple]
    ) -> None:
        """Batch upsert fingerprints"""
        try:
            batch_size = 100  # Pinecone batch limit
            
            for i in range(0, len(fingerprints), batch_size):
                batch = fingerprints[i:i + batch_size]
                
                vector_data_list = []
                for song_id, fingerprint_data, metadata in batch:
                    vector = self.fingerprint_to_vector(fingerprint_data)
                    
                    full_metadata = metadata.copy()
                    full_metadata["song_id"] = str(song_id)
                    full_metadata["fingerprint_id"] = str(UUID.uuid4())
                    full_metadata["created_at"] = asyncio.get_event_loop().time()
                    
                    vector_data = {
                        "id": f"fingerprint_{song_id}",
                        "values": vector,
                        "metadata": full_metadata
                    }
                    
                    vector_data_list.append(vector_data)
                
                await self.upsert_vectors(vector_data_list)
                
            logger.info(f"Successfully batch upserted {len(fingerprints)} fingerprints")
            
        except Exception as e:
            raise VectorDatabaseError(f"Failed to batch upsert fingerprints: {str(e)}")
    
    async def health_check(self) -> bool:
        """Check vector database health"""
        try:
            await self.get_index_stats()
            return True
        except Exception:
            return False
    
    async def close(self) -> None:
        """Close the HTTP client"""
        await self.client.aclose()


# Global vector database service instance
_vector_service: Optional[VectorDatabaseService] = None


async def get_vector_service() -> VectorDatabaseService:
    """Get or create vector database service instance"""
    global _vector_service
    
    if _vector_service is None:
        _vector_service = VectorDatabaseService()
        await _vector_service.initialize()
    
    return _vector_service


async def close_vector_service() -> None:
    """Close vector database service"""
    global _vector_service
    
    if _vector_service:
        await _vector_service.close()
        _vector_service = None
