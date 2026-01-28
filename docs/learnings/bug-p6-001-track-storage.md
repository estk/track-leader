# BUG-P6-001: GPS Track Storage Refactoring

**Date:** January 28, 2026
**Status:** Resolved

## Problem

The `/activities/{id}/track` endpoint returned 404 "Not Found" because it tried to read and parse the raw GPX file from object storage on every request. This was:

1. **Inefficient** - parsing XML on every request
2. **Fragile** - depended on GPX file existing in object store
3. **Duplicative** - we already parse GPX during upload

## Root Cause

The `get_activity_track` handler in `handlers.rs` was:
1. Getting the activity from the database
2. Reading the GPX file from object storage using `activity.object_store_path`
3. Parsing the GPX XML
4. Extracting track points

If the GPX file was missing or the object store was unavailable, the endpoint failed.

## Solution

Store track data in PostgreSQL during upload, retrieve from database on request.

### Schema Change (Migration 014)

Converted `tracks.geo` from `GEOGRAPHY(LineString, 4326)` to `GEOGRAPHY(LineStringZM, 4326)`:

- **X** = longitude
- **Y** = latitude
- **Z** = elevation in meters
- **M** = timestamp as unix epoch seconds

This 4D geometry stores all GPX trackpoint data in a single PostGIS column.

### Code Changes

1. **`activity_queue.rs`**: Extract elevation and timestamps during GPX parsing, store via `save_track_geometry_with_data()`

2. **`database.rs`**:
   - `save_track_geometry_with_data()` - builds LineStringZM WKT
   - `get_track_points()` - extracts XYZM using `ST_DumpPoints()`

3. **`handlers.rs`**: `get_activity_track` reads from database instead of object storage

## Key Learnings

### PostGIS LineStringZM

- PostGIS supports 4D geometries out of the box
- Z and M dimensions are preserved through all operations
- 2D spatial operations (ST_Intersects, ST_DWithin) ignore Z and M
- Use `ST_Force4D()` to upgrade existing 2D geometries (sets Z=0, M=0)

### WKT Format

```
LINESTRING ZM(lon lat ele epoch, lon lat ele epoch, ...)
```

Note: WKT uses `lon lat` order, not `lat lon` like most APIs.

### Extracting Points

```sql
SELECT
    ST_X(geom) as lon,
    ST_Y(geom) as lat,
    ST_Z(geom) as elevation,
    ST_M(geom) as epoch
FROM tracks t,
LATERAL ST_DumpPoints(t.geo::geometry) AS dp(path, geom)
WHERE t.activity_id = $1
ORDER BY dp.path[1]
```

The `LATERAL` join with `ST_DumpPoints` expands the geometry into individual points while preserving order.

### Timestamp Storage

Unix epoch as float64 in M dimension:
- Allows sub-second precision if needed
- 0 indicates missing timestamp
- Converts back to ISO 8601 for API responses

### Backwards Compatibility

For 2D WKT input (legacy or segment creation), use `ST_Force4D()`:

```sql
INSERT INTO tracks (user_id, activity_id, geo)
VALUES ($1, $2, ST_Force4D(ST_GeogFromText($3)::geometry)::geography)
```

## Testing

1. Existing tracks (migrated from 2D) work - elevation/time show as `null`
2. New uploads store elevation and timestamps correctly
3. API returns data in expected format with RFC3339 timestamps
4. Segment matching unchanged - PostGIS ignores Z/M for spatial queries

## Future Work

- `activity_sensor_data` table created for FIT/TCX sensor data (heart rate, cadence, power)
- Arrays parallel to track geometry points for efficient bulk reads
