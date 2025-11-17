use sqlx::PgPool;
use tracing::subscriber::set_global_default;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use zero2prod::{configuration::get_configuration, startup::run};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .with_span_events(FmtSpan::CLOSE)
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("info".parse().expect("failed to parse directive")),
        )
        .finish();
    set_global_default(subscriber).expect("Failed to set subscriber");
    let settings = get_configuration()?;
    let address = format!("127.0.0.1:{}", settings.application_port);
    let connection_string = settings.database.connection_string();
    let connection_pool = PgPool::connect(&connection_string).await?;
    let serve = run(address, connection_pool).await?;
    serve.await?;
    Ok(())
}
