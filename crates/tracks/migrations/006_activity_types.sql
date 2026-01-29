-- Activity types table with canonical short names

-- Activity types table
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

-- Seed built-in types with fixed UUIDs for referential integrity
INSERT INTO activity_types (id, name, is_builtin) VALUES
    ('00000000-0000-0000-0000-000000000001', 'walk', true),
    ('00000000-0000-0000-0000-000000000002', 'run', true),
    ('00000000-0000-0000-0000-000000000003', 'hike', true),
    ('00000000-0000-0000-0000-000000000004', 'road', true),
    ('00000000-0000-0000-0000-000000000005', 'mtb', true),
    ('00000000-0000-0000-0000-000000000006', 'emtb', true),
    ('00000000-0000-0000-0000-000000000007', 'gravel', true),
    ('00000000-0000-0000-0000-000000000008', 'unknown', true);

-- Aliases table for flexible lookup (allows 1:many mapping)
CREATE TABLE activity_aliases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alias TEXT NOT NULL,
    activity_type_id UUID NOT NULL REFERENCES activity_types(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(alias, activity_type_id)
);

CREATE INDEX idx_activity_aliases_alias ON activity_aliases(alias);

COMMENT ON TABLE activity_aliases IS 'Maps alternative names to canonical activity types. 1:many for disambiguation UI.';

-- Seed aliases for common alternative names
INSERT INTO activity_aliases (alias, activity_type_id) VALUES
    ('walking', '00000000-0000-0000-0000-000000000001'),
    ('running', '00000000-0000-0000-0000-000000000002'),
    ('hiking', '00000000-0000-0000-0000-000000000003'),
    ('road_cycling', '00000000-0000-0000-0000-000000000004'),
    ('mountain_biking', '00000000-0000-0000-0000-000000000005'),
    ('e-mtb', '00000000-0000-0000-0000-000000000006');

-- Seed ambiguous aliases (for user disambiguation)
INSERT INTO activity_aliases (alias, activity_type_id) VALUES
    ('biking', '00000000-0000-0000-0000-000000000004'),
    ('biking', '00000000-0000-0000-0000-000000000005'),
    ('biking', '00000000-0000-0000-0000-000000000006'),
    ('biking', '00000000-0000-0000-0000-000000000007'),
    ('cycling', '00000000-0000-0000-0000-000000000004'),
    ('cycling', '00000000-0000-0000-0000-000000000005'),
    ('cycling', '00000000-0000-0000-0000-000000000007'),
    ('ebike', '00000000-0000-0000-0000-000000000006');

-- Add foreign key constraints to activities and segments
-- These tables were created earlier without FKs to avoid circular dependency
ALTER TABLE activities ADD CONSTRAINT fk_activities_activity_type
    FOREIGN KEY (activity_type_id) REFERENCES activity_types(id);

ALTER TABLE segments ADD CONSTRAINT fk_segments_activity_type
    FOREIGN KEY (activity_type_id) REFERENCES activity_types(id);
