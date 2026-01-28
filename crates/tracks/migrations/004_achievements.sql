-- Gamification: achievements and leaderboards

-- Achievement types
CREATE TYPE achievement_type AS ENUM ('kom', 'qom', 'local_legend', 'course_record');

-- Achievements table
CREATE TABLE achievements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    segment_id UUID NOT NULL REFERENCES segments(id) ON DELETE CASCADE,
    effort_id UUID REFERENCES segment_efforts(id) ON DELETE SET NULL,

    achievement_type achievement_type NOT NULL,

    -- When the achievement was earned and lost
    earned_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    lost_at TIMESTAMP WITH TIME ZONE,

    -- For local legend: track the effort count during the achievement period
    effort_count INTEGER,

    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Unique constraint: only one active achievement per user/segment/type
CREATE UNIQUE INDEX idx_achievements_active
    ON achievements(user_id, segment_id, achievement_type)
    WHERE lost_at IS NULL;

CREATE INDEX idx_achievements_user ON achievements(user_id);
CREATE INDEX idx_achievements_segment ON achievements(segment_id);
CREATE INDEX idx_achievements_type ON achievements(achievement_type);
CREATE INDEX idx_achievements_earned ON achievements(earned_at DESC);
CREATE INDEX idx_achievements_active_segment ON achievements(segment_id, achievement_type) WHERE lost_at IS NULL;

COMMENT ON TABLE achievements IS 'User achievements like KOM, QOM, and Local Legend crowns';
COMMENT ON COLUMN achievements.achievement_type IS 'Type of crown: kom (King of Mountain), qom (Queen of Mountain), local_legend, course_record';
COMMENT ON COLUMN achievements.lost_at IS 'When the user lost this achievement (dethroned); NULL if still active';
COMMENT ON COLUMN achievements.effort_count IS 'For local_legend: number of efforts during the achievement period';

-- Leaderboard cache table for fast retrieval of pre-computed rankings
CREATE TABLE leaderboard_cache (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    segment_id UUID NOT NULL REFERENCES segments(id) ON DELETE CASCADE,

    -- Cache key components
    scope TEXT NOT NULL,  -- 'all_time', 'year', 'month', 'week'
    scope_value TEXT,     -- '2024', '2024-01', '2024-W03' for time-based scopes
    gender gender,        -- NULL means all genders
    age_group TEXT,       -- NULL means all ages, e.g., '25-34'

    -- Cached data (JSON array of leaderboard entries)
    entries JSONB NOT NULL,
    entry_count INTEGER NOT NULL DEFAULT 0,

    -- Cache metadata
    computed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,

    -- Unique constraint for cache key
    CONSTRAINT leaderboard_cache_key UNIQUE (segment_id, scope, scope_value, gender, age_group)
);

CREATE INDEX idx_leaderboard_cache_segment ON leaderboard_cache(segment_id);
CREATE INDEX idx_leaderboard_cache_expires ON leaderboard_cache(expires_at);

COMMENT ON TABLE leaderboard_cache IS 'Pre-computed leaderboard entries for fast retrieval';
COMMENT ON COLUMN leaderboard_cache.scope IS 'Time scope: all_time, year, month, week';
COMMENT ON COLUMN leaderboard_cache.scope_value IS 'Specific time period value for year/month/week scopes';
COMMENT ON COLUMN leaderboard_cache.entries IS 'JSON array of LeaderboardEntry objects';
