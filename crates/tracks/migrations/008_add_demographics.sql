-- Add demographic fields to users for leaderboard filtering
-- Gender enum for leaderboard filtering
CREATE TYPE gender AS ENUM ('male', 'female', 'other', 'prefer_not_to_say');

ALTER TABLE users
    ADD COLUMN gender gender,
    ADD COLUMN birth_year INTEGER,
    ADD COLUMN weight_kg FLOAT,
    ADD COLUMN country TEXT,
    ADD COLUMN region TEXT;

-- Index for demographic filtering in leaderboards
CREATE INDEX idx_users_gender ON users(gender) WHERE gender IS NOT NULL;
CREATE INDEX idx_users_birth_year ON users(birth_year) WHERE birth_year IS NOT NULL;
CREATE INDEX idx_users_country ON users(country) WHERE country IS NOT NULL;

COMMENT ON COLUMN users.gender IS 'User gender for leaderboard filtering';
COMMENT ON COLUMN users.birth_year IS 'Birth year for age group filtering';
COMMENT ON COLUMN users.weight_kg IS 'Weight in kg for power calculations';
COMMENT ON COLUMN users.country IS 'Country for regional leaderboards';
COMMENT ON COLUMN users.region IS 'State/province/region for local leaderboards';
