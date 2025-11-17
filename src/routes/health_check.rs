use axum::response::IntoResponse;
use reqwest::StatusCode;
use tracing::instrument;

#[instrument]
pub async fn health_check() -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}
