use axum::{Router, routing::get};

use crate::{
    config::AppConfig,
    db::connect_sqlite,
    http::handlers::{
        health::health,
        requests::{create_request, requests_index, show_request},
    },
};

pub async fn build_app(config: AppConfig) -> anyhow::Result<Router> {
    config.validate()?;
    let pool = connect_sqlite(&config.database).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(Router::new()
        .route("/health", get(health))
        .route("/requests", get(requests_index).post(create_request))
        .route("/requests/{id}", get(show_request))
        .with_state(pool))
}
