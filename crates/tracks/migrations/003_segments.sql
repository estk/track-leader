-- Segments table - defines a segment of trail
CREATE TABLE segments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    creator_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    activity_type activity_type NOT NULL,

    -- Geographic data
    geo GEOGRAPHY(LineString, 4326) NOT NULL,
    start_point GEOGRAPHY(Point, 4326) NOT NULL,
    end_point GEOGRAPHY(Point, 4326) NOT NULL,

    -- Computed stats
    distance_meters FLOAT NOT NULL,
    elevation_gain_meters FLOAT,
    elevation_loss_meters FLOAT,

    -- Metadata
    visibility TEXT NOT NULL DEFAULT 'public',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE,
    deleted_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_segments_creator ON segments(creator_id);
CREATE INDEX idx_segments_activity_type ON segments(activity_type);
CREATE INDEX idx_segments_visibility ON segments(visibility) WHERE deleted_at IS NULL;
CREATE INDEX idx_segments_geo_gist ON segments USING GIST (geo);
CREATE INDEX idx_segments_start_gist ON segments USING GIST (start_point);

-- Segment efforts - each time someone completes a segment
CREATE TABLE segment_efforts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    segment_id UUID NOT NULL REFERENCES segments(id) ON DELETE CASCADE,
    activity_id UUID NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Timing
    started_at TIMESTAMP WITH TIME ZONE NOT NULL,
    elapsed_time_seconds FLOAT NOT NULL,

    -- Computed from activity track
    moving_time_seconds FLOAT,
    average_speed_mps FLOAT,
    max_speed_mps FLOAT,

    -- Rankings (computed/cached)
    is_personal_record BOOLEAN NOT NULL DEFAULT FALSE,

    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_segment_efforts_segment ON segment_efforts(segment_id);
CREATE INDEX idx_segment_efforts_activity ON segment_efforts(activity_id);
CREATE INDEX idx_segment_efforts_user ON segment_efforts(user_id);
CREATE INDEX idx_segment_efforts_time ON segment_efforts(segment_id, elapsed_time_seconds);
CREATE INDEX idx_segment_efforts_pr ON segment_efforts(segment_id, user_id) WHERE is_personal_record = TRUE;

-- Segment stars (favorites)
CREATE TABLE segment_stars (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    segment_id UUID NOT NULL REFERENCES segments(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, segment_id)
);
