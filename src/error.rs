use axum::{Json, response::IntoResponse};
use reqwest::StatusCode;
use serde_json::json;
use tracing::{error, instrument, warn};

pub enum AppError {
    DatabaseError(sqlx::Error),
    ValidationError(String),
    NotFound(String),
}

impl From<sqlx::Error> for AppError {
    #[instrument(level = "error", skip(err))]
    fn from(err: sqlx::Error) -> Self {
        AppError::DatabaseError(err)
    }
}

impl IntoResponse for AppError {
    #[instrument(level = "info", skip(self))]
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            AppError::DatabaseError(_db_error) => {
                // The actual db_error is now logged in the From<sqlx::Error> impl
                // We keep a generic message for the user.
                error!("Returning database error response to user.");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            AppError::ValidationError(msg) => {
                warn!("Validation error: {}", msg);
                (StatusCode::BAD_REQUEST, msg)
            }
            AppError::NotFound(msg) => {
                warn!("Not Found error: {}", msg);
                (StatusCode::NOT_FOUND, msg)
            }
        };
        error!(
            status = %status,
            error_message = %error_message,
            "Sending error response"
        );
        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}
