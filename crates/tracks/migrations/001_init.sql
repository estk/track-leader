-- Create activity_type enum
CREATE TYPE activity_type AS ENUM ('run', 'bike', 'walk', 'hike', 'mtb', 'other');

-- Create activities table
CREATE TABLE activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    activity_type activity_type NOT NULL,
    filename TEXT NOT NULL,
    object_store_path TEXT NOT NULL,
    distance DOUBLE PRECISION NOT NULL DEFAULT 0,
    ascent DOUBLE PRECISION NOT NULL DEFAULT 0,
    descent DOUBLE PRECISION NOT NULL DEFAULT 0,
    duration BIGINT NOT NULL DEFAULT 0,
    submitted_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create indexes for better query performance
CREATE INDEX idx_activities_user_id ON activities(user_id);
CREATE INDEX idx_activities_submitted_at ON activities(submitted_at);
