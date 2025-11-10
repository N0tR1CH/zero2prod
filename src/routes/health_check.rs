use axum::response::IntoResponse;
use reqwest::StatusCode;

pub async fn health_check() -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}
