-- Track Leader - Supabase Database Setup
-- Run this in the Supabase SQL Editor (Dashboard > SQL Editor > New Query)
--
-- This combines all migrations into a single script for initial setup.
-- For incremental updates, run individual migration files.

-- ============================================================================
-- 001_core.sql - Foundation: users, activities, tracks, scores
-- ============================================================================

-- PostGIS Extension (already enabled on Supabase, but safe to run)
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

CREATE INDEX idx_users_gender ON users(gender) WHERE gender IS NOT NULL;
CREATE INDEX idx_users_birth_year ON users(birth_year) WHERE birth_year IS NOT NULL;
CREATE INDEX idx_users_weight_kg ON users(weight_kg) WHERE weight_kg IS NOT NULL;
CREATE INDEX idx_users_country ON users(country) WHERE country IS NOT NULL;

-- Activities table
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
    type_boundaries TIMESTAMPTZ[],
    segment_types UUID[]
);

COMMENT ON COLUMN activities.visibility IS 'Visibility: public, private, or teams_only';
COMMENT ON COLUMN activities.type_boundaries IS 'Multi-sport: timestamps marking segment boundaries';
COMMENT ON COLUMN activities.segment_types IS 'Multi-sport: activity type IDs for each segment';

CREATE INDEX idx_activities_user_id ON activities(user_id);
CREATE INDEX idx_activities_submitted_at ON activities(submitted_at);
CREATE INDEX idx_activities_visibility ON activities(visibility) WHERE deleted_at IS NULL;
CREATE INDEX idx_activities_user_date ON activities(user_id, submitted_at DESC) WHERE deleted_at IS NULL;
CREATE INDEX idx_activities_user_type_date ON activities(user_id, activity_type_id, submitted_at DESC);
CREATE INDEX idx_activities_feed ON activities(submitted_at DESC) WHERE visibility = 'public';

-- Tracks table with LineStringZM geometry
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

-- Sensor data arrays
CREATE TABLE activity_sensor_data (
    activity_id UUID PRIMARY KEY REFERENCES activities(id) ON DELETE CASCADE,
    heart_rates int[],
    cadences int[],
    powers int[],
    temperatures double precision[]
);

COMMENT ON TABLE activity_sensor_data IS 'Sensor data arrays parallel to track geometry points';

-- ============================================================================
-- 002_segments.sql - Segment definitions and efforts
-- ============================================================================

CREATE TABLE segments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    creator_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    activity_type_id UUID NOT NULL,
    geo GEOGRAPHY(LineStringZ, 4326) NOT NULL,
    start_point GEOGRAPHY(Point, 4326) NOT NULL,
    end_point GEOGRAPHY(Point, 4326) NOT NULL,
    distance_meters FLOAT NOT NULL,
    elevation_gain_meters FLOAT,
    elevation_loss_meters FLOAT,
    average_grade FLOAT,
    max_grade FLOAT,
    climb_category INTEGER,
    visibility TEXT NOT NULL DEFAULT 'public',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE,
    deleted_at TIMESTAMP WITH TIME ZONE
);

COMMENT ON COLUMN segments.visibility IS 'Visibility: public, private, or teams_only';

CREATE INDEX idx_segments_creator ON segments(creator_id);
CREATE INDEX idx_segments_activity_type_id ON segments(activity_type_id);
CREATE INDEX idx_segments_visibility ON segments(visibility) WHERE deleted_at IS NULL;
CREATE INDEX idx_segments_geo_gist ON segments USING GIST (geo);
CREATE INDEX idx_segments_start_gist ON segments USING GIST (start_point);
CREATE INDEX idx_segments_type ON segments(activity_type_id, created_at DESC);

CREATE TABLE segment_efforts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    segment_id UUID NOT NULL REFERENCES segments(id) ON DELETE CASCADE,
    activity_id UUID NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    started_at TIMESTAMP WITH TIME ZONE NOT NULL,
    elapsed_time_seconds FLOAT NOT NULL,
    moving_time_seconds FLOAT,
    average_speed_mps FLOAT,
    max_speed_mps FLOAT,
    start_fraction FLOAT,
    end_fraction FLOAT,
    is_personal_record BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

