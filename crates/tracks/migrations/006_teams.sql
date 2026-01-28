-- Teams: Group-based access control for activities and segments

-- Team role enum
CREATE TYPE team_role AS ENUM ('owner', 'admin', 'member');

-- Team visibility enum (whether team is discoverable)
CREATE TYPE team_visibility AS ENUM ('public', 'private');

-- Team join policy enum
CREATE TYPE team_join_policy AS ENUM ('open', 'request', 'invitation');

-- Teams table
CREATE TABLE teams (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT,
    avatar_url TEXT,
    visibility team_visibility NOT NULL DEFAULT 'private',
    join_policy team_join_policy NOT NULL DEFAULT 'invitation',
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Denormalized counts for performance
    member_count INTEGER NOT NULL DEFAULT 1,
    activity_count INTEGER NOT NULL DEFAULT 0,
    segment_count INTEGER NOT NULL DEFAULT 0,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_teams_owner ON teams(owner_id);
CREATE INDEX idx_teams_visibility ON teams(visibility) WHERE deleted_at IS NULL;

-- Team memberships (many-to-many users <-> teams)
CREATE TABLE team_memberships (
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role team_role NOT NULL DEFAULT 'member',
    invited_by UUID REFERENCES users(id) ON DELETE SET NULL,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (team_id, user_id)
);

CREATE INDEX idx_team_memberships_user ON team_memberships(user_id);
CREATE INDEX idx_team_memberships_team ON team_memberships(team_id);

-- Team join requests (for request-based joining)
CREATE TABLE team_join_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    message TEXT,
    status TEXT NOT NULL DEFAULT 'pending',  -- 'pending', 'approved', 'rejected'
    reviewed_by UUID REFERENCES users(id) ON DELETE SET NULL,
    reviewed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (team_id, user_id)
);

CREATE INDEX idx_team_join_requests_team ON team_join_requests(team_id) WHERE status = 'pending';
CREATE INDEX idx_team_join_requests_user ON team_join_requests(user_id);

-- Team invitations (for email-based invites)
CREATE TABLE team_invitations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    email TEXT NOT NULL,
    invited_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role team_role NOT NULL DEFAULT 'member',
    token TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    accepted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (team_id, email)
);

CREATE INDEX idx_team_invitations_token ON team_invitations(token) WHERE accepted_at IS NULL;
CREATE INDEX idx_team_invitations_team ON team_invitations(team_id) WHERE accepted_at IS NULL;
CREATE INDEX idx_team_invitations_email ON team_invitations(email) WHERE accepted_at IS NULL;

-- Activity-team junction table (many-to-many)
CREATE TABLE activity_teams (
    activity_id UUID NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    shared_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    shared_by UUID REFERENCES users(id) ON DELETE SET NULL,
    PRIMARY KEY (activity_id, team_id)
);

CREATE INDEX idx_activity_teams_team ON activity_teams(team_id);
CREATE INDEX idx_activity_teams_activity ON activity_teams(activity_id);

-- Segment-team junction table (many-to-many)
CREATE TABLE segment_teams (
    segment_id UUID NOT NULL REFERENCES segments(id) ON DELETE CASCADE,
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    shared_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (segment_id, team_id)
);

CREATE INDEX idx_segment_teams_team ON segment_teams(team_id);
CREATE INDEX idx_segment_teams_segment ON segment_teams(segment_id);
