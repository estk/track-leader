//! Team generation for test data.

use rand::Rng;
use rand_distr::{Distribution, Poisson};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

/// Team visibility options matching database enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeamVisibility {
    Public,
    Private,
}

impl TeamVisibility {
    /// Returns the database string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            TeamVisibility::Public => "public",
            TeamVisibility::Private => "private",
        }
    }
}

/// Team join policy options matching database enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeamJoinPolicy {
    Open,
    Request,
    Invitation,
}

impl TeamJoinPolicy {
    /// Returns the database string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            TeamJoinPolicy::Open => "open",
            TeamJoinPolicy::Request => "request",
            TeamJoinPolicy::Invitation => "invitation",
        }
    }
}

/// Team role options matching database enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeamRole {
    Owner,
    Admin,
    Member,
}

impl TeamRole {
    /// Returns the database string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            TeamRole::Owner => "owner",
            TeamRole::Admin => "admin",
            TeamRole::Member => "member",
        }
    }
}

/// Generated team data ready for database insertion.
#[derive(Debug, Clone)]
pub struct GeneratedTeam {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub visibility: TeamVisibility,
    pub join_policy: TeamJoinPolicy,
    pub owner_id: Uuid,
    pub created_at: OffsetDateTime,
}

/// Generated team membership data ready for database insertion.
#[derive(Debug, Clone)]
pub struct GeneratedTeamMembership {
    pub team_id: Uuid,
    pub user_id: Uuid,
    pub role: TeamRole,
    pub invited_by: Option<Uuid>,
    pub joined_at: OffsetDateTime,
}

/// Generated activity-team association ready for database insertion.
#[derive(Debug, Clone)]
pub struct GeneratedActivityTeam {
    pub activity_id: Uuid,
    pub team_id: Uuid,
    pub shared_at: OffsetDateTime,
    pub shared_by: Option<Uuid>,
}

/// Generated segment-team association ready for database insertion.
#[derive(Debug, Clone)]
pub struct GeneratedSegmentTeam {
    pub segment_id: Uuid,
    pub team_id: Uuid,
    pub shared_at: OffsetDateTime,
}

/// Configuration for team generation.
#[derive(Debug, Clone)]
pub struct TeamGenConfig {
    /// Probability distribution for team visibility [public, private].
    pub visibility_distribution: (f64, f64),
    /// Probability distribution for join policy [open, request, invitation].
    pub join_policy_distribution: (f64, f64, f64),
    /// Average number of members per team (including owner).
    pub avg_members_per_team: f64,
    /// Probability that a member is an admin (vs regular member).
    pub admin_probability: f64,
    /// Average number of teams each activity is shared with.
    /// Setting this > 1.5 ensures many activities appear in 2+ teams.
    pub avg_teams_per_activity: f64,
    /// Fraction of activities that should be shared to teams (0.0-1.0).
    pub activity_share_fraction: f64,
    /// Fraction of segments that should be shared to teams (0.0-1.0).
    pub segment_share_fraction: f64,
}

impl Default for TeamGenConfig {
    fn default() -> Self {
        Self {
            visibility_distribution: (0.7, 0.3), // 70% public, 30% private
            join_policy_distribution: (0.2, 0.5, 0.3), // 20% open, 50% request, 30% invitation
            avg_members_per_team: 8.0,
            admin_probability: 0.15,
            avg_teams_per_activity: 2.5, // Ensures most shared activities are in 2+ teams
            activity_share_fraction: 0.6,
            segment_share_fraction: 0.4,
        }
    }
}

/// Team name templates.
const TEAM_PREFIXES: &[&str] = &[
    "Boulder", "Mountain", "Valley", "Peak", "Trail", "Summit", "Ridge", "Canyon", "Lake", "River",
    "Forest", "Desert", "Urban", "Metro", "Elite", "Casual", "Weekend", "Morning", "Evening",
    "Night",
];

const TEAM_SUFFIXES: &[&str] = &[
    "Runners Club",
    "Cycling Team",
    "Hikers",
    "Trail Blazers",
    "Athletes",
    "Endurance Club",
    "Fitness Group",
    "Speed Team",
    "Adventure Club",
    "Explorers",
    "Striders",
    "Wheelers",
    "Trekkers",
    "Crew",
    "Squad",
];

