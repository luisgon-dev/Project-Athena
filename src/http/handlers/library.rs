use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use std::path::Path;

use crate::{
    app::AppState,
    db::repositories::SqliteRequestRepository,
    domain::library::{LibraryScanJobRecord, LibraryScanResponse},
    http::{auth::require_admin, error::AppError},
    library_scanner::LibraryScanner,
};

pub async fn trigger_scan(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<LibraryScanResponse>), AppError> {
    require_admin(&state, &headers).await?;
    let settings = state
        .settings
        .get_persisted_runtime_settings()
        .await?
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("runtime settings row missing")))?;

    let repo = SqliteRequestRepository::new(state.pool.clone());
    let job_id = repo.create_scan_job().await?;

    let ebooks_root = settings.storage.ebooks_root.clone();
    let audiobooks_root = settings.storage.audiobooks_root.clone();

    tokio::spawn(async move {
        let repo = SqliteRequestRepository::new(state.pool);
        let ebooks_path = Path::new(&ebooks_root);
        let audiobooks_path = Path::new(&audiobooks_root);

        match LibraryScanner::scan(ebooks_path, audiobooks_path, &repo).await {
            Ok(counts) => {
                if let Err(error) = repo
                    .complete_scan_job(
                        job_id,
                        counts.ebooks_found,
                        counts.audiobooks_found,
                        counts.duplicates_skipped,
                        None,
                    )
                    .await
                {
                    tracing::error!(error = %error, job_id, "failed to complete scan job");
                }
            }
            Err(error) => {
                tracing::error!(error = %error, job_id, "library scan failed");
                if let Err(inner) = repo
                    .complete_scan_job(job_id, 0, 0, 0, Some(&error.to_string()))
                    .await
                {
                    tracing::error!(error = %inner, job_id, "failed to record scan job failure");
                }
            }
        }
    });

    Ok((StatusCode::ACCEPTED, Json(LibraryScanResponse { job_id })))
}

pub async fn scan_status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Option<LibraryScanJobRecord>>, AppError> {
    require_admin(&state, &headers).await?;
    let repo = SqliteRequestRepository::new(state.pool);
    let job = repo.latest_scan_job().await?;
    Ok(Json(job))
}
