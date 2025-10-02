-- SONICA Audio Recognition Database Schema
-- PostgreSQL database schema for the SONICA audio recognition system

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) UNIQUE NOT NULL,
    username VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    avatar_url TEXT,
    is_active BOOLEAN DEFAULT true,
    is_verified BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- User sessions table
CREATE TABLE user_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    last_accessed_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Audio files table
CREATE TABLE audio_files (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    filename VARCHAR(255) NOT NULL,
    original_filename VARCHAR(255) NOT NULL,
    file_path TEXT NOT NULL,
    file_size BIGINT NOT NULL,
    duration_seconds DECIMAL(10,3),
    sample_rate INTEGER,
    channels INTEGER,
    bit_depth INTEGER,
    format VARCHAR(50) NOT NULL, -- wav, mp3, flac, etc.
    mime_type VARCHAR(100) NOT NULL,
    uploaded_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Audio recognition jobs table
CREATE TABLE recognition_jobs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    audio_file_id UUID NOT NULL REFERENCES audio_files(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- pending, processing, completed, failed
    language VARCHAR(10) DEFAULT 'en-US',
    model_name VARCHAR(100),
    confidence_threshold DECIMAL(3,2) DEFAULT 0.5,
    processing_started_at TIMESTAMP WITH TIME ZONE,
    processing_completed_at TIMESTAMP WITH TIME ZONE,
    error_message TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Recognition results table
CREATE TABLE recognition_results (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    job_id UUID NOT NULL REFERENCES recognition_jobs(id) ON DELETE CASCADE,
    text TEXT NOT NULL,
    confidence DECIMAL(3,2) NOT NULL,
    start_time DECIMAL(10,3) NOT NULL,
    end_time DECIMAL(10,3) NOT NULL,
    speaker_id VARCHAR(100),
    language VARCHAR(10),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Audio features table (for ML model training/analysis)
CREATE TABLE audio_features (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    audio_file_id UUID NOT NULL REFERENCES audio_files(id) ON DELETE CASCADE,
    feature_type VARCHAR(50) NOT NULL, -- mfcc, spectral_centroid, zero_crossing_rate, etc.
    feature_vector JSONB NOT NULL,
    extraction_method VARCHAR(100),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Model configurations table
CREATE TABLE model_configs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) UNIQUE NOT NULL,
    model_type VARCHAR(50) NOT NULL, -- speech_to_text, speaker_identification, emotion_detection
    model_path TEXT NOT NULL,
    version VARCHAR(20) NOT NULL,
    language VARCHAR(10),
    is_active BOOLEAN DEFAULT true,
    accuracy_score DECIMAL(3,2),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Audio processing logs table
CREATE TABLE processing_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    job_id UUID REFERENCES recognition_jobs(id) ON DELETE CASCADE,
    audio_file_id UUID REFERENCES audio_files(id) ON DELETE CASCADE,
    log_level VARCHAR(20) NOT NULL, -- info, warning, error, debug
    message TEXT NOT NULL,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for better performance
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX idx_user_sessions_token_hash ON user_sessions(token_hash);
CREATE INDEX idx_audio_files_uploaded_by ON audio_files(uploaded_by);
CREATE INDEX idx_audio_files_format ON audio_files(format);
CREATE INDEX idx_recognition_jobs_audio_file_id ON recognition_jobs(audio_file_id);
CREATE INDEX idx_recognition_jobs_user_id ON recognition_jobs(user_id);
CREATE INDEX idx_recognition_jobs_status ON recognition_jobs(status);
CREATE INDEX idx_recognition_jobs_created_at ON recognition_jobs(created_at);
CREATE INDEX idx_recognition_results_job_id ON recognition_results(job_id);
CREATE INDEX idx_recognition_results_confidence ON recognition_results(confidence);
CREATE INDEX idx_audio_features_audio_file_id ON audio_features(audio_file_id);
CREATE INDEX idx_audio_features_feature_type ON audio_features(feature_type);
CREATE INDEX idx_model_configs_model_type ON model_configs(model_type);
CREATE INDEX idx_model_configs_is_active ON model_configs(is_active);
CREATE INDEX idx_processing_logs_job_id ON processing_logs(job_id);
CREATE INDEX idx_processing_logs_audio_file_id ON processing_logs(audio_file_id);
CREATE INDEX idx_processing_logs_log_level ON processing_logs(log_level);
CREATE INDEX idx_processing_logs_created_at ON processing_logs(created_at);

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers for updated_at columns
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_recognition_jobs_updated_at BEFORE UPDATE ON recognition_jobs
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_model_configs_updated_at BEFORE UPDATE ON model_configs
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
