-- Add elevation and time arrays to tracks table
-- These parallel arrays correspond 1:1 with points in the geo LineString

ALTER TABLE tracks
ADD COLUMN elevations double precision[],
ADD COLUMN recorded_times timestamptz[];

COMMENT ON COLUMN tracks.elevations IS 'Elevation in meters for each point (parallel to geo LineString points)';
COMMENT ON COLUMN tracks.recorded_times IS 'Timestamp for each point (parallel to geo LineString points)';
