-- Achievements table for KOM/QOM crowns and other segment achievements
CREATE TYPE achievement_type AS ENUM ('kom', 'qom', 'local_legend', 'course_record');

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
