-- Performance indexes for common query patterns
-- Separated for easy future tuning
-- Note: For production deployment, consider applying these with CONCURRENTLY
-- to avoid table locks on large datasets

-- Users: Demographic filtering for leaderboards
CREATE INDEX IF NOT EXISTS idx_users_gender ON users(gender) WHERE gender IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_users_birth_year ON users(birth_year) WHERE birth_year IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_users_country ON users(country) WHERE country IS NOT NULL;

-- Activities: Visibility and soft delete filtering
CREATE INDEX IF NOT EXISTS idx_activities_visibility ON activities(visibility) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_activities_user_date ON activities(user_id, submitted_at DESC) WHERE deleted_at IS NULL;

-- Activities: User's activities by type and date (for user activity lists)
CREATE INDEX IF NOT EXISTS idx_activities_user_type_date ON activities(user_id, activity_type, submitted_at DESC);

-- Activities: Recent activities for feed (public activities by time)
CREATE INDEX IF NOT EXISTS idx_activities_feed ON activities(submitted_at DESC) WHERE visibility = 'public';

-- Segment efforts: Leaderboard queries (segment sorted by time)
CREATE INDEX IF NOT EXISTS idx_efforts_segment_time ON segment_efforts(segment_id, elapsed_time_seconds ASC);

-- Segment efforts: User's efforts on a segment (for PR tracking)
CREATE INDEX IF NOT EXISTS idx_efforts_user_segment ON segment_efforts(user_id, segment_id, elapsed_time_seconds ASC);

-- Segments: Nearby segment lookup by activity type
CREATE INDEX IF NOT EXISTS idx_segments_type ON segments(activity_type, created_at DESC);

-- Follows: Efficient lookup with timestamps
CREATE INDEX IF NOT EXISTS idx_follows_follower_time ON follows(follower_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_follows_following_time ON follows(following_id, created_at DESC);

-- Kudos: Count and lookup for activities with timestamps
CREATE INDEX IF NOT EXISTS idx_kudos_activity_time ON kudos(activity_id, created_at DESC);

-- Comments: Activity comments by time (ascending for display order)
CREATE INDEX IF NOT EXISTS idx_comments_activity_time ON comments(activity_id, created_at ASC);

-- Notifications: Unread notifications for a user (for badge counts)
CREATE INDEX IF NOT EXISTS idx_notifications_user_unread ON notifications(user_id, read_at) WHERE read_at IS NULL;

-- Notifications: User's notifications by time
CREATE INDEX IF NOT EXISTS idx_notifications_user_time ON notifications(user_id, created_at DESC);
