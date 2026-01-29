-- Core foundation: users, activities, tracks, scores

-- PostGIS Extension
CREATE EXTENSION IF NOT EXISTS postgis;

-- Enums
CREATE TYPE gender AS ENUM ('male', 'female', 'other', 'prefer_not_to_say');

-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    password_hash TEXT,
    auth_provider TEXT DEFAULT 'email',
    avatar_url TEXT,
    bio TEXT,
    gender gender,
    birth_year INTEGER,
    weight_kg FLOAT,
    country TEXT,
    region TEXT,
    follower_count INTEGER NOT NULL DEFAULT 0,
    following_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE
);

COMMENT ON COLUMN users.gender IS 'User gender for leaderboard filtering';
COMMENT ON COLUMN users.birth_year IS 'Birth year for age group filtering';
COMMENT ON COLUMN users.weight_kg IS 'Weight in kg for power calculations';
COMMENT ON COLUMN users.country IS 'Country for regional leaderboards';
COMMENT ON COLUMN users.region IS 'State/province/region for local leaderboards';

-- User demographic indexes
CREATE INDEX idx_users_gender ON users(gender) WHERE gender IS NOT NULL;
CREATE INDEX idx_users_birth_year ON users(birth_year) WHERE birth_year IS NOT NULL;
CREATE INDEX idx_users_weight_kg ON users(weight_kg) WHERE weight_kg IS NOT NULL;
CREATE INDEX idx_users_country ON users(country) WHERE country IS NOT NULL;

-- Activities table
-- Note: activity_type_id FK added after activity_types table is created in 005_activity_types.sql
CREATE TABLE activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    activity_type_id UUID NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    visibility TEXT DEFAULT 'public',
    object_store_path TEXT NOT NULL,
    submitted_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    started_at TIMESTAMP WITH TIME ZONE,
    deleted_at TIMESTAMP WITH TIME ZONE,
    kudos_count INTEGER NOT NULL DEFAULT 0,
    comment_count INTEGER NOT NULL DEFAULT 0,
    -- Multi-sport support
    type_boundaries TIMESTAMPTZ[],
    segment_types UUID[]
);

COMMENT ON COLUMN activities.visibility IS 'Visibility: public, private, or teams_only';
COMMENT ON COLUMN activities.type_boundaries IS 'Multi-sport: timestamps marking segment boundaries. First = start, last = end.';
COMMENT ON COLUMN activities.segment_types IS 'Multi-sport: activity type IDs for each segment. Length = type_boundaries.length - 1.';

CREATE INDEX idx_activities_user_id ON activities(user_id);
CREATE INDEX idx_activities_submitted_at ON activities(submitted_at);
CREATE INDEX idx_activities_visibility ON activities(visibility) WHERE deleted_at IS NULL;
CREATE INDEX idx_activities_user_date ON activities(user_id, submitted_at DESC) WHERE deleted_at IS NULL;
CREATE INDEX idx_activities_user_type_date ON activities(user_id, activity_type_id, submitted_at DESC);
CREATE INDEX idx_activities_feed ON activities(submitted_at DESC) WHERE visibility = 'public';

-- Tracks table with LineStringZM geometry
-- X = longitude, Y = latitude, Z = elevation (meters), M = timestamp (unix epoch seconds)
CREATE TABLE tracks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    activity_id UUID NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    geo GEOGRAPHY(LineStringZM, 4326) NOT NULL
);

COMMENT ON COLUMN tracks.geo IS 'Track geometry: X=lon, Y=lat, Z=elevation(m), M=timestamp(unix epoch)';

CREATE INDEX idx_tracks_user_id ON tracks(user_id);
CREATE UNIQUE INDEX idx_tracks_activity_id ON tracks(activity_id);
CREATE INDEX idx_tracks_geo ON tracks USING GIST (geo);

-- Scores table
CREATE TABLE scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    activity_id UUID NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    distance FLOAT NOT NULL,
    duration FLOAT NOT NULL,
    elevation_gain FLOAT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_scores_user_id ON scores(user_id);
CREATE INDEX idx_scores_activity_id ON scores(activity_id);

-- Sensor data arrays (parallel to track geometry points)
CREATE TABLE activity_sensor_data (
    activity_id UUID PRIMARY KEY REFERENCES activities(id) ON DELETE CASCADE,
    heart_rates int[],
    cadences int[],
    powers int[],
    temperatures double precision[]
);

COMMENT ON TABLE activity_sensor_data IS 'Sensor data arrays parallel to track geometry points';
