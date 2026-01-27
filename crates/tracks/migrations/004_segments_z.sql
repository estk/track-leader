-- Add Z dimension support to segments geo column for elevation data
ALTER TABLE segments
    ALTER COLUMN geo TYPE GEOGRAPHY(LineStringZ, 4326)
    USING ST_Force3D(geo::geometry)::geography;
