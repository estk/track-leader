-- Add 'teams_only' as a valid visibility option for activities and segments
-- The visibility column is TEXT, so no schema change is needed.
-- This migration documents the addition of the 'teams_only' value.

-- Activities with 'teams_only' visibility are only visible to:
-- 1. The activity owner
-- 2. Members of teams the activity is shared with (via activity_teams table)

-- Segments with 'teams_only' visibility are only visible to:
-- 1. The segment creator
-- 2. Members of teams the segment is shared with (via segment_teams table)

-- Add a comment to document valid visibility values
COMMENT ON COLUMN activities.visibility IS 'Visibility: public, private, or teams_only';
COMMENT ON COLUMN segments.visibility IS 'Visibility: public, private, or teams_only';
