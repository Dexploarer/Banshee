-- Initial schema for Banshee Memory Plugin
-- PostgreSQL database schema for emotional intelligence and conversation storage

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create enum for emotion types (based on OCC model)
CREATE TYPE emotion_type AS ENUM (
    'Joy', 'Distress', 'Hope', 'Fear', 'Satisfaction', 'Disappointment', 
    'Relief', 'FearConfirmed', 'Pride', 'Shame', 'Admiration', 'Reproach',
    'Gratification', 'Remorse', 'Gratitude', 'Anger', 'Love', 'Hate',
    'HappyFor', 'Resentment', 'Gloating', 'Pity'
);

-- Create enum for conversation roles
CREATE TYPE conversation_role AS ENUM ('user', 'assistant', 'system', 'function');

-- Create enum for memory types
CREATE TYPE memory_type AS ENUM (
    'goal', 'preference', 'personality', 'skill', 'fact', 'knowledge',
    'experience', 'relationship', 'context', 'temp', 'cache'
);

-- Function to update timestamp on row updates
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Emotional States Table
-- Stores current emotional state snapshots for each agent
CREATE TABLE emotional_states (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    agent_id UUID NOT NULL,
    emotions JSONB NOT NULL, -- HashMap<Emotion, EmotionalIntensity>
    decay_rates JSONB NOT NULL, -- HashMap<Emotion, f32>
    overall_valence REAL NOT NULL, -- -1.0 to 1.0
    overall_arousal REAL NOT NULL, -- 0.0 to 1.0
    dominant_emotion TEXT, -- Name of strongest emotion
    dominant_intensity REAL, -- Intensity of strongest emotion
    emotional_temperature REAL DEFAULT 0.0, -- How volatile the agent is feeling
    is_frustrated BOOLEAN DEFAULT FALSE, -- High anger + distress
    is_confident BOOLEAN DEFAULT FALSE, -- High pride, low shame
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT emotional_states_agent_unique UNIQUE (agent_id),
    CONSTRAINT emotional_states_valence_check CHECK (overall_valence >= -1.0 AND overall_valence <= 1.0),
    CONSTRAINT emotional_states_arousal_check CHECK (overall_arousal >= 0.0 AND overall_arousal <= 1.0),
    CONSTRAINT emotional_states_intensity_check CHECK (dominant_intensity IS NULL OR (dominant_intensity >= 0.0 AND dominant_intensity <= 1.0))
);

-- Emotional Events Table
-- Historical log of all emotional events and their impacts
CREATE TABLE emotional_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    agent_id UUID NOT NULL,
    event_type TEXT NOT NULL, -- 'task_completed', 'user_feedback', etc.
    event_data JSONB NOT NULL, -- Full event details
    resulting_emotions JSONB NOT NULL, -- HashMap<Emotion, EmotionalIntensity>
    context_factors JSONB, -- Additional context that influenced the appraisal
    intensity_delta REAL NOT NULL, -- Overall change in emotional intensity
    valence_delta REAL NOT NULL, -- Change in emotional valence
    arousal_delta REAL NOT NULL, -- Change in emotional arousal
    appraisal_duration_ms INTEGER DEFAULT 0, -- Time taken to process the event
    personality_influence JSONB, -- How personality factors affected the response
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT emotional_events_intensity_delta_check CHECK (intensity_delta >= -10.0 AND intensity_delta <= 10.0),
    CONSTRAINT emotional_events_valence_delta_check CHECK (valence_delta >= -2.0 AND valence_delta <= 2.0),
    CONSTRAINT emotional_events_arousal_delta_check CHECK (arousal_delta >= -2.0 AND arousal_delta <= 2.0)
);

