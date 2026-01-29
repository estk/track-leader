-- Activity types table with canonical short names
-- Replaces the activity_type enum with a table that supports custom types

-- 1. Create activity_types table
CREATE TABLE activity_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    is_builtin BOOLEAN NOT NULL DEFAULT FALSE,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_activity_types_name ON activity_types(name);

COMMENT ON TABLE activity_types IS 'Canonical activity types with short names (run, mtb, road, etc.)';
COMMENT ON COLUMN activity_types.is_builtin IS 'True for system-provided types that cannot be deleted';
COMMENT ON COLUMN activity_types.created_by IS 'User who created this type (NULL for built-in types)';

-- 2. Seed built-in types with fixed UUIDs for referential integrity
INSERT INTO activity_types (id, name, is_builtin) VALUES
    ('00000000-0000-0000-0000-000000000001', 'walk', true),
    ('00000000-0000-0000-0000-000000000002', 'run', true),
    ('00000000-0000-0000-0000-000000000003', 'hike', true),
    ('00000000-0000-0000-0000-000000000004', 'road', true),
    ('00000000-0000-0000-0000-000000000005', 'mtb', true),
    ('00000000-0000-0000-0000-000000000006', 'emtb', true),
    ('00000000-0000-0000-0000-000000000007', 'gravel', true),
    ('00000000-0000-0000-0000-000000000008', 'unknown', true);

-- 3. Create aliases table for flexible lookup (allows 1:many mapping)
CREATE TABLE activity_aliases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alias TEXT NOT NULL,
    activity_type_id UUID NOT NULL REFERENCES activity_types(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(alias, activity_type_id)
);

CREATE INDEX idx_activity_aliases_alias ON activity_aliases(alias);

COMMENT ON TABLE activity_aliases IS 'Maps alternative names to canonical activity types. 1:many for disambiguation UI.';

-- 4. Seed aliases for old enum values (used in migration)
INSERT INTO activity_aliases (alias, activity_type_id) VALUES
    ('walking', '00000000-0000-0000-0000-000000000001'),
    ('running', '00000000-0000-0000-0000-000000000002'),
    ('hiking', '00000000-0000-0000-0000-000000000003'),
    ('road_cycling', '00000000-0000-0000-0000-000000000004'),
    ('mountain_biking', '00000000-0000-0000-0000-000000000005'),
    ('e-mtb', '00000000-0000-0000-0000-000000000006');

-- 5. Seed ambiguous aliases (for user disambiguation)
INSERT INTO activity_aliases (alias, activity_type_id) VALUES
    ('biking', '00000000-0000-0000-0000-000000000004'),
    ('biking', '00000000-0000-0000-0000-000000000005'),
    ('biking', '00000000-0000-0000-0000-000000000006'),
    ('biking', '00000000-0000-0000-0000-000000000007'),
    ('cycling', '00000000-0000-0000-0000-000000000004'),
    ('cycling', '00000000-0000-0000-0000-000000000005'),
    ('cycling', '00000000-0000-0000-0000-000000000007'),
    ('ebike', '00000000-0000-0000-0000-000000000006');

-- 6. Add FK column to activities (nullable initially for migration)
ALTER TABLE activities ADD COLUMN activity_type_id UUID REFERENCES activity_types(id);

-- 7. Migrate existing activities data via aliases (old enum values → type IDs)
UPDATE activities SET activity_type_id = (
    SELECT activity_type_id FROM activity_aliases
    WHERE alias = activities.activity_type::TEXT
    LIMIT 1
);

-- 8. Handle any activities with 'unknown' type (not in aliases)
UPDATE activities SET activity_type_id = '00000000-0000-0000-0000-000000000008'
WHERE activity_type_id IS NULL;

-- 9. Make activity_type_id NOT NULL after migration
ALTER TABLE activities ALTER COLUMN activity_type_id SET NOT NULL;

-- 10. Add multi-sport columns to activities
-- type_boundaries: timestamps marking segment boundaries
-- segment_types: UUIDs referencing activity_types for each segment
-- Invariant: length(segment_types) = length(type_boundaries) - 1
ALTER TABLE activities
    ADD COLUMN type_boundaries TIMESTAMPTZ[],
    ADD COLUMN segment_types UUID[];

COMMENT ON COLUMN activities.type_boundaries IS 'Multi-sport: timestamps marking segment boundaries. First = start, last = end.';
COMMENT ON COLUMN activities.segment_types IS 'Multi-sport: activity type IDs for each segment. Length = type_boundaries.length - 1.';

-- 11. Add FK column to segments (nullable initially for migration)
ALTER TABLE segments ADD COLUMN activity_type_id UUID REFERENCES activity_types(id);

-- 12. Migrate existing segments data via aliases
UPDATE segments SET activity_type_id = (
    SELECT activity_type_id FROM activity_aliases
    WHERE alias = segments.activity_type::TEXT
    LIMIT 1
);

-- 13. Handle any segments with 'unknown' type
UPDATE segments SET activity_type_id = '00000000-0000-0000-0000-000000000008'
WHERE activity_type_id IS NULL;

-- 14. Make segments activity_type_id NOT NULL after migration
ALTER TABLE segments ALTER COLUMN activity_type_id SET NOT NULL;

-- 15. Create index on segments for new column
CREATE INDEX idx_segments_activity_type_id ON segments(activity_type_id);

-- Note: Old enum columns (activities.activity_type, segments.activity_type) are kept
-- for now. They will be dropped in a future migration after verification.
-- DROP COLUMN activity_type and DROP TYPE activity_type will be done separately.
