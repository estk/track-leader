-- Convert tracks geometry to 4D LineStringZM
-- X = longitude, Y = latitude, Z = elevation (meters), M = timestamp (unix epoch seconds)

-- First drop any indexes on the geo column that might interfere
DROP INDEX IF EXISTS idx_tracks_geo;

-- Convert existing 2D LineString to 4D LineStringZM
-- Existing points get Z=0 (no elevation), M=0 (no timestamp)
ALTER TABLE tracks
  ALTER COLUMN geo TYPE GEOGRAPHY(LineStringZM, 4326)
  USING ST_Force4D(geo::geometry)::geography;

-- Recreate spatial index
CREATE INDEX idx_tracks_geo ON tracks USING GIST (geo);

COMMENT ON COLUMN tracks.geo IS 'Track geometry: X=lon, Y=lat, Z=elevation(m), M=timestamp(unix epoch)';

-- Drop the elevation/time arrays that were added in the previous migration attempt
-- (if they exist from the previous migration)
ALTER TABLE tracks DROP COLUMN IF EXISTS elevations;
ALTER TABLE tracks DROP COLUMN IF EXISTS recorded_times;

-- Sensor data arrays (parallel to track geometry points)
-- For future use when importing FIT/TCX files
CREATE TABLE IF NOT EXISTS activity_sensor_data (
  activity_id UUID PRIMARY KEY REFERENCES activities(id) ON DELETE CASCADE,
  heart_rates int[],
  cadences int[],
  powers int[],
  temperatures double precision[]
);

COMMENT ON TABLE activity_sensor_data IS 'Sensor data arrays parallel to track geometry points';
