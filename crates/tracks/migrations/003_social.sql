-- Social features: follows, kudos, comments, notifications

-- Follows table
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

-- Kudos table
CREATE TABLE kudos (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    activity_id UUID NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (user_id, activity_id)
);

CREATE INDEX idx_kudos_activity ON kudos(activity_id);
CREATE INDEX idx_kudos_activity_time ON kudos(activity_id, created_at DESC);

-- Comments table (threaded)
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

-- Notifications table
CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    notification_type TEXT NOT NULL,  -- 'follow', 'kudos', 'comment', 'crown_achieved', 'crown_lost', 'pr'
    actor_id UUID REFERENCES users(id) ON DELETE SET NULL,  -- Who triggered the notification
    target_type TEXT,  -- 'activity', 'segment', 'comment', 'user'
    target_id UUID,    -- ID of the target entity
    message TEXT,      -- Optional custom message
    read_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_notifications_user_created ON notifications(user_id, created_at DESC);
CREATE INDEX idx_notifications_unread ON notifications(user_id) WHERE read_at IS NULL;
CREATE INDEX idx_notifications_user_unread ON notifications(user_id, read_at) WHERE read_at IS NULL;
CREATE INDEX idx_notifications_user_time ON notifications(user_id, created_at DESC);
