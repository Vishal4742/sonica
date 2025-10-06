-- Initial database schema for Sonica audio engine

-- Create songs table
CREATE TABLE IF NOT EXISTS songs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    artist VARCHAR(255) NOT NULL,
    album VARCHAR(255),
    language VARCHAR(50) NOT NULL DEFAULT 'en',
    genre VARCHAR(100),
    duration INTEGER, -- Duration in seconds
    release_year INTEGER,
    audio_url TEXT,
    artwork_url TEXT,
    popularity_score FLOAT DEFAULT 0.0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create fingerprints table
CREATE TABLE IF NOT EXISTS fingerprints (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    song_id UUID NOT NULL REFERENCES songs(id) ON DELETE CASCADE,
    fingerprint_data BYTEA NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create recognition logs table
CREATE TABLE IF NOT EXISTS recognition_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    song_id UUID REFERENCES songs(id) ON DELETE SET NULL,
    confidence FLOAT NOT NULL,
    processing_time_ms BIGINT NOT NULL,
    audio_duration FLOAT,
    user_id UUID,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create users table (for future user management)
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE,
    name VARCHAR(255),
    preferences JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create user favorites table
CREATE TABLE IF NOT EXISTS user_favorites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    song_id UUID NOT NULL REFERENCES songs(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(user_id, song_id)
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_songs_language ON songs(language);
CREATE INDEX IF NOT EXISTS idx_songs_artist ON songs(artist);
CREATE INDEX IF NOT EXISTS idx_songs_genre ON songs(genre);
CREATE INDEX IF NOT EXISTS idx_songs_popularity ON songs(popularity_score DESC);
CREATE INDEX IF NOT EXISTS idx_songs_created_at ON songs(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_fingerprints_song_id ON fingerprints(song_id);
CREATE INDEX IF NOT EXISTS idx_fingerprints_created_at ON fingerprints(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_recognition_logs_song_id ON recognition_logs(song_id);
CREATE INDEX IF NOT EXISTS idx_recognition_logs_created_at ON recognition_logs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_recognition_logs_user_id ON recognition_logs(user_id);

CREATE INDEX IF NOT EXISTS idx_user_favorites_user_id ON user_favorites(user_id);
CREATE INDEX IF NOT EXISTS idx_user_favorites_song_id ON user_favorites(song_id);

-- Create full-text search index
CREATE INDEX IF NOT EXISTS idx_songs_search ON songs USING gin(to_tsvector('english', title || ' ' || artist || ' ' || COALESCE(album, '')));

-- Create function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers for updated_at
CREATE TRIGGER update_songs_updated_at BEFORE UPDATE ON songs
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Create materialized view for popular songs by language
CREATE MATERIALIZED VIEW IF NOT EXISTS popular_songs_by_language AS
SELECT 
    language,
    id,
    title,
    artist,
    album,
    genre,
    duration,
    release_year,
    popularity_score,
    ROW_NUMBER() OVER (PARTITION BY language ORDER BY popularity_score DESC) as rank
FROM songs
WHERE popularity_score > 0.1;

CREATE UNIQUE INDEX IF NOT EXISTS idx_popular_songs_by_language ON popular_songs_by_language(language, rank);

-- Create function to refresh materialized view
CREATE OR REPLACE FUNCTION refresh_popular_songs()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY popular_songs_by_language;
END;
$$ LANGUAGE plpgsql;

-- Insert some sample data for testing
INSERT INTO songs (id, title, artist, album, language, genre, duration, release_year, popularity_score) VALUES
    ('550e8400-e29b-41d4-a716-446655440000', 'Tum Hi Ho', 'Arijit Singh', 'Aashiqui 2', 'hi', 'bollywood', 240, 2013, 0.95),
    ('550e8400-e29b-41d4-a716-446655440001', 'Chahun Main Ya Naa', 'Arijit Singh', 'Aashiqui 2', 'hi', 'bollywood', 280, 2013, 0.90),
    ('550e8400-e29b-41d4-a716-446655440002', 'Sun Raha Hai Na Tu', 'Ankit Tiwari', 'Aashiqui 2', 'hi', 'bollywood', 260, 2013, 0.88),
    ('550e8400-e29b-41d4-a716-446655440003', 'Shape of You', 'Ed Sheeran', '÷ (Divide)', 'en', 'pop', 233, 2017, 0.92),
    ('550e8400-e29b-41d4-a716-446655440004', 'Despacito', 'Luis Fonsi', 'Despacito', 'es', 'reggaeton', 281, 2017, 0.89)
ON CONFLICT (id) DO NOTHING;