const TEAM_DESCRIPTIONS: &[&str] = &[
    "Join us for weekly group rides and runs!",
    "A community of like-minded athletes pushing their limits.",
    "Casual group for outdoor enthusiasts of all skill levels.",
    "Training together to achieve our goals.",
    "Weekend warriors getting out on the trails.",
    "Building a supportive community one mile at a time.",
    "From beginners to pros, everyone is welcome.",
    "Exploring new routes and making new friends.",
    "Competitive athletes seeking training partners.",
    "Adventure seekers unite!",
];

/// Generates teams, memberships, and sharing data.
pub struct TeamGenerator {
    config: TeamGenConfig,
}

impl TeamGenerator {
    /// Creates a new team generator with default configuration.
    pub fn new() -> Self {
        Self {
            config: TeamGenConfig::default(),
        }
    }

    /// Creates a generator with custom configuration.
    pub fn with_config(config: TeamGenConfig) -> Self {
        Self { config }
    }

    /// Generates a batch of teams.
    pub fn generate_teams(
        &self,
        count: usize,
        owner_ids: &[Uuid],
        base_time: OffsetDateTime,
        rng: &mut impl Rng,
    ) -> Vec<GeneratedTeam> {
        if owner_ids.is_empty() {
            return Vec::new();
        }

        (0..count)
            .map(|_| self.generate_single_team(owner_ids, base_time, rng))
            .collect()
    }

    /// Generates a single team.
    fn generate_single_team(
        &self,
        owner_ids: &[Uuid],
        base_time: OffsetDateTime,
        rng: &mut impl Rng,
    ) -> GeneratedTeam {
        let owner_id = owner_ids[rng.gen_range(0..owner_ids.len())];
        let created_at = base_time - Duration::days(rng.gen_range(0..365));

        GeneratedTeam {
            id: Uuid::new_v4(),
            name: self.generate_team_name(rng),
            description: self.generate_description(rng),
            visibility: self.pick_visibility(rng),
            join_policy: self.pick_join_policy(rng),
            owner_id,
            created_at,
        }
    }

    /// Generates a team name.
    fn generate_team_name(&self, rng: &mut impl Rng) -> String {
        let prefix = TEAM_PREFIXES[rng.gen_range(0..TEAM_PREFIXES.len())];
        let suffix = TEAM_SUFFIXES[rng.gen_range(0..TEAM_SUFFIXES.len())];
        format!("{prefix} {suffix}")
    }

