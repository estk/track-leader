//! Pagination helpers and types.

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Default pagination limit.
pub const DEFAULT_LIMIT: i64 = 50;

/// Returns the default pagination limit.
pub fn default_limit() -> i64 {
    DEFAULT_LIMIT
}

/// Standard pagination query parameters.
#[derive(Debug, Clone, Deserialize, ToSchema, IntoParams)]
pub struct PaginationQuery {
    /// Maximum number of results to return.
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// Number of results to skip.
    #[serde(default)]
    pub offset: i64,
}

impl Default for PaginationQuery {
    fn default() -> Self {
        Self {
            limit: DEFAULT_LIMIT,
            offset: 0,
        }
    }
}

/// Paginated response wrapper.
#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total_count: i64,
    pub limit: i64,
    pub offset: i64,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total_count: i64, limit: i64, offset: i64) -> Self {
        Self {
            items,
            total_count,
            limit,
            offset,
        }
    }
}