COMMENT ON COLUMN segment_efforts.start_fraction IS 'Fractional position (0-1) on activity track where segment starts';
COMMENT ON COLUMN segment_efforts.end_fraction IS 'Fractional position (0-1) on activity track where segment ends';

CREATE INDEX idx_segment_efforts_segment ON segment_efforts(segment_id);
CREATE INDEX idx_segment_efforts_activity ON segment_efforts(activity_id);
CREATE INDEX idx_segment_efforts_user ON segment_efforts(user_id);
CREATE INDEX idx_segment_efforts_time ON segment_efforts(segment_id, elapsed_time_seconds);
CREATE INDEX idx_segment_efforts_pr ON segment_efforts(segment_id, user_id) WHERE is_personal_record = TRUE;
CREATE INDEX idx_efforts_segment_time ON segment_efforts(segment_id, elapsed_time_seconds ASC);
CREATE INDEX idx_efforts_user_segment ON segment_efforts(user_id, segment_id, elapsed_time_seconds ASC);

CREATE TABLE segment_stars (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    segment_id UUID NOT NULL REFERENCES segments(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, segment_id)
);

CREATE INDEX idx_segment_stars_user ON segment_stars(user_id, created_at DESC);

-- ============================================================================
-- 003_social.sql - Social features: follows, kudos, comments, notifications
-- ============================================================================

CREATE TABLE follows (
    follower_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    following_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (follower_id, following_id),
    CONSTRAINT no_self_follow CHECK (follower_id != following_id)
);

CREATE INDEX idx_follows_following ON follows(following_id);
CREATE INDEX idx_follows_follower ON follows(follower_id);
CREATE INDEX idx_follows_follower_time ON follows(follower_id, created_at DESC);
CREATE INDEX idx_follows_following_time ON follows(following_id, created_at DESC);

CREATE TABLE kudos (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    activity_id UUID NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (user_id, activity_id)
);

CREATE INDEX idx_kudos_activity ON kudos(activity_id);
CREATE INDEX idx_kudos_activity_time ON kudos(activity_id, created_at DESC);

