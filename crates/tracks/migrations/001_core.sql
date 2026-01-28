-- Core foundation: users, activities, tracks, scores

-- PostGIS Extension
CREATE EXTENSION IF NOT EXISTS postgis;

-- Enums
CREATE TYPE activity_type AS ENUM ('walking', 'running', 'hiking', 'road_cycling', 'mountain_biking', 'unknown');
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

-- Activities table
CREATE TABLE activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    activity_type activity_type NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    visibility TEXT DEFAULT 'public',
    object_store_path TEXT NOT NULL,
    submitted_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    started_at TIMESTAMP WITH TIME ZONE,
    deleted_at TIMESTAMP WITH TIME ZONE,
    kudos_count INTEGER NOT NULL DEFAULT 0,
    comment_count INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_activities_user_id ON activities(user_id);
CREATE INDEX idx_activities_submitted_at ON activities(submitted_at);

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
