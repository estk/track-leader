//! Social interaction generation (follows, kudos, comments).

use rand::Rng;
use rand_distr::{Distribution, Poisson};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

/// Generated follow relationship.
#[derive(Debug, Clone)]
pub struct GeneratedFollow {
    pub follower_id: Uuid,
    pub following_id: Uuid,
    pub created_at: OffsetDateTime,
}

/// Generated kudos (like) on an activity.
#[derive(Debug, Clone)]
pub struct GeneratedKudos {
    pub user_id: Uuid,
    pub activity_id: Uuid,
    pub created_at: OffsetDateTime,
}

/// Generated comment on an activity.
#[derive(Debug, Clone)]
pub struct GeneratedComment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub activity_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub content: String,
    pub created_at: OffsetDateTime,
}

/// Configuration for social graph generation.
#[derive(Debug, Clone)]
pub struct SocialGenConfig {
    /// Average number of follows per user.
    pub avg_follows_per_user: f64,
    /// Probability of mutual follow (if A follows B, probability B follows A).
    pub mutual_follow_probability: f64,
    /// Average kudos per activity.
    pub avg_kudos_per_activity: f64,
    /// Average comments per activity.
    pub avg_comments_per_activity: f64,
    /// Probability that a comment is a reply to another comment.
    pub reply_probability: f64,
}

impl Default for SocialGenConfig {
    fn default() -> Self {
        Self {
            avg_follows_per_user: 15.0,
            mutual_follow_probability: 0.6,
            avg_kudos_per_activity: 5.0,
            avg_comments_per_activity: 1.5,
            reply_probability: 0.3,
        }
    }
}

/// Generates social interactions.
pub struct SocialGenerator {
    config: SocialGenConfig,
    comment_templates: Vec<String>,
}

impl SocialGenerator {
    /// Creates a new social generator with default configuration.
    pub fn new() -> Self {
        Self {
            config: SocialGenConfig::default(),
            comment_templates: default_comment_templates(),
        }
    }

    /// Creates a generator with custom configuration.
    pub fn with_config(config: SocialGenConfig) -> Self {
        Self {
            config,
            comment_templates: default_comment_templates(),
        }
    }

    /// Generates a social follow graph for a set of users.
    ///
    /// Creates a realistic follow network where:
    /// - Some users are more popular (followed by many)
    /// - Mutual follows are common
    /// - Users don't follow themselves
    pub fn generate_follow_graph(
        &self,
        user_ids: &[Uuid],
        base_time: OffsetDateTime,
        rng: &mut impl Rng,
    ) -> Vec<GeneratedFollow> {
        if user_ids.len() < 2 {
            return Vec::new();
        }

        let mut follows = Vec::new();
        let poisson = Poisson::new(self.config.avg_follows_per_user).unwrap();

        for (idx, &follower_id) in user_ids.iter().enumerate() {
            let num_follows = (poisson.sample(rng) as usize).min(user_ids.len() - 1);

            // Pick random users to follow (excluding self)
            let mut available: Vec<usize> = (0..user_ids.len()).filter(|&i| i != idx).collect();
            let to_follow = available.drain(..).take(num_follows).collect::<Vec<_>>();

            for follow_idx in to_follow {
                let following_id = user_ids[follow_idx];
                let created_at = base_time - Duration::days(rng.gen_range(0..365));

                follows.push(GeneratedFollow {
                    follower_id,
                    following_id,
                    created_at,
                });

                // Maybe add mutual follow
                if rng.r#gen::<f64>() < self.config.mutual_follow_probability {
                    follows.push(GeneratedFollow {
                        follower_id: following_id,
                        following_id: follower_id,
                        created_at: created_at + Duration::hours(rng.gen_range(1..48)),
                    });
                }
            }
        }

        // Deduplicate follows (mutual follows may create duplicates)
        let mut seen = std::collections::HashSet::new();
        follows.retain(|f| seen.insert((f.follower_id, f.following_id)));