CREATE TABLE comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    activity_id UUID NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES comments(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_comments_activity ON comments(activity_id, created_at);
CREATE INDEX idx_comments_activity_time ON comments(activity_id, created_at ASC);

CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    notification_type TEXT NOT NULL,
    actor_id UUID REFERENCES users(id) ON DELETE SET NULL,
    target_type TEXT,
    target_id UUID,
    message TEXT,
    read_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_notifications_user_created ON notifications(user_id, created_at DESC);
CREATE INDEX idx_notifications_unread ON notifications(user_id) WHERE read_at IS NULL;
CREATE INDEX idx_notifications_user_unread ON notifications(user_id, read_at) WHERE read_at IS NULL;
CREATE INDEX idx_notifications_user_time ON notifications(user_id, created_at DESC);

-- ============================================================================
-- 004_achievements.sql - Gamification (with local_legend already removed)
-- ============================================================================

CREATE TYPE achievement_type AS ENUM ('kom', 'qom', 'course_record');

CREATE TABLE achievements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    segment_id UUID NOT NULL REFERENCES segments(id) ON DELETE CASCADE,
    effort_id UUID REFERENCES segment_efforts(id) ON DELETE SET NULL,
    achievement_type achievement_type NOT NULL,
    earned_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    lost_at TIMESTAMP WITH TIME ZONE,
    effort_count INTEGER,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_achievements_active
    ON achievements(user_id, segment_id, achievement_type)
    WHERE lost_at IS NULL;

CREATE INDEX idx_achievements_user ON achievements(user_id);
CREATE INDEX idx_achievements_segment ON achievements(segment_id);
CREATE INDEX idx_achievements_type ON achievements(achievement_type);
CREATE INDEX idx_achievements_earned ON achievements(earned_at DESC);
CREATE INDEX idx_achievements_active_segment ON achievements(segment_id, achievement_type) WHERE lost_at IS NULL;

COMMENT ON TABLE achievements IS 'User achievements like KOM, QOM crowns';
COMMENT ON COLUMN achievements.achievement_type IS 'Type of crown: kom, qom, course_record';
COMMENT ON COLUMN achievements.lost_at IS 'When the user lost this achievement; NULL if still active';

CREATE TABLE leaderboard_cache (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    segment_id UUID NOT NULL REFERENCES segments(id) ON DELETE CASCADE,
    scope TEXT NOT NULL,
    scope_value TEXT,
    gender gender,
    age_group TEXT,
    entries JSONB NOT NULL,
    entry_count INTEGER NOT NULL DEFAULT 0,
    computed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    CONSTRAINT leaderboard_cache_key UNIQUE (segment_id, scope, scope_value, gender, age_group)
);

CREATE INDEX idx_leaderboard_cache_segment ON leaderboard_cache(segment_id);
CREATE INDEX idx_leaderboard_cache_expires ON leaderboard_cache(expires_at);

COMMENT ON TABLE leaderboard_cache IS 'Pre-computed leaderboard entries for fast retrieval';

-- ============================================================================
-- 005_teams.sql - Teams: Group-based access control
-- ============================================================================

CREATE TYPE team_role AS ENUM ('owner', 'admin', 'member');
CREATE TYPE team_visibility AS ENUM ('public', 'private');
CREATE TYPE team_join_policy AS ENUM ('open', 'request', 'invitation');

CREATE TABLE teams (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT,
    avatar_url TEXT,
    visibility team_visibility NOT NULL DEFAULT 'private',
    join_policy team_join_policy NOT NULL DEFAULT 'invitation',
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    member_count INTEGER NOT NULL DEFAULT 1,
    activity_count INTEGER NOT NULL DEFAULT 0,
    segment_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_teams_owner ON teams(owner_id);
CREATE INDEX idx_teams_visibility ON teams(visibility) WHERE deleted_at IS NULL;
CREATE INDEX idx_teams_owner_created ON teams(owner_id, created_at DESC) WHERE deleted_at IS NULL;

CREATE TABLE team_memberships (
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role team_role NOT NULL DEFAULT 'member',
    invited_by UUID REFERENCES users(id) ON DELETE SET NULL,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (team_id, user_id)
);

CREATE INDEX idx_team_memberships_user ON team_memberships(user_id);
CREATE INDEX idx_team_memberships_team ON team_memberships(team_id);
CREATE INDEX idx_team_memberships_user_joined ON team_memberships(user_id, joined_at DESC);
CREATE INDEX idx_team_memberships_team_role ON team_memberships(team_id, role);

CREATE TABLE team_join_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    message TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    reviewed_by UUID REFERENCES users(id) ON DELETE SET NULL,
    reviewed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (team_id, user_id)
);

CREATE INDEX idx_team_join_requests_team ON team_join_requests(team_id) WHERE status = 'pending';
CREATE INDEX idx_team_join_requests_user ON team_join_requests(user_id);
CREATE INDEX idx_team_join_requests_pending ON team_join_requests(team_id, created_at DESC) WHERE status = 'pending';

CREATE TABLE team_invitations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    email TEXT NOT NULL,
    invited_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role team_role NOT NULL DEFAULT 'member',
    token TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    accepted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (team_id, email)
);

CREATE INDEX idx_team_invitations_token ON team_invitations(token) WHERE accepted_at IS NULL;
CREATE INDEX idx_team_invitations_team ON team_invitations(team_id) WHERE accepted_at IS NULL;
CREATE INDEX idx_team_invitations_email ON team_invitations(email) WHERE accepted_at IS NULL;
CREATE INDEX idx_team_invitations_expires ON team_invitations(expires_at) WHERE accepted_at IS NULL;

CREATE TABLE activity_teams (
    activity_id UUID NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    shared_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    shared_by UUID REFERENCES users(id) ON DELETE SET NULL,
    PRIMARY KEY (activity_id, team_id)
);

CREATE INDEX idx_activity_teams_team ON activity_teams(team_id);
CREATE INDEX idx_activity_teams_activity ON activity_teams(activity_id);
CREATE INDEX idx_activity_teams_team_shared ON activity_teams(team_id, shared_at DESC);

