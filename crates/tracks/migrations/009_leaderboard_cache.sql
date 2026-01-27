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
