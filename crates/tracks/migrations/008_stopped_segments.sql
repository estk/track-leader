-- Migration: 008_stopped_segments
-- Add tables for stopped segment detection and dig tagging

-- Table for auto-detected stopped segments during activity upload
CREATE TABLE IF NOT EXISTS activity_stopped_segments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    activity_id UUID NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    duration_seconds DOUBLE PRECISION NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Table for user-tagged dig segments (subset of stopped segments marked as trail work)
CREATE TABLE IF NOT EXISTS activity_dig_segments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    activity_id UUID NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    duration_seconds DOUBLE PRECISION NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_activity_stopped_segments_activity_id
    ON activity_stopped_segments(activity_id);
CREATE INDEX IF NOT EXISTS idx_activity_dig_segments_activity_id
    ON activity_dig_segments(activity_id);
