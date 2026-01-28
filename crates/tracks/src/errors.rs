use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_json::json;
use thiserror::Error;
use tracing::error;

use crate::database::SimilarSegment;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("GPX parsing error: {0}")]
    GpxParsing(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Not found")]
    NotFound,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Internal server error")]
    Internal,

    #[error("Queue error: {0}")]
    Queue(#[from] anyhow::Error),

    #[error("Similar segments already exist")]
    SimilarSegmentsExist(Vec<SimilarSegment>),
}

/// Summary of a similar segment for conflict responses.
#[derive(Debug, Serialize)]
pub struct SimilarSegmentSummary {
    pub id: String,
    pub name: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::SimilarSegmentsExist(similar) => {
                let summaries: Vec<SimilarSegmentSummary> = similar
                    .into_iter()
                    .map(|s| SimilarSegmentSummary {
                        id: s.id.to_string(),
                        name: s.name,
                    })
                    .collect();
                let body = Json(json!({
                    "error": "Similar segments already exist",
                    "similar_segments": summaries,
                }));
                (StatusCode::CONFLICT, body).into_response()
            }
            _ => {
                let (status, error_message) = match &self {
                    AppError::Database(e) => {
                        error!("Database error: {e}");
                        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
                    }
                    AppError::GpxParsing(msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
                    AppError::Io(e) => {
                        error!("IO error: {e}");
                        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
                    }
                    AppError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
                    AppError::NotFound => (StatusCode::NOT_FOUND, "Not found"),
                    AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
                    AppError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden"),
                    AppError::Internal => {
                        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
                    }
                    AppError::Queue(e) => {
                        error!("Queue error: {e}");
                        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
                    }
                    AppError::SimilarSegmentsExist(_) => unreachable!(),
                };

                let body = Json(json!({
                    "error": error_message,
                }));

                (status, body).into_response()
            }
        }
    }
}
