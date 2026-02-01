-- Migration: Fix NULL started_at values in activities table
-- Activities seeded before started_at was added have NULL values.
-- This migration backfills them with submitted_at as a sensible default.

-- Backfill NULL started_at with submitted_at
UPDATE activities
SET started_at = submitted_at
WHERE started_at IS NULL;

-- Add NOT NULL constraint now that all values are populated
-- Note: We don't add a DEFAULT because started_at should come from GPX data,
-- and the application always provides it. This constraint ensures data integrity.
ALTER TABLE activities
ALTER COLUMN started_at SET NOT NULL;

-- Add comment documenting the column
COMMENT ON COLUMN activities.started_at IS 'When the activity actually occurred (from GPX track data). Required.';
