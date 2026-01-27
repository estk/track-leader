-- Social follows system
-- Enables users to follow other users

CREATE TABLE follows (
    follower_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    following_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (follower_id, following_id),
    -- Prevent self-follows
    CONSTRAINT no_self_follow CHECK (follower_id != following_id)
);

-- Index for "who follows this user" queries
CREATE INDEX idx_follows_following ON follows(following_id);

-- Index for "who does this user follow" queries
CREATE INDEX idx_follows_follower ON follows(follower_id);

-- Denormalized counts for performance (avoids COUNT queries)
ALTER TABLE users ADD COLUMN follower_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE users ADD COLUMN following_count INTEGER NOT NULL DEFAULT 0;
