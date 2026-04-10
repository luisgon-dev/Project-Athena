use axum::serve;
use book_router::{app::build_app, config::AppConfig};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,tower_http=info")),
        )
        .init();

    let config = AppConfig::from_env()?;
    info!(bind_addr = %config.bind_addr, "starting book_router");
    let listener = TcpListener::bind(&config.bind_addr).await?;
    let app = build_app(config).await?;

    info!("book_router listening");
    serve(listener, app).await?;
    Ok(())
}
