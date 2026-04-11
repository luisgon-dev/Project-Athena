use axum::{Router, routing::get};
use sqlx::SqlitePool;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;

use crate::{
    config::AppConfig,
    db::connect_sqlite,
    http::handlers::{
        covers::openlibrary_cover,
        health::health,
        requests::{create_request, requests_index, search_requests, show_request},
    },
    metadata::openlibrary::OpenLibraryClient,
    workers::backfill::BackfillWorker,
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
    let open_library = OpenLibraryClient::new(config.metadata_base_url, config.cover_base_url);
    
    BackfillWorker::spawn(pool.clone(), open_library.clone());
    
    let state = AppState { pool, open_library };
    let api_router = Router::new()
        .route("/health", get(health))
        .route("/requests", get(requests_index).post(create_request))
        .route("/requests/search", get(search_requests))
        .route("/requests/{id}", get(show_request))
        .route("/covers/openlibrary/{cover_id}", get(openlibrary_cover))
        .with_state(state.clone());
    let frontend_service =
        ServeDir::new("frontend/build").not_found_service(ServeFile::new("frontend/build/index.html"));

    Ok(Router::new()
        .nest("/api/v1", api_router)
        .fallback_service(frontend_service)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .with_state(state))
}
