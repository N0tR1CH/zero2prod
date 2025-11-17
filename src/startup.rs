use axum::{
    Router,
    routing::{get, post},
    serve,
};
use sqlx::PgPool;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::routes;

type Serve = serve::Serve<tokio::net::TcpListener, Router, Router>;

pub async fn run(address: String, connection_pool: PgPool) -> anyhow::Result<Serve> {
    info!("Starting server");

    let app = Router::new()
        .route("/health_check", get(routes::health_check))
        .route("/subscriptions", post(routes::subscribe))
        .with_state(connection_pool.clone())
        .layer(TraceLayer::new_for_http());
    let listener = tokio::net::TcpListener::bind(address).await?;

    info!("listening on {}", listener.local_addr().unwrap());
    let server = axum::serve(listener, app);
    Ok(server)
}
