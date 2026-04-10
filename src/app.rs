use anyhow::Context;
use axum::{Router, routing::get};
use serde_json::json;
use sqlx::Row;
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
    let open_library = OpenLibraryClient::new(config.metadata_base_url);
    backfill_legacy_request_work_ids(&pool, &open_library).await?;
    let state = AppState {
        pool,
        open_library,
    };

    Ok(Router::new()
        .route("/health", get(health))
        .route("/requests", get(requests_index).post(create_request))
        .route("/requests/{id}", get(show_request))
        .with_state(state))
}

async fn backfill_legacy_request_work_ids(
    pool: &SqlitePool,
    open_library: &OpenLibraryClient,
) -> anyhow::Result<()> {
    let legacy_rows = sqlx::query(
        "SELECT id, title, author
         FROM requests
         WHERE external_work_id = ''",
    )
    .fetch_all(pool)
    .await?;

    for row in legacy_rows {
        let request_id = row.get::<String, _>("id");
        let title = row.get::<String, _>("title");
        let author = row.get::<String, _>("author");
        if let Err(error) =
            backfill_request_work_identity(pool, open_library, &request_id, &title, &author).await
        {
            eprintln!("legacy request backfill skipped for {request_id}: {error:#}");
        }
    }

    Ok(())
}

async fn backfill_request_work_identity(
    pool: &SqlitePool,
    open_library: &OpenLibraryClient,
    request_id: &str,
    title: &str,
    author: &str,
) -> anyhow::Result<()> {
    let resolved = open_library
        .resolve_work(title, author)
        .await
        .with_context(|| format!("failed to resolve canonical work for request {request_id}"))?;
    let external_work_id = resolved.work.external_id.clone();
    let resolved_title = resolved.work.title.clone();
    let resolved_author = resolved.work.primary_author.clone();

    let event_row = sqlx::query(
        "SELECT id, payload_json
         FROM request_events
         WHERE request_id = ?
         ORDER BY id ASC
         LIMIT 1",
    )
    .bind(request_id)
    .fetch_one(pool)
    .await
    .with_context(|| format!("missing request.created event for request {request_id}"))?;
    let event_id = event_row.get::<i64, _>("id");
    let payload_json = event_row.get::<String, _>("payload_json");
    let mut payload: serde_json::Value = serde_json::from_str(&payload_json)
        .with_context(|| format!("invalid request event payload for request {request_id}"))?;

    if let Some(object) = payload.as_object_mut() {
        object.insert(
            "external_work_id".to_string(),
            json!(&external_work_id),
        );
        object.insert(
            "title".to_string(),
            json!(&resolved_title),
        );
        object.insert(
            "author".to_string(),
            json!(&resolved_author),
        );
        object.insert(
            "work".to_string(),
            json!({
                "external_id": &external_work_id,
                "title": &resolved_title,
                "author": &resolved_author,
            }),
        );
    }

    let mut tx = pool.begin().await?;

    sqlx::query("UPDATE requests SET external_work_id = ?, title = ?, author = ? WHERE id = ?")
        .bind(&external_work_id)
        .bind(&resolved_title)
        .bind(&resolved_author)
        .bind(request_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("UPDATE request_events SET payload_json = ? WHERE id = ?")
        .bind(payload.to_string())
        .bind(event_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(())
}
