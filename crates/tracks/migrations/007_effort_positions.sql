-- Add fractional position columns to segment_efforts for highlighting on activity maps
ALTER TABLE segment_efforts
    ADD COLUMN start_fraction FLOAT,
    ADD COLUMN end_fraction FLOAT;

COMMENT ON COLUMN segment_efforts.start_fraction IS 'Fractional position (0-1) on the activity track where segment starts';
COMMENT ON COLUMN segment_efforts.end_fraction IS 'Fractional position (0-1) on the activity track where segment ends';
