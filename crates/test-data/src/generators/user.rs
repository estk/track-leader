//! User generation with demographics.

use fake::{Fake, faker::name::en::Name};
use rand::Rng;
use rand_distr::{Distribution, Normal};
use uuid::Uuid;

use tracks::models::Gender;

/// Generated user data ready for database insertion.
#[derive(Debug, Clone)]
pub struct GeneratedUser {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub gender: Option<Gender>,
    pub birth_year: Option<i32>,
    pub weight_kg: Option<f64>,
    pub country: Option<String>,
    pub region: Option<String>,
}

/// Configuration for user generation.
#[derive(Debug, Clone)]
pub struct UserGenConfig {
    /// Distribution of genders (male, female, other, prefer_not_to_say).
    pub gender_distribution: [f64; 4],
    /// Mean birth year.
    pub birth_year_mean: i32,
    /// Standard deviation of birth year.
    pub birth_year_std: f64,
    /// Probability that demographics are filled in.
    pub demographics_fill_rate: f64,
    /// Default country.
    pub default_country: String,
    /// Possible regions within the country.
    pub regions: Vec<String>,
}

impl Default for UserGenConfig {
    fn default() -> Self {
        Self {
            // Approximate real-world fitness app demographics
            gender_distribution: [0.55, 0.35, 0.02, 0.08], // M, F, Other, PrefNot
            birth_year_mean: 1985,
            birth_year_std: 12.0,
            demographics_fill_rate: 0.7,
            default_country: "US".to_string(),
            regions: vec![
                "CO".to_string(),
                "NV".to_string(),
                "CA".to_string(),
                "UT".to_string(),
            ],
        }
    }
}

/// Generates realistic user data for testing.
pub struct UserGenerator {
    config: UserGenConfig,
}

impl UserGenerator {
    /// Creates a new user generator with default configuration.
    pub fn new() -> Self {
        Self {
            config: UserGenConfig::default(),
        }
    }

    /// Creates a generator with custom configuration.
    pub fn with_config(config: UserGenConfig) -> Self {
        Self { config }
    }

    /// Generates a single user.
    pub fn generate(&self, rng: &mut impl Rng) -> GeneratedUser {
        let id = Uuid::new_v4();
        let name: String = Name().fake_with_rng(rng);
        let email = self.generate_email(&name, rng);

        // Hash using the same algorithm the auth system uses
        let password_hash =
            tracks::auth::hash_password("tracks.rs").expect("Failed to hash password");

        let (gender, birth_year, weight_kg, country, region) =
            if rng.r#gen::<f64>() < self.config.demographics_fill_rate {
                (
                    Some(self.generate_gender(rng)),
                    Some(self.generate_birth_year(rng)),
                    Some(self.generate_weight(rng)),
                    Some(self.config.default_country.clone()),
                    self.config
                        .regions
                        .get(rng.gen_range(0..self.config.regions.len()))
                        .cloned(),
                )
            } else {
                (None, None, None, None, None)
            };

        GeneratedUser {
            id,
            name,
            email,
            password_hash,
            gender,
            birth_year,
            weight_kg,
            country,
            region,
        }
    }

    /// Generates multiple users.
    pub fn generate_batch(&self, count: usize, rng: &mut impl Rng) -> Vec<GeneratedUser> {
        (0..count).map(|_| self.generate(rng)).collect()
    }

    /// Generates an email from a name.
    fn generate_email(&self, name: &str, rng: &mut impl Rng) -> String {
        let normalized: String = name
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == ' ')
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(".");

        let suffix: u32 = rng.gen_range(1..9999);
        let domains = ["gmail.com", "outlook.com", "yahoo.com", "proton.me"];
        let domain = domains[rng.gen_range(0..domains.len())];

        format!("{normalized}{suffix}@{domain}")
    }

    /// Generates a gender based on configured distribution.
    fn generate_gender(&self, rng: &mut impl Rng) -> Gender {
        let roll: f64 = rng.r#gen();
        let mut cumulative = 0.0;

        for (i, &weight) in self.config.gender_distribution.iter().enumerate() {
            cumulative += weight;
            if roll < cumulative {
                return match i {
                    0 => Gender::Male,
                    1 => Gender::Female,
                    2 => Gender::Other,
                    _ => Gender::PreferNotToSay,
                };
            }
        }

        Gender::PreferNotToSay
    }

    /// Generates a birth year based on configured distribution.
    fn generate_birth_year(&self, rng: &mut impl Rng) -> i32 {
        let normal = Normal::new(
            self.config.birth_year_mean as f64,
            self.config.birth_year_std,
        )
        .unwrap();

        let year = normal.sample(rng) as i32;
        // Clamp to reasonable range (18-80 years old in ~2024)
        year.clamp(1944, 2006)
    }

    /// Generates a weight based on gender.
    fn generate_weight(&self, rng: &mut impl Rng) -> f64 {
        // Fitness app users tend to be in shape
        let (mean, std_dev) = (70.0, 12.0);
        let normal = Normal::new(mean, std_dev).unwrap();
        let weight: f64 = normal.sample(rng);
        weight.clamp(45.0, 120.0)
    }
}

impl Default for UserGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_user() {
        let user_gen = UserGenerator::new();
        let mut rng = rand::thread_rng();
        let user = user_gen.generate(&mut rng);

        assert!(!user.name.is_empty());
        assert!(user.email.contains('@'));
        assert!(!user.password_hash.is_empty());
    }

    #[test]
    fn test_generate_batch() {
        let user_gen = UserGenerator::new();
        let mut rng = rand::thread_rng();
        let users = user_gen.generate_batch(10, &mut rng);

        assert_eq!(users.len(), 10);

        // All UUIDs should be unique
        let ids: std::collections::HashSet<_> = users.iter().map(|u| u.id).collect();
        assert_eq!(ids.len(), 10);
    }
}
