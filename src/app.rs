use axum::{Router, routing::get};
use sqlx::SqlitePool;

use crate::{
    config::AppConfig,
    db::connect_sqlite,
    http::handlers::{
        health::health,
        requests::{create_request, requests_index, show_request},
    },
    metadata::openlibrary::OpenLibraryClient,
};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub open_library: OpenLibraryClient,
}

pub async fn build_app(config: AppConfig) -> anyhow::Result<Router> {
    config.validate()?;
    let pool = connect_sqlite(&config.database).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    let state = AppState {
        pool,
        open_library: OpenLibraryClient::new(config.metadata_base_url),
    };

    Ok(Router::new()
        .route("/health", get(health))
        .route("/requests", get(requests_index).post(create_request))
        .route("/requests/{id}", get(show_request))
        .with_state(state))
}
