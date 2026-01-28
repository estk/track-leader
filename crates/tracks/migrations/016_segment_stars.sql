-- Segment stars table for users to bookmark/favorite segments
CREATE TABLE IF NOT EXISTS segment_stars (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    segment_id UUID NOT NULL REFERENCES segments(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, segment_id)
);

-- Index for looking up a user's starred segments by time
CREATE INDEX IF NOT EXISTS idx_segment_stars_user
  ON segment_stars(user_id, created_at DESC);
