-- Performance indexes for Phase 6 launch optimization
-- These indexes optimize common query patterns in the application

-- Activities: User's activities by type and date (for user activity lists)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_activities_user_type_date
  ON activities(user_id, activity_type, submitted_at DESC);

-- Segment efforts: Leaderboard queries (segment sorted by time)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_efforts_segment_time
  ON segment_efforts(segment_id, elapsed_time_seconds ASC);

-- Segment efforts: User's efforts on a segment (for PR tracking)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_efforts_user_segment
  ON segment_efforts(user_id, segment_id, elapsed_time_seconds ASC);

-- Notifications: Unread notifications for a user
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_notifications_user_unread
  ON notifications(user_id, read_at) WHERE read_at IS NULL;

-- Notifications: User's notifications by time
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_notifications_user_time
  ON notifications(user_id, created_at DESC);

-- Feed: Recent activities for feed (public activities by time)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_activities_feed
  ON activities(submitted_at DESC) WHERE visibility = 'public';

-- Follows: Efficient lookup of who follows whom
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_follows_follower
  ON follows(follower_id, created_at DESC);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_follows_following
  ON follows(following_id, created_at DESC);

-- Kudos: Count and lookup for activities
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_kudos_activity
  ON kudos(activity_id, created_at DESC);

-- Comments: Activity comments by time
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_comments_activity
  ON comments(activity_id, created_at ASC);

-- Segments: Nearby segment lookup by activity type
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_segments_type
  ON segments(activity_type, created_at DESC);

-- Starred segments: User's starred segments
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_starred_user
  ON starred_segments(user_id, created_at DESC);
