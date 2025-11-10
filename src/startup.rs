use axum::{
    Router,
    routing::{get, post},
    serve,
};

use crate::routes;

type Serve = serve::Serve<tokio::net::TcpListener, Router, Router>;

pub async fn run(address: String) -> anyhow::Result<Serve> {
    let app = Router::new()
        .route("/health_check", get(routes::health_check))
        .route("/subscriptions", post(routes::subscribe));
    let listener = tokio::net::TcpListener::bind(address).await?;
    let server = axum::serve(listener, app);
    Ok(server)
}