        follows
    }

    /// Generates kudos for an activity.
    pub fn generate_kudos(
        &self,
        activity_id: Uuid,
        activity_owner_id: Uuid,
        potential_givers: &[Uuid],
        activity_time: OffsetDateTime,
        rng: &mut impl Rng,
    ) -> Vec<GeneratedKudos> {
        let poisson = Poisson::new(self.config.avg_kudos_per_activity).unwrap();
        let num_kudos = (poisson.sample(rng) as usize).min(potential_givers.len());

        // Filter out the activity owner
        let eligible: Vec<Uuid> = potential_givers
            .iter()
            .filter(|&&id| id != activity_owner_id)
            .copied()
            .collect();

        eligible
            .into_iter()
            .take(num_kudos)
            .map(|user_id| {
                let created_at = activity_time + Duration::hours(rng.gen_range(0..72));
                GeneratedKudos {
                    user_id,
                    activity_id,
                    created_at,
                }
            })
            .collect()
    }

    /// Generates comments for an activity.
    pub fn generate_comments(
        &self,
        activity_id: Uuid,
        activity_owner_id: Uuid,
        potential_commenters: &[Uuid],
        activity_time: OffsetDateTime,
        rng: &mut impl Rng,
    ) -> Vec<GeneratedComment> {
        let poisson = Poisson::new(self.config.avg_comments_per_activity).unwrap();
        let num_comments = (poisson.sample(rng) as usize).min(potential_commenters.len());

        if num_comments == 0 {
            return Vec::new();
        }

        let mut comments = Vec::new();
        let mut comment_ids: Vec<Uuid> = Vec::new();

        for i in 0..num_comments {
            // Pick a commenter (can include owner responding to comments)
            let user_id = if i == 0 || rng.r#gen::<f64>() > 0.3 {
                // Most comments from others
                let idx = rng.gen_range(0..potential_commenters.len());
                potential_commenters[idx]
            } else {
                // Owner sometimes responds
                activity_owner_id
            };

            // Determine if this is a reply
            let parent_id =
                if !comment_ids.is_empty() && rng.r#gen::<f64>() < self.config.reply_probability {
                    Some(comment_ids[rng.gen_range(0..comment_ids.len())])
                } else {
                    None
                };

            let content = self.generate_comment_text(parent_id.is_some(), rng);
            let created_at = activity_time
                + Duration::hours(rng.gen_range(0..48))
                + Duration::minutes(rng.gen_range(0..60) * i as i64);

            let id = Uuid::new_v4();
            comment_ids.push(id);

            comments.push(GeneratedComment {
                id,
                user_id,
                activity_id,
                parent_id,
                content,
                created_at,
            });
        }

        comments
    }

    /// Generates realistic comment text.
    fn generate_comment_text(&self, is_reply: bool, rng: &mut impl Rng) -> String {
        if is_reply {
            let reply_templates = [
                "Thanks!",
                "Appreciate it!",
                "You too!",
                "Next time for sure.",
                "Let's do it again soon!",
                "Great suggestion, will try that.",
            ];
            let idx = rng.gen_range(0..reply_templates.len());
            reply_templates[idx].to_string()
        } else {
            let idx = rng.gen_range(0..self.comment_templates.len());
            self.comment_templates[idx].clone()
        }
    }
}

impl Default for SocialGenerator {
    fn default() -> Self {
        Self::new()
    }
}

fn default_comment_templates() -> Vec<String> {
    vec![
        "Great effort!".into(),
        "Strong finish! 💪".into(),
        "Nice pace!".into(),
        "Looking fast out there.".into(),
        "That's a tough climb, well done.".into(),
        "Beautiful route!".into(),
        "Jealous of that weather!".into(),
        "Keep it up!".into(),
        "We should ride together sometime.".into(),
        "What was the trail like?".into(),
        "That elevation gain though 🏔️".into(),
        "PR incoming!".into(),
        "Consistent pacing, nice work.".into(),
        "Great to see you out there!".into(),
        "That descent looks fun.".into(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_follow_graph() {
        let social_gen = SocialGenerator::new();
        let mut rng = rand::thread_rng();

        let users: Vec<Uuid> = (0..20).map(|_| Uuid::new_v4()).collect();
        let follows = social_gen.generate_follow_graph(&users, OffsetDateTime::now_utc(), &mut rng);

        assert!(!follows.is_empty());

        // No self-follows
        for f in &follows {
            assert_ne!(f.follower_id, f.following_id);
        }
    }

    #[test]
    fn test_kudos_generation() {
        let social_gen = SocialGenerator::new();
        let mut rng = rand::thread_rng();

        let activity_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let users: Vec<Uuid> = (0..10).map(|_| Uuid::new_v4()).collect();

        let kudos = social_gen.generate_kudos(
            activity_id,
            owner_id,
            &users,
            OffsetDateTime::now_utc(),
            &mut rng,
        );

        // Owner shouldn't give kudos to their own activity
        for k in &kudos {
            assert_ne!(k.user_id, owner_id);
        }
    }

    #[test]
    fn test_comments_with_replies() {
        let social_gen = SocialGenerator::with_config(SocialGenConfig {
            avg_comments_per_activity: 5.0,
            reply_probability: 0.5,
            ..Default::default()
        });
        let mut rng = rand::thread_rng();

        let activity_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let users: Vec<Uuid> = (0..10).map(|_| Uuid::new_v4()).collect();

        let comments = social_gen.generate_comments(
            activity_id,
            owner_id,
            &users,
            OffsetDateTime::now_utc(),
            &mut rng,
        );

        // Should have some comments
        assert!(!comments.is_empty());

        // All comments should have content
        for c in &comments {
            assert!(!c.content.is_empty());
        }
    }
}
