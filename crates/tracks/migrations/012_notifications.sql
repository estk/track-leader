-- Notifications system
-- Stores in-app notifications for users

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

-- Primary query: get notifications for a user, newest first
CREATE INDEX idx_notifications_user_created ON notifications(user_id, created_at DESC);

-- Efficient unread count query
CREATE INDEX idx_notifications_unread ON notifications(user_id) WHERE read_at IS NULL;
