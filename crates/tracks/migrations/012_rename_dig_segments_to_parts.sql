-- Migration: 012_rename_dig_segments_to_parts
-- Rename dig "segments" to "parts" to avoid confusion with competitive segments
-- This handles existing databases; new databases already have correct names from updated 008

-- Only rename if old name exists
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'activity_dig_segments') THEN
        ALTER TABLE activity_dig_segments RENAME TO activity_dig_parts;
    END IF;

    IF EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_activity_dig_segments_activity_id') THEN
        ALTER INDEX idx_activity_dig_segments_activity_id RENAME TO idx_activity_dig_parts_activity_id;
    END IF;
END $$;
