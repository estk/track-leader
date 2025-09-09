-- Create activity_type enum
CREATE TYPE activity_type AS ENUM ('running', 'cycling', 'walking', 'hiking', 'other');

-- Create activities table
CREATE TABLE activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    activity_type activity_type NOT NULL,
    filename TEXT NOT NULL,
    object_store_path TEXT NOT NULL,
    total_distance DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_ascent DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_descent DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_time BIGINT NOT NULL DEFAULT 0,
    submitted_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create track_points table
CREATE TABLE track_points (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    activity_id UUID NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    latitude DOUBLE PRECISION NOT NULL,
    longitude DOUBLE PRECISION NOT NULL,
    elevation DOUBLE PRECISION,
    time TIMESTAMP WITH TIME ZONE,
    sequence INTEGER NOT NULL
);

-- Create indexes for better query performance
CREATE INDEX idx_activities_user_id ON activities(user_id);
CREATE INDEX idx_activities_submitted_at ON activities(submitted_at);
CREATE INDEX idx_track_points_activity_id ON track_points(activity_id);
CREATE INDEX idx_track_points_sequence ON track_points(activity_id, sequence);
