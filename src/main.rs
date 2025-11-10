use zero2prod::{configuration::get_configuration, startup::run};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = get_configuration()?;
    let address = format!("127.0.0.1:{}", settings.application_port);
    let serve = run(address).await?;
    serve.await?;
    Ok(())
}
