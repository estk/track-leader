-- Migration: 011_dig_activity_type
-- Add DIG/Trail Work activity type for tracking trail maintenance time

INSERT INTO activity_types (id, name, is_builtin) VALUES
    ('00000000-0000-0000-0000-000000000009', 'dig', true)
ON CONFLICT (id) DO NOTHING;

-- Add alias for trail work
INSERT INTO activity_aliases (alias, activity_type_id) VALUES
    ('trail_work', '00000000-0000-0000-0000-000000000009')
ON CONFLICT DO NOTHING;