    /// Generates a description (sometimes None).
    fn generate_description(&self, rng: &mut impl Rng) -> Option<String> {
        if rng.r#gen::<f64>() < 0.8 {
            Some(TEAM_DESCRIPTIONS[rng.gen_range(0..TEAM_DESCRIPTIONS.len())].to_string())
        } else {
            None
        }
    }

    /// Picks a visibility based on distribution.
    fn pick_visibility(&self, rng: &mut impl Rng) -> TeamVisibility {
        let roll: f64 = rng.r#gen();
        if roll < self.config.visibility_distribution.0 {
            TeamVisibility::Public
        } else {
            TeamVisibility::Private
        }
    }

    /// Picks a join policy based on distribution.
    fn pick_join_policy(&self, rng: &mut impl Rng) -> TeamJoinPolicy {
        let roll: f64 = rng.r#gen();
        let (open, request, _invitation) = self.config.join_policy_distribution;
        if roll < open {
            TeamJoinPolicy::Open
        } else if roll < open + request {
            TeamJoinPolicy::Request
        } else {
            TeamJoinPolicy::Invitation
        }
    }

    /// Generates team memberships for all teams.
    ///
    /// Each team gets a Poisson-distributed number of members (excluding owner,
    /// who is always included). Owner is automatically added with role=Owner.
    pub fn generate_memberships(
        &self,
        teams: &[GeneratedTeam],
        user_ids: &[Uuid],
        rng: &mut impl Rng,
    ) -> Vec<GeneratedTeamMembership> {
        if user_ids.is_empty() {
            return Vec::new();
        }

        let mut memberships = Vec::new();
        let poisson = Poisson::new(self.config.avg_members_per_team - 1.0).unwrap();

        for team in teams {
            // Add owner membership
            memberships.push(GeneratedTeamMembership {
                team_id: team.id,
                user_id: team.owner_id,
                role: TeamRole::Owner,
                invited_by: None,
                joined_at: team.created_at,
            });

            // Generate additional members
            let num_members = poisson.sample(rng) as usize;
            let eligible: Vec<Uuid> = user_ids
                .iter()
                .filter(|&&id| id != team.owner_id)
                .copied()
                .collect();

            let members_to_add = num_members.min(eligible.len());
            let mut added = std::collections::HashSet::new();

            for _ in 0..members_to_add {
                let idx = rng.gen_range(0..eligible.len());
                let user_id = eligible[idx];

                if added.insert(user_id) {
                    let role = if rng.r#gen::<f64>() < self.config.admin_probability {
                        TeamRole::Admin
                    } else {
                        TeamRole::Member
                    };

                    let joined_at = team.created_at + Duration::days(rng.gen_range(0..30));

                    memberships.push(GeneratedTeamMembership {
                        team_id: team.id,
                        user_id,
                        role,
                        invited_by: Some(team.owner_id),
                        joined_at,
                    });
                }
            }
        }

        memberships
    }

    /// Generates activity-team associations.
    ///
    /// Uses Poisson distribution with avg_teams_per_activity to ensure
    /// activities appear in multiple teams.
    pub fn generate_activity_teams(
        &self,
        activity_ids: &[Uuid],
        activity_user_map: &std::collections::HashMap<Uuid, Uuid>,
        teams: &[GeneratedTeam],
        memberships: &[GeneratedTeamMembership],
        rng: &mut impl Rng,
    ) -> Vec<GeneratedActivityTeam> {
        if teams.is_empty() || activity_ids.is_empty() {
            return Vec::new();
        }

        // Build a lookup: user_id -> teams they belong to
        let user_teams: std::collections::HashMap<Uuid, Vec<&GeneratedTeam>> = {
            let mut map: std::collections::HashMap<Uuid, Vec<&GeneratedTeam>> =
                std::collections::HashMap::new();
            for membership in memberships {
                for team in teams {
                    if team.id == membership.team_id {
                        map.entry(membership.user_id).or_default().push(team);
                    }
                }
            }
            map
        };

        let mut activity_teams = Vec::new();
        let poisson = Poisson::new(self.config.avg_teams_per_activity).unwrap();

        for &activity_id in activity_ids {
            // Only share a fraction of activities
            if rng.r#gen::<f64>() > self.config.activity_share_fraction {
                continue;
            }

            // Get the activity owner's teams
            let Some(&user_id) = activity_user_map.get(&activity_id) else {
                continue;
            };
            let Some(user_team_list) = user_teams.get(&user_id) else {
                continue;
            };

            if user_team_list.is_empty() {
                continue;
            }

            // Determine how many teams to share with
            let num_teams = (poisson.sample(rng) as usize)
                .max(1) // At least 1 if we're sharing
                .min(user_team_list.len());

            // Pick random teams
            let mut picked = std::collections::HashSet::new();
            for _ in 0..num_teams {
                let idx = rng.gen_range(0..user_team_list.len());
                if picked.insert(idx) {
                    let team = user_team_list[idx];
                    let shared_at = team.created_at + Duration::days(rng.gen_range(1..60));

                    activity_teams.push(GeneratedActivityTeam {
                        activity_id,
                        team_id: team.id,
                        shared_at,
                        shared_by: Some(user_id),
                    });
                }
            }
        }

        activity_teams
    }

    /// Generates segment-team associations.
    pub fn generate_segment_teams(
        &self,
        segment_ids: &[Uuid],
        teams: &[GeneratedTeam],
        rng: &mut impl Rng,
    ) -> Vec<GeneratedSegmentTeam> {
        if teams.is_empty() || segment_ids.is_empty() {
            return Vec::new();
        }

        let mut segment_teams = Vec::new();
        let poisson = Poisson::new(self.config.avg_teams_per_activity).unwrap();

        for &segment_id in segment_ids {
            // Only share a fraction of segments
            if rng.r#gen::<f64>() > self.config.segment_share_fraction {
                continue;
            }

            // Determine how many teams to share with
            let num_teams = (poisson.sample(rng) as usize).max(1).min(teams.len());

            // Pick random teams
            let mut picked = std::collections::HashSet::new();
            for _ in 0..num_teams {
                let idx = rng.gen_range(0..teams.len());
                if picked.insert(idx) {
                    let team = &teams[idx];
                    let shared_at = team.created_at + Duration::days(rng.gen_range(1..60));

                    segment_teams.push(GeneratedSegmentTeam {
                        segment_id,
                        team_id: team.id,
                        shared_at,
                    });
                }
            }
        }

        segment_teams
    }
}

