"""
Main API router for Sonica v1
"""

from fastapi import APIRouter

from app.api.v1.endpoints import recognition, songs, users, search, health

api_router = APIRouter()

# Include all endpoint routers
api_router.include_router(health.router, prefix="/health", tags=["health"])
api_router.include_router(recognition.router, prefix="/recognize", tags=["recognition"])
api_router.include_router(songs.router, prefix="/songs", tags=["songs"])
api_router.include_router(search.router, prefix="/search", tags=["search"])
api_router.include_router(users.router, prefix="/users", tags=["users"])
