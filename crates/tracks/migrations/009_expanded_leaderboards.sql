-- Migration: 009_expanded_leaderboards
-- Add leaderboard_type enum and featured_leaderboard column to teams

-- Leaderboard type enum for identifying which leaderboard is featured
CREATE TYPE leaderboard_type AS ENUM (
    'crowns',
    'distance',
    'dig_time',
    'dig_percentage',
    'average_speed'
);

-- Add featured leaderboard column to teams table
ALTER TABLE teams ADD COLUMN featured_leaderboard leaderboard_type;

-- Index for querying teams by featured leaderboard type
CREATE INDEX idx_teams_featured_leaderboard ON teams(featured_leaderboard) WHERE featured_leaderboard IS NOT NULL;
