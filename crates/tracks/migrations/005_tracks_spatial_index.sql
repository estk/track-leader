-- Add GIST spatial index on tracks.geo for efficient spatial queries
CREATE INDEX IF NOT EXISTS idx_tracks_geo_gist ON tracks USING GIST (geo);

-- Add unique constraint on activity_id to ensure one track per activity
-- Drop existing index first, then recreate as unique
DROP INDEX IF EXISTS idx_tracks_activities_id;
CREATE UNIQUE INDEX idx_tracks_activity_id ON tracks(activity_id);
