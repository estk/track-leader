-- Remove Local Legend achievement type

DELETE FROM achievements WHERE achievement_type = 'local_legend';

CREATE TYPE achievement_type_new AS ENUM ('kom', 'qom', 'course_record');

ALTER TABLE achievements
    ALTER COLUMN achievement_type TYPE achievement_type_new
    USING achievement_type::text::achievement_type_new;

DROP TYPE achievement_type;

ALTER TYPE achievement_type_new RENAME TO achievement_type;

COMMENT ON COLUMN achievements.achievement_type IS 'Type of crown: kom, qom, course_record';
