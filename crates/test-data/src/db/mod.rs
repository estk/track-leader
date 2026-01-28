//! Database integration for seeding test data.
//!
//! The [`Seeder`] provides methods for inserting generated test data
//! into the database, with support for bulk operations and progress reporting.

mod seeder;

pub use seeder::{SeedError, Seeder};
