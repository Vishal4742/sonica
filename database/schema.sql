-- SONICA Audio Recognition Database Schema
-- PostgreSQL database schema for the SONICA audio recognition system
-- Optimized for audio fingerprinting and song recognition

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Songs table: Stores metadata for all songs in the system
-- This is the main catalog of songs that can be recognized
CREATE TABLE songs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title VARCHAR(255) NOT NULL,
    artist VARCHAR(255) NOT NULL,
    album VARCHAR(255),
    language VARCHAR(10) DEFAULT 'en-US',
    duration DECIMAL(10,3) NOT NULL, -- Duration in seconds
    r2_audio_url TEXT, -- Cloudflare R2 URL for audio file
    r2_thumbnail_url TEXT, -- Cloudflare R2 URL for album art/thumbnail
    audio_key VARCHAR(500), -- R2 object key for audio file
    thumbnail_key VARCHAR(500), -- R2 object key for thumbnail
    recognition_count INTEGER DEFAULT 0, -- Number of times this song has been recognized
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Fingerprints table: Stores audio fingerprints for fast song matching
-- Each song has multiple fingerprints at different time offsets
-- The hash column contains the audio fingerprint hash for matching
CREATE TABLE fingerprints (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    song_id UUID NOT NULL REFERENCES songs(id) ON DELETE CASCADE,
    hash VARCHAR(32) NOT NULL, -- Audio fingerprint hash (MD5 or similar)
    time_offset DECIMAL(10,3) NOT NULL, -- Time offset in seconds where this fingerprint was taken
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Recognitions table: Logs all successful song recognitions
-- Tracks when songs are identified, confidence levels, and user information
CREATE TABLE recognitions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    song_id UUID NOT NULL REFERENCES songs(id) ON DELETE CASCADE,
    recognized_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    confidence DECIMAL(3,2) NOT NULL, -- Recognition confidence (0.0 to 1.0)
    user_ip INET, -- IP address of the user who requested recognition
    processing_time_ms INTEGER, -- Time taken to process the recognition in milliseconds
    audio_duration DECIMAL(10,3), -- Duration of the audio sample that was recognized
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for optimal performance
-- Hash index for fast fingerprint lookups (most critical for recognition speed)
CREATE INDEX idx_fingerprints_hash ON fingerprints USING hash (hash);

-- B-tree indexes for efficient queries
CREATE INDEX idx_songs_artist ON songs(artist);
CREATE INDEX idx_songs_title ON songs(title);
CREATE INDEX idx_songs_album ON songs(album);
CREATE INDEX idx_songs_language ON songs(language);
CREATE INDEX idx_songs_recognition_count ON songs(recognition_count);
CREATE INDEX idx_songs_created_at ON songs(created_at);

-- Fingerprints table indexes
CREATE INDEX idx_fingerprints_song_id ON fingerprints(song_id);
CREATE INDEX idx_fingerprints_time_offset ON fingerprints(time_offset);
CREATE INDEX idx_fingerprints_created_at ON fingerprints(created_at);

-- Recognitions table indexes
CREATE INDEX idx_recognitions_song_id ON recognitions(song_id);
CREATE INDEX idx_recognitions_recognized_at ON recognitions(recognized_at);
CREATE INDEX idx_recognitions_confidence ON recognitions(confidence);
CREATE INDEX idx_recognitions_user_ip ON recognitions(user_ip);
CREATE INDEX idx_recognitions_processing_time ON recognitions(processing_time_ms);
CREATE INDEX idx_recognitions_created_at ON recognitions(created_at);

-- Composite indexes for common query patterns
CREATE INDEX idx_fingerprints_hash_song_id ON fingerprints(hash, song_id);
CREATE INDEX idx_recognitions_song_recognized_at ON recognitions(song_id, recognized_at);
CREATE INDEX idx_songs_artist_title ON songs(artist, title);

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create trigger for songs table updated_at column
CREATE TRIGGER update_songs_updated_at BEFORE UPDATE ON songs
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Create function to increment recognition count
CREATE OR REPLACE FUNCTION increment_recognition_count()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE songs 
    SET recognition_count = recognition_count + 1,
        updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.song_id;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create trigger to automatically increment recognition count
CREATE TRIGGER increment_song_recognition_count 
    AFTER INSERT ON recognitions
    FOR EACH ROW EXECUTE FUNCTION increment_recognition_count();

-- Create function to get song statistics
CREATE OR REPLACE FUNCTION get_song_stats(song_uuid UUID)
RETURNS TABLE (
    total_recognitions BIGINT,
    avg_confidence DECIMAL,
    avg_processing_time DECIMAL,
    first_recognized TIMESTAMP WITH TIME ZONE,
    last_recognized TIMESTAMP WITH TIME ZONE
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        COUNT(*) as total_recognitions,
        AVG(r.confidence) as avg_confidence,
        AVG(r.processing_time_ms) as avg_processing_time,
        MIN(r.recognized_at) as first_recognized,
        MAX(r.recognized_at) as last_recognized
    FROM recognitions r
    WHERE r.song_id = song_uuid;
END;
$$ language 'plpgsql';

-- Create view for song recognition summary
CREATE VIEW song_recognition_summary AS
SELECT 
    s.id,
    s.title,
    s.artist,
    s.album,
    s.recognition_count,
    COUNT(r.id) as actual_recognitions,
    AVG(r.confidence) as avg_confidence,
    AVG(r.processing_time_ms) as avg_processing_time,
    MIN(r.recognized_at) as first_recognized,
    MAX(r.recognized_at) as last_recognized,
    s.created_at,
    s.updated_at
FROM songs s
LEFT JOIN recognitions r ON s.id = r.song_id
GROUP BY s.id, s.title, s.artist, s.album, s.recognition_count, s.created_at, s.updated_at;
