-- Performance indexes for teams feature
-- Separated for easy future tuning

-- Teams: User's teams list (sorted by most recent)
CREATE INDEX IF NOT EXISTS idx_teams_owner_created ON teams(owner_id, created_at DESC) WHERE deleted_at IS NULL;

-- Team memberships: User's teams lookup with join date
CREATE INDEX IF NOT EXISTS idx_team_memberships_user_joined ON team_memberships(user_id, joined_at DESC);

-- Team memberships: Team members by role (for admin lookups)
CREATE INDEX IF NOT EXISTS idx_team_memberships_team_role ON team_memberships(team_id, role);

-- Activity teams: Team's activities (for team activity feed)
CREATE INDEX IF NOT EXISTS idx_activity_teams_team_shared ON activity_teams(team_id, shared_at DESC);

-- Segment teams: Team's segments (for team segment list)
CREATE INDEX IF NOT EXISTS idx_segment_teams_team_shared ON segment_teams(team_id, shared_at DESC);

-- Team join requests: Pending requests for a team (for admin review)
CREATE INDEX IF NOT EXISTS idx_team_join_requests_pending ON team_join_requests(team_id, created_at DESC) WHERE status = 'pending';

-- Team invitations: Active invitations expiry check
CREATE INDEX IF NOT EXISTS idx_team_invitations_expires ON team_invitations(expires_at) WHERE accepted_at IS NULL;