-- Conversation Messages Table
-- All conversation history with emotional context
CREATE TABLE conversation_messages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    agent_id UUID NOT NULL,
    role conversation_role NOT NULL,
    content TEXT NOT NULL,
    metadata JSONB, -- Additional message metadata (source, tool calls, etc.)
    emotional_context JSONB, -- Emotional state when message was sent/received
    tokens_used INTEGER, -- Token count if applicable
    content_length INTEGER GENERATED ALWAYS AS (length(content)) STORED,
    word_count INTEGER GENERATED ALWAYS AS (array_length(string_to_array(trim(content), ' '), 1)) STORED,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT conversation_messages_tokens_check CHECK (tokens_used IS NULL OR tokens_used >= 0),
    CONSTRAINT conversation_messages_content_check CHECK (length(trim(content)) > 0)
);

-- Memory Records Table
-- Persistent storage for agent knowledge, preferences, and temporary data
CREATE TABLE memory_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    agent_id UUID NOT NULL,
    memory_type TEXT NOT NULL,
    key TEXT NOT NULL,
    data JSONB NOT NULL,
    importance REAL NOT NULL DEFAULT 0.5, -- 0.0 to 1.0 for prioritization
    access_count INTEGER NOT NULL DEFAULT 0,
    last_accessed TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ, -- TTL for temporary memories
    tags TEXT[] DEFAULT '{}', -- Searchable tags
    
    CONSTRAINT memory_records_unique_key UNIQUE (agent_id, memory_type, key),
    CONSTRAINT memory_records_importance_check CHECK (importance >= 0.0 AND importance <= 1.0),
    CONSTRAINT memory_records_access_count_check CHECK (access_count >= 0)
);

-- Agent Sessions Table
-- Track agent activity sessions for analytics
CREATE TABLE agent_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    agent_id UUID NOT NULL,
    session_start TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    session_end TIMESTAMPTZ,
    initial_emotional_state JSONB,
    final_emotional_state JSONB,
    total_messages INTEGER DEFAULT 0,
    total_events INTEGER DEFAULT 0,
    total_tokens_used INTEGER DEFAULT 0,
    session_metadata JSONB, -- Context about the session
    
    CONSTRAINT agent_sessions_end_check CHECK (session_end IS NULL OR session_end >= session_start),
    CONSTRAINT agent_sessions_messages_check CHECK (total_messages >= 0),
    CONSTRAINT agent_sessions_events_check CHECK (total_events >= 0),
    CONSTRAINT agent_sessions_tokens_check CHECK (total_tokens_used >= 0)
);

-- Performance Indexes
-- Emotional States
CREATE INDEX idx_emotional_states_agent_id ON emotional_states(agent_id);
CREATE INDEX idx_emotional_states_updated_at ON emotional_states(updated_at);
CREATE INDEX idx_emotional_states_valence ON emotional_states(overall_valence);
CREATE INDEX idx_emotional_states_arousal ON emotional_states(overall_arousal);
CREATE INDEX idx_emotional_states_dominant ON emotional_states(dominant_emotion, dominant_intensity);

-- Emotional Events
CREATE INDEX idx_emotional_events_agent_id ON emotional_events(agent_id);
CREATE INDEX idx_emotional_events_created_at ON emotional_events(created_at);
CREATE INDEX idx_emotional_events_type ON emotional_events(event_type);
CREATE INDEX idx_emotional_events_valence_delta ON emotional_events(valence_delta);
CREATE INDEX idx_emotional_events_compound ON emotional_events(agent_id, event_type, created_at);

-- Conversation Messages
CREATE INDEX idx_conversation_messages_agent_id ON conversation_messages(agent_id);
CREATE INDEX idx_conversation_messages_created_at ON conversation_messages(created_at);
CREATE INDEX idx_conversation_messages_role ON conversation_messages(role);
CREATE INDEX idx_conversation_messages_tokens ON conversation_messages(tokens_used);
CREATE INDEX idx_conversation_messages_content_search ON conversation_messages USING gin(to_tsvector('english', content));
CREATE INDEX idx_conversation_messages_compound ON conversation_messages(agent_id, role, created_at);

