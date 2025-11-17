use axum::Form;
use axum::extract::{FromRequest, Request, State};
use axum::response::IntoResponse;
use sqlx::PgPool;
use sqlx::types::chrono::Utc;
use tracing::{info, instrument};
use uuid::Uuid;

use crate::error::AppError;

#[derive(serde::Deserialize, Debug)]
pub struct SubscribeForm {
    pub name: String,
    pub email: String,
}

impl<S> FromRequest<S> for SubscribeForm
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Form(subscribe_form) = Form::<Self>::from_request(req, state)
            .await
            .map_err(|err| AppError::ValidationError(err.to_string()))?;
        Ok(subscribe_form)
    }
}

#[instrument]
pub async fn subscribe(
    State(pool): State<PgPool>,
    form: SubscribeForm,
) -> Result<impl IntoResponse, AppError> {
    let request_id = Uuid::new_v4().to_string();
    info!(
        request_id = request_id,
        "Adding '{}' '{}' as a new subscriber.", form.email, form.name
    );
    info!(
        request_id = request_id,
        "Saving new subscriber details in the database"
    );
    let _row = sqlx::query!(
        r#"
        insert into subscriptions (id, email, name, subscribed_at)
        values ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now(),
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::DatabaseError(e))?;
    info!(request_id = request_id, "New subscriber have been saved");
    Ok(())
}
