-- Drop legacy activity_type enum columns now that migration to UUID-based types is complete

-- 1. Drop the enum columns from activities and segments
ALTER TABLE activities DROP COLUMN activity_type;
ALTER TABLE segments DROP COLUMN activity_type;

-- 2. Drop the enum type
DROP TYPE activity_type;