impl Default for TeamGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_teams() {
        let team_gen = TeamGenerator::new();
        let mut rng = rand::thread_rng();
        let users: Vec<Uuid> = (0..20).map(|_| Uuid::new_v4()).collect();

        let teams = team_gen.generate_teams(5, &users, OffsetDateTime::now_utc(), &mut rng);

        assert_eq!(teams.len(), 5);
        for team in &teams {
            assert!(!team.name.is_empty());
            assert!(users.contains(&team.owner_id));
        }
    }

    #[test]
    fn test_generate_memberships() {
        let team_gen = TeamGenerator::new();
        let mut rng = rand::thread_rng();
        let users: Vec<Uuid> = (0..50).map(|_| Uuid::new_v4()).collect();

        let teams = team_gen.generate_teams(3, &users, OffsetDateTime::now_utc(), &mut rng);
        let memberships = team_gen.generate_memberships(&teams, &users, &mut rng);

        // Each team should have at least the owner
        for team in &teams {
            let team_members: Vec<_> = memberships
                .iter()
                .filter(|m| m.team_id == team.id)
                .collect();
            assert!(!team_members.is_empty());

            // Owner should be present with Owner role
            let owner_membership = team_members.iter().find(|m| m.user_id == team.owner_id);
            assert!(owner_membership.is_some());
            assert!(matches!(owner_membership.unwrap().role, TeamRole::Owner));
        }
    }

    #[test]
    fn test_generate_activity_teams() {
        let team_gen = TeamGenerator::with_config(TeamGenConfig {
            avg_teams_per_activity: 2.5,
            activity_share_fraction: 1.0, // Share all activities for test
            ..Default::default()
        });
        let mut rng = rand::thread_rng();

        let users: Vec<Uuid> = (0..30).map(|_| Uuid::new_v4()).collect();
        let teams = team_gen.generate_teams(5, &users, OffsetDateTime::now_utc(), &mut rng);
        let memberships = team_gen.generate_memberships(&teams, &users, &mut rng);

        // Create activities owned by users who are in teams
        let member_ids: Vec<Uuid> = memberships.iter().map(|m| m.user_id).collect();
        let activities: Vec<Uuid> = (0..20).map(|_| Uuid::new_v4()).collect();
        let activity_user_map: std::collections::HashMap<Uuid, Uuid> = activities
            .iter()
            .enumerate()
            .map(|(i, &aid)| (aid, member_ids[i % member_ids.len()]))
            .collect();

        let activity_teams = team_gen.generate_activity_teams(
            &activities,
            &activity_user_map,
            &teams,
            &memberships,
            &mut rng,
        );

        // Should have generated some activity-team associations
        assert!(!activity_teams.is_empty());

        // Count activities that appear in 2+ teams
        let mut activity_team_counts: std::collections::HashMap<Uuid, usize> =
            std::collections::HashMap::new();
        for at in &activity_teams {
            *activity_team_counts.entry(at.activity_id).or_insert(0) += 1;
        }

        let multi_team_count = activity_team_counts.values().filter(|&&c| c >= 2).count();
        // With avg 2.5 teams per activity, we should have many activities in 2+ teams
        assert!(
            multi_team_count > 0,
            "Should have activities in multiple teams, got {multi_team_count}"
        );
    }

    #[test]
    fn test_visibility_distribution() {
        let team_gen = TeamGenerator::with_config(TeamGenConfig {
            visibility_distribution: (0.9, 0.1), // 90% public
            ..Default::default()
        });
        let mut rng = rand::thread_rng();
        let users: Vec<Uuid> = (0..10).map(|_| Uuid::new_v4()).collect();

        let teams = team_gen.generate_teams(100, &users, OffsetDateTime::now_utc(), &mut rng);

        let public_count = teams
            .iter()
            .filter(|t| matches!(t.visibility, TeamVisibility::Public))
            .count();

        // Should be roughly 90% public (allow some variance)
        assert!(
            public_count > 70 && public_count <= 100,
            "Expected ~90% public, got {public_count}/100"
        );
    }
}
