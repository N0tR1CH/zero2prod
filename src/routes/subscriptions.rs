use axum::Form;
use axum::extract::rejection::FormRejection;
use axum::response::IntoResponse;
use reqwest::StatusCode;

#[derive(serde::Deserialize)]
pub struct SubscribeForm {
    name: String,
    email: String,
}

pub async fn subscribe(
    form: Result<Form<SubscribeForm>, FormRejection>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if let Ok(Form(form)) = form {
        Ok(format!("name = {} email = {}", form.name, form.email))
    } else {
        Err((
            StatusCode::BAD_REQUEST,
            "name or email is not given".to_string(),
        ))
    }
}
