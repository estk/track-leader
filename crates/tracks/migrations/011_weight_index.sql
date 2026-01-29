-- Add index on weight_kg for weight class filtering in leaderboards
CREATE INDEX IF NOT EXISTS idx_users_weight_kg ON users(weight_kg) WHERE weight_kg IS NOT NULL;
