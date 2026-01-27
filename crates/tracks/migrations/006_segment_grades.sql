-- Add grade and climb category columns to segments table
ALTER TABLE segments
    ADD COLUMN average_grade FLOAT,
    ADD COLUMN max_grade FLOAT,
    ADD COLUMN climb_category INTEGER;

-- Climb category values:
-- NULL = Not categorized (flat or unknown)
-- 4 = Cat 4 (20-39 points)
-- 3 = Cat 3 (40-79 points)
-- 2 = Cat 2 (80-159 points)
-- 1 = Cat 1 (160-319 points)
-- 0 = HC / Hors Catégorie (320+ points)
-- Points = elevation_gain_meters * (distance_meters / 1000) * average_grade_factor
