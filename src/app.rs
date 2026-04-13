use std::{path::Path, time::Duration};

use axum::{
    Router,
    response::Html,
    routing::{get, get_service},
};
use sqlx::SqlitePool;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;

use crate::{
    config::AppConfig,
    db::{connect_sqlite, repositories::SqliteSettingsRepository},
    http::handlers::{
        covers::openlibrary_cover,
        health::health,
        library::{scan_status, trigger_scan},
        prowlarr::{
            delete_indexer, get_indexer, get_indexer_schema, get_indexers, get_system_status,
            post_indexer, test_indexer, update_indexer,
        },
        requests::{
            approve_review_candidate, create_request, reject_review_candidate, requests_index,
            retry_request_search, search_requests, show_request,
        },
        settings::{
            get_acquisition_settings, get_audiobookshelf_settings, get_import_settings,
            get_prowlarr_settings, get_qbittorrent_settings, get_runtime_settings,
            get_storage_settings, list_synced_indexers, test_audiobookshelf_settings,
            test_prowlarr_settings, test_qbittorrent_settings,
            update_acquisition_settings, update_import_settings, update_prowlarr_settings,
            update_audiobookshelf_settings, update_qbittorrent_settings,
            update_runtime_settings, update_storage_settings,
        },
    },
    metadata::openlibrary::OpenLibraryClient,
    workers::{backfill::BackfillWorker, download_worker::DownloadWorker, search_worker::SearchWorker},
};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub settings: SqliteSettingsRepository,
}

impl AppState {
    pub async fn open_library_client(&self) -> anyhow::Result<OpenLibraryClient> {
        let settings = self
            .settings
            .get_persisted_runtime_settings()
            .await?
            .ok_or_else(|| anyhow::anyhow!("runtime settings row missing"))?;

        Ok(OpenLibraryClient::new(
            settings.metadata.base_url,
            settings.metadata.cover_base_url,
        ))
    }
}

pub async fn build_app(config: AppConfig) -> anyhow::Result<Router> {
    config.validate()?;
    let pool = connect_sqlite(&config.database).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    let settings = SqliteSettingsRepository::new(pool.clone());
    let runtime_settings = settings.ensure_seeded(&config).await?;
    let open_library = OpenLibraryClient::new(
        runtime_settings.metadata.base_url,
        runtime_settings.metadata.cover_base_url,
    );

    BackfillWorker::spawn(pool.clone(), open_library.clone());
    if config.enable_fulfillment_workers {
        SearchWorker::spawn(pool.clone(), settings.clone(), Duration::from_secs(15));
        DownloadWorker::spawn(pool.clone(), settings.clone(), Duration::from_secs(10));
    }

    let state = AppState { pool, settings };
    let api_router = Router::new()
        .route("/health", get(health))
        .route("/requests", get(requests_index).post(create_request))
        .route("/requests/search", get(search_requests))
        .route("/requests/{id}", get(show_request))
        .route("/requests/{id}/retry-search", axum::routing::post(retry_request_search))
        .route(
            "/requests/{id}/review-queue/{candidate_id}/approve",
            axum::routing::post(approve_review_candidate),
        )
        .route(
            "/requests/{id}/review-queue/{candidate_id}/reject",
            axum::routing::post(reject_review_candidate),
        )
        .route("/covers/openlibrary/{cover_id}", get(openlibrary_cover))
        .route(
            "/settings/runtime",
            get(get_runtime_settings).put(update_runtime_settings),
        )
        .route(
            "/settings/storage",
            get(get_storage_settings).put(update_storage_settings),
        )
        .route(
            "/settings/import",
            get(get_import_settings).put(update_import_settings),
        )
        .route(
            "/settings/acquisition",
            get(get_acquisition_settings).put(update_acquisition_settings),
        )
        .route(
            "/settings/download-clients/qbittorrent",
            get(get_qbittorrent_settings).put(update_qbittorrent_settings),
        )
        .route(
            "/settings/download-clients/qbittorrent/test",
            axum::routing::post(test_qbittorrent_settings),
        )
        .route(
            "/settings/integrations/prowlarr",
            get(get_prowlarr_settings).put(update_prowlarr_settings),
        )
        .route(
            "/settings/integrations/prowlarr/test",
            axum::routing::post(test_prowlarr_settings),
        )
        .route(
            "/settings/integrations/audiobookshelf",
            get(get_audiobookshelf_settings).put(update_audiobookshelf_settings),
        )
        .route(
            "/settings/integrations/audiobookshelf/test",
            axum::routing::post(test_audiobookshelf_settings),
        )
        .route("/settings/synced-indexers", get(list_synced_indexers))
        .route("/library/scan", axum::routing::post(trigger_scan))
        .route("/library/scan-status", get(scan_status))
        .route("/system/status", get(get_system_status))
        .route("/indexer", get(get_indexers).post(post_indexer))
        .route("/indexer/schema", get(get_indexer_schema))
        .route("/indexer/test", axum::routing::post(test_indexer))
        .route(
            "/indexer/{id}",
            get(get_indexer).put(update_indexer).delete(delete_indexer),
        )
        .with_state(state.clone());
    let app = Router::new().nest("/api/v1", api_router);
    let app = if Path::new("frontend/build/index.html").exists() {
        let frontend_service = get_service(
            ServeDir::new("frontend/build")
                .not_found_service(ServeFile::new("frontend/build/index.html")),
        );
        app.fallback_service(frontend_service)
    } else {
        app.fallback(spa_shell)
    };

    Ok(app
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .with_state(state))
}

async fn spa_shell() -> Html<&'static str> {
    Html(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>Project Athena</title></head><body><div id=\"app\"></div></body></html>",
    )
}