CREATE TABLE segment_teams (
    segment_id UUID NOT NULL REFERENCES segments(id) ON DELETE CASCADE,
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    shared_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (segment_id, team_id)
);

CREATE INDEX idx_segment_teams_team ON segment_teams(team_id);
CREATE INDEX idx_segment_teams_segment ON segment_teams(segment_id);
CREATE INDEX idx_segment_teams_team_shared ON segment_teams(team_id, shared_at DESC);

-- ============================================================================
-- 006_activity_types.sql - Activity types with canonical short names
-- ============================================================================

CREATE TABLE activity_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    is_builtin BOOLEAN NOT NULL DEFAULT FALSE,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_activity_types_name ON activity_types(name);

COMMENT ON TABLE activity_types IS 'Canonical activity types with short names';
COMMENT ON COLUMN activity_types.is_builtin IS 'True for system-provided types';

-- Seed built-in types with fixed UUIDs
INSERT INTO activity_types (id, name, is_builtin) VALUES
    ('00000000-0000-0000-0000-000000000001', 'walk', true),
    ('00000000-0000-0000-0000-000000000002', 'run', true),
    ('00000000-0000-0000-0000-000000000003', 'hike', true),
    ('00000000-0000-0000-0000-000000000004', 'road', true),
    ('00000000-0000-0000-0000-000000000005', 'mtb', true),
    ('00000000-0000-0000-0000-000000000006', 'emtb', true),
    ('00000000-0000-0000-0000-000000000007', 'gravel', true),
    ('00000000-0000-0000-0000-000000000008', 'unknown', true);

CREATE TABLE activity_aliases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alias TEXT NOT NULL,
    activity_type_id UUID NOT NULL REFERENCES activity_types(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(alias, activity_type_id)
);

CREATE INDEX idx_activity_aliases_alias ON activity_aliases(alias);

COMMENT ON TABLE activity_aliases IS 'Maps alternative names to canonical activity types';

-- Seed aliases
INSERT INTO activity_aliases (alias, activity_type_id) VALUES
    ('walking', '00000000-0000-0000-0000-000000000001'),
    ('running', '00000000-0000-0000-0000-000000000002'),
    ('hiking', '00000000-0000-0000-0000-000000000003'),
    ('road_cycling', '00000000-0000-0000-0000-000000000004'),
    ('mountain_biking', '00000000-0000-0000-0000-000000000005'),
    ('e-mtb', '00000000-0000-0000-0000-000000000006'),
    ('biking', '00000000-0000-0000-0000-000000000004'),
    ('biking', '00000000-0000-0000-0000-000000000005'),
    ('biking', '00000000-0000-0000-0000-000000000006'),
    ('biking', '00000000-0000-0000-0000-000000000007'),
    ('cycling', '00000000-0000-0000-0000-000000000004'),
    ('cycling', '00000000-0000-0000-0000-000000000005'),
    ('cycling', '00000000-0000-0000-0000-000000000007'),
    ('ebike', '00000000-0000-0000-0000-000000000006');

-- Add foreign key constraints
ALTER TABLE activities ADD CONSTRAINT fk_activities_activity_type
    FOREIGN KEY (activity_type_id) REFERENCES activity_types(id);

ALTER TABLE segments ADD CONSTRAINT fk_segments_activity_type
    FOREIGN KEY (activity_type_id) REFERENCES activity_types(id);

-- ============================================================================
-- 008_stopped_segments.sql - Stopped segment detection
-- ============================================================================

CREATE TABLE IF NOT EXISTS activity_stopped_segments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    activity_id UUID NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    duration_seconds DOUBLE PRECISION NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS activity_dig_parts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    activity_id UUID NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    duration_seconds DOUBLE PRECISION NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_activity_stopped_segments_activity_id
    ON activity_stopped_segments(activity_id);
CREATE INDEX IF NOT EXISTS idx_activity_dig_parts_activity_id
    ON activity_dig_parts(activity_id);

-- ============================================================================
-- Done! Your Supabase database is ready for Track Leader.
-- ============================================================================
