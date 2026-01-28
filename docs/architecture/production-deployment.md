# Production Deployment Guide

## Database Migrations

Migrations are managed by SQLx and run automatically on backend startup. For development, this works seamlessly. For production deployments, some migrations require special handling.

### Performance Indexes

Migration `015_performance_indexes.sql` creates indexes that optimize common query patterns. In development, these run as regular `CREATE INDEX` statements within a transaction.

**For production deployments with existing data**, these indexes should be created manually using `CONCURRENTLY` to avoid locking tables during creation:

```sql
-- Connect to production database and run each index separately
-- These can be run while the application is serving traffic

-- Activities: User's activities by type and date
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_activities_user_type_date
  ON activities(user_id, activity_type, submitted_at DESC);

-- Segment efforts: Leaderboard queries
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_efforts_segment_time
  ON segment_efforts(segment_id, elapsed_time_seconds ASC);

-- Segment efforts: User's efforts on a segment (PR tracking)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_efforts_user_segment
  ON segment_efforts(user_id, segment_id, elapsed_time_seconds ASC);

-- Notifications: Unread notifications for a user
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_notifications_user_unread
  ON notifications(user_id, read_at) WHERE read_at IS NULL;

-- Notifications: User's notifications by time
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_notifications_user_time
  ON notifications(user_id, created_at DESC);

-- Feed: Recent activities for feed
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_activities_feed
  ON activities(submitted_at DESC) WHERE visibility = 'public';

-- Follows: Lookup of who follows whom
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

-- Segment stars: User's starred segments
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_segment_stars_user
  ON segment_stars(user_id, created_at DESC);
```

**Why CONCURRENTLY?**
- Regular `CREATE INDEX` locks the table for writes during index creation
- On large tables, this can cause significant downtime
- `CREATE INDEX CONCURRENTLY` builds the index without blocking writes (takes longer but no lock)

**Caveats:**
- `CONCURRENTLY` cannot run inside a transaction
- If interrupted, may leave an invalid index that needs `REINDEX CONCURRENTLY` or `DROP INDEX`
- Slightly slower than regular index creation

### Pre-deployment Checklist

1. **Backup the database** before running migrations
2. **Check for pending migrations**: `cargo sqlx migrate info`
3. **Run migrations**: Application startup handles this automatically
4. **Create concurrent indexes** (if deploying to existing data): Run the SQL above manually
5. **Verify indexes exist**: `\di` in psql or check `pg_indexes` view

### Monitoring Index Creation

```sql
-- Check progress of concurrent index creation
SELECT
  a.pid,
  a.query,
  p.phase,
  p.blocks_total,
  p.blocks_done,
  round(100.0 * p.blocks_done / nullif(p.blocks_total, 0), 1) AS "% done"
FROM pg_stat_activity a
JOIN pg_stat_progress_create_index p ON p.pid = a.pid;
```
