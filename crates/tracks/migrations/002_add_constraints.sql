-- Add foreign key constraints and indexes

-- Add foreign key constraint to activities
ALTER TABLE activities
    ADD CONSTRAINT fk_activities_user
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

-- Add foreign key constraints to tracks
ALTER TABLE tracks
    ADD CONSTRAINT fk_tracks_user
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    ADD CONSTRAINT fk_tracks_activity
    FOREIGN KEY (activity_id) REFERENCES activities(id) ON DELETE CASCADE;

-- Add foreign key constraints to scores
ALTER TABLE scores
    ADD CONSTRAINT fk_scores_user
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    ADD CONSTRAINT fk_scores_activity
    FOREIGN KEY (activity_id) REFERENCES activities(id) ON DELETE CASCADE;

-- Add spatial index for tracks (for segment matching)
CREATE INDEX IF NOT EXISTS idx_tracks_geo_gist ON tracks USING GIST (geo);

-- Add additional user profile fields
ALTER TABLE users ADD COLUMN IF NOT EXISTS password_hash TEXT;
ALTER TABLE users ADD COLUMN IF NOT EXISTS auth_provider TEXT DEFAULT 'email';
ALTER TABLE users ADD COLUMN IF NOT EXISTS avatar_url TEXT;
ALTER TABLE users ADD COLUMN IF NOT EXISTS bio TEXT;
ALTER TABLE users ADD COLUMN IF NOT EXISTS updated_at TIMESTAMP WITH TIME ZONE;

-- Add additional activity fields
ALTER TABLE activities ADD COLUMN IF NOT EXISTS description TEXT;
ALTER TABLE activities ADD COLUMN IF NOT EXISTS visibility TEXT DEFAULT 'public';
ALTER TABLE activities ADD COLUMN IF NOT EXISTS started_at TIMESTAMP WITH TIME ZONE;
ALTER TABLE activities ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMP WITH TIME ZONE;

-- Add indexes for common queries
CREATE INDEX IF NOT EXISTS idx_activities_visibility ON activities(visibility) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_activities_user_date ON activities(user_id, submitted_at DESC) WHERE deleted_at IS NULL;
