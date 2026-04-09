use axum::serve;
use book_router::{app::build_app, config::AppConfig};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::from_env()?;
    let listener = TcpListener::bind(&config.bind_addr).await?;
    let app = build_app(config)?;

    serve(listener, app).await?;
    Ok(())
}
