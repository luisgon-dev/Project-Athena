use book_router::{app::build_app, config::AppConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::for_tests();
    let _app = build_app(config).await?;
    Ok(())
}
