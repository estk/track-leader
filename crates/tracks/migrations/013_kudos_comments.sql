-- Kudos and comments system

-- Kudos table
CREATE TABLE kudos (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    activity_id UUID NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (user_id, activity_id)
);

CREATE INDEX idx_kudos_activity ON kudos(activity_id);

-- Comments table
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

-- Denormalized counts on activities for performance
ALTER TABLE activities ADD COLUMN kudos_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE activities ADD COLUMN comment_count INTEGER NOT NULL DEFAULT 0;
