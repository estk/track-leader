//! Reusable SQL query builder for dynamic WHERE clauses.
//!
//! This module provides a builder pattern for constructing SQL queries with
//! dynamic conditions while tracking parameter indices for safe binding.

use time::Date;
use uuid::Uuid;

use crate::models::DateRangeFilter;

/// Builder for constructing SQL WHERE clauses with parameter tracking.
///
/// # Example
/// ```ignore
/// let mut qb = QueryBuilder::new();
/// qb.add_condition("deleted_at IS NULL".into());
/// if let Some(type_id) = activity_type_id {
///     qb.add_param_condition("activity_type_id = ");
/// }
/// let where_clause = qb.build_where();
/// ```
#[derive(Debug, Default)]
pub struct QueryBuilder {
    conditions: Vec<String>,
    param_idx: usize,
}

impl QueryBuilder {
    /// Creates a new empty query builder.
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
            param_idx: 1,
        }
    }

    /// Adds a static condition (no parameter binding).
    pub fn add_condition(&mut self, condition: impl Into<String>) -> &mut Self {
        self.conditions.push(condition.into());
        self
    }

    /// Adds a condition with a parameter placeholder, incrementing the param index.
    /// Returns the parameter index that was used.
    pub fn add_param_condition(&mut self, condition_prefix: &str) -> usize {
        let idx = self.param_idx;
        self.conditions.push(format!("{condition_prefix}${idx}"));
        self.param_idx += 1;
        idx
    }

    /// Adds an optional value as a condition if it's Some.
    /// The condition_fn receives the current param index and should return the condition string.
    pub fn add_optional<T, F>(&mut self, value: &Option<T>, condition_fn: F) -> &mut Self
    where
        F: FnOnce(usize) -> String,
    {
        if value.is_some() {
            let condition = condition_fn(self.param_idx);
            self.conditions.push(condition);
            self.param_idx += 1;
        }
        self
    }

    /// Adds a date range filter condition.
    pub fn add_date_range(
        &mut self,
        filter: DateRangeFilter,
        column: &str,
        start_date: &Option<Date>,
        end_date: &Option<Date>,
    ) -> &mut Self {
        // First check for preset range
        if let Some(condition) = filter.to_sql_condition(column) {
            self.conditions.push(condition);
        } else if filter == DateRangeFilter::Custom {
            // Handle custom date range with start_date and end_date params
            if start_date.is_some() {
                let idx = self.param_idx;
                self.conditions.push(format!("{column} >= ${idx}::date"));
                self.param_idx += 1;
            }
            if end_date.is_some() {
                let idx = self.param_idx;
                // Add 1 day to include the full end date
                self.conditions
                    .push(format!("{column} < ${idx}::date + INTERVAL '1 day'"));
                self.param_idx += 1;
            }
        }
        self
    }

    /// Returns the current parameter index (for binding).
    pub fn current_param_idx(&self) -> usize {
        self.param_idx
    }

    /// Increments and returns the next parameter index.
    pub fn next_param_idx(&mut self) -> usize {
        let idx = self.param_idx;
        self.param_idx += 1;
        idx
    }

    /// Returns the number of conditions added.
    pub fn condition_count(&self) -> usize {
        self.conditions.len()
    }

    /// Returns true if no conditions have been added.
    pub fn is_empty(&self) -> bool {
        self.conditions.is_empty()
    }

    /// Builds the WHERE clause string (without the "WHERE" keyword).
    /// Returns an empty string if no conditions were added.
    pub fn build_where(&self) -> String {
        self.conditions.join(" AND ")
    }

    /// Builds the full WHERE clause including the "WHERE" keyword.
    /// Returns "WHERE 1=1" if no conditions (always true).
    pub fn build_where_clause(&self) -> String {
        if self.conditions.is_empty() {
            "WHERE 1=1".to_string()
        } else {
            format!("WHERE {}", self.conditions.join(" AND "))
        }
    }
}

/// Helper trait for binding optional values to sqlx queries.
pub trait BindOptional<'q, DB: sqlx::Database> {
    /// Binds an optional UUID value if present.
    fn bind_optional_uuid(self, value: Option<Uuid>) -> Self;
    /// Binds an optional string value if present.
    fn bind_optional_str(self, value: Option<&str>) -> Self;
    /// Binds an optional date value if present.
    fn bind_optional_date(self, value: Option<Date>) -> Self;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_builder() {
        let qb = QueryBuilder::new();
        assert!(qb.is_empty());
        assert_eq!(qb.build_where(), "");
        assert_eq!(qb.build_where_clause(), "WHERE 1=1");
    }

    #[test]
    fn test_single_condition() {
        let mut qb = QueryBuilder::new();
        qb.add_condition("deleted_at IS NULL");
        assert_eq!(qb.build_where(), "deleted_at IS NULL");
    }

    #[test]
    fn test_multiple_conditions() {
        let mut qb = QueryBuilder::new();
        qb.add_condition("deleted_at IS NULL");
        qb.add_condition("visibility = 'public'");
        assert_eq!(
            qb.build_where(),
            "deleted_at IS NULL AND visibility = 'public'"
        );
    }

    #[test]
    fn test_param_condition() {
        let mut qb = QueryBuilder::new();
        qb.add_condition("deleted_at IS NULL");
        let idx = qb.add_param_condition("activity_type_id = ");
        assert_eq!(idx, 1);
        assert_eq!(
            qb.build_where(),
            "deleted_at IS NULL AND activity_type_id = $1"
        );
    }

    #[test]
    fn test_optional_some() {
        let mut qb = QueryBuilder::new();
        let type_id: Option<Uuid> = Some(Uuid::nil());
        qb.add_optional(&type_id, |idx| format!("activity_type_id = ${idx}"));
        assert_eq!(qb.build_where(), "activity_type_id = $1");
    }

    #[test]
    fn test_optional_none() {
        let mut qb = QueryBuilder::new();
        let type_id: Option<Uuid> = None;
        qb.add_optional(&type_id, |idx| format!("activity_type_id = ${idx}"));
        assert!(qb.is_empty());
    }

    #[test]
    fn test_date_range_week() {
        let mut qb = QueryBuilder::new();
        qb.add_date_range(DateRangeFilter::Week, "submitted_at", &None, &None);
        assert_eq!(
            qb.build_where(),
            "submitted_at >= NOW() - INTERVAL '7 days'"
        );
    }

    #[test]
    fn test_date_range_all() {
        let mut qb = QueryBuilder::new();
        qb.add_date_range(DateRangeFilter::All, "submitted_at", &None, &None);
        assert!(qb.is_empty());
    }
}