-- Memory Records
CREATE INDEX idx_memory_records_agent_id ON memory_records(agent_id);
CREATE INDEX idx_memory_records_type ON memory_records(memory_type);
CREATE INDEX idx_memory_records_importance ON memory_records(importance DESC);
CREATE INDEX idx_memory_records_expires_at ON memory_records(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX idx_memory_records_last_accessed ON memory_records(last_accessed);
CREATE INDEX idx_memory_records_tags ON memory_records USING gin(tags);
CREATE INDEX idx_memory_records_compound ON memory_records(agent_id, memory_type, importance DESC);

-- Agent Sessions
CREATE INDEX idx_agent_sessions_agent_id ON agent_sessions(agent_id);
CREATE INDEX idx_agent_sessions_start ON agent_sessions(session_start);
CREATE INDEX idx_agent_sessions_end ON agent_sessions(session_end) WHERE session_end IS NOT NULL;

-- Full-text search indexes
CREATE INDEX idx_memory_records_data_search ON memory_records USING gin(to_tsvector('english', data::text));

-- Triggers for automatic timestamp updates
CREATE TRIGGER update_emotional_states_updated_at
    BEFORE UPDATE ON emotional_states
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_memory_records_updated_at
    BEFORE UPDATE ON memory_records
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Partitioning for large tables (optional, for high-volume deployments)
-- Partition emotional_events by month
-- CREATE TABLE emotional_events_template (LIKE emotional_events INCLUDING ALL);
-- SELECT create_distributed_table('emotional_events', 'agent_id');

-- Views for common queries
CREATE VIEW agent_emotional_summary AS
SELECT 
    agent_id,
    overall_valence,
    overall_arousal,
    dominant_emotion,
    dominant_intensity,
    emotional_temperature,
    is_frustrated,
    is_confident,
    updated_at
FROM emotional_states;

CREATE VIEW recent_emotional_events AS
SELECT 
    agent_id,
    event_type,
    valence_delta,
    arousal_delta,
    created_at
FROM emotional_events
WHERE created_at > NOW() - INTERVAL '24 hours'
ORDER BY created_at DESC;

CREATE VIEW conversation_stats AS
SELECT 
    agent_id,
    COUNT(*) as total_messages,
    COUNT(CASE WHEN role = 'user' THEN 1 END) as user_messages,
    COUNT(CASE WHEN role = 'assistant' THEN 1 END) as assistant_messages,
    SUM(COALESCE(tokens_used, 0)) as total_tokens,
    AVG(content_length) as avg_message_length,
    MIN(created_at) as first_message,
    MAX(created_at) as last_message
FROM conversation_messages
GROUP BY agent_id;

-- Functions for common operations
CREATE OR REPLACE FUNCTION get_agent_emotional_state(agent_uuid UUID)
RETURNS TABLE(
    valence REAL,
    arousal REAL,
    dominant_emotion TEXT,
    intensity REAL,
    last_updated TIMESTAMPTZ
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        es.overall_valence,
        es.overall_arousal,
        es.dominant_emotion,
        es.dominant_intensity,
        es.updated_at
    FROM emotional_states es
    WHERE es.agent_id = agent_uuid;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION cleanup_expired_memories()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM memory_records 
    WHERE expires_at IS NOT NULL AND expires_at < NOW();
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Grant permissions (adjust as needed for your deployment)
-- GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO banshee_app;
-- GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO banshee_app;

-- Sample data for testing (uncomment for development)
/*
INSERT INTO emotional_states (agent_id, emotions, decay_rates, overall_valence, overall_arousal) VALUES
(uuid_generate_v4(), '{"Joy": 0.8, "Pride": 0.6}', '{"Joy": 0.05, "Pride": 0.03}', 0.7, 0.75);

INSERT INTO memory_records (agent_id, memory_type, key, data, importance) VALUES
(uuid_generate_v4(), 'goal', 'primary_objective', '{"goal": "Complete all tasks efficiently", "priority": "high"}', 0.9);
*/