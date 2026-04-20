use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;

use crate::{
    app::AppState,
    db::repositories::SqliteRequestRepository,
    domain::{
        catalog::WorkSearch,
        requests::{
            CreateRequest, CreateRequestSelection, MediaType, RequestDetailRecord,
            RequestListRecord,
        },
    },
    http::error::AppError,
    workers::{download_worker::DownloadWorker, search_worker::SearchWorker},
};

#[derive(Deserialize)]
pub struct RequestSearchQuery {
    pub title: Option<String>,
    pub author: Option<String>,
}

pub async fn requests_index(
    State(state): State<AppState>,
) -> Result<Json<Vec<RequestListRecord>>, AppError> {
    let repo = SqliteRequestRepository::new(state.pool);
    let requests = repo.list().await?;

    Ok(Json(requests))
}

pub async fn search_requests(
    State(state): State<AppState>,
    Query(search): Query<RequestSearchQuery>,
) -> Result<Json<WorkSearch>, AppError> {
    let title = search.title.unwrap_or_default();
    let author = search.author.unwrap_or_default();
    let has_searched = !(title.trim().is_empty() && author.trim().is_empty());
    let works = if has_searched {
        state
            .open_library_client()
            .await?
            .search_works(&title, &author)
            .await?
    } else {
        WorkSearch { works: Vec::new() }
    };

    Ok(Json(works))
}

pub async fn create_request(
    State(state): State<AppState>,
    Json(payload): Json<CreateRequestSelection>,
) -> Result<
    (
        StatusCode,
        Json<Vec<crate::domain::requests::RequestRecord>>,
    ),
    AppError,
> {
    let selected_work_id = normalize_optional_text(payload.selected_work_id)
        .ok_or_else(|| AppError::BadRequest("missing selected_work_id".to_string()))?;

    let open_library = state.open_library_client().await?;
    let selected_work = open_library
        .resolve_work_by_id(&selected_work_id)
        .await?
        .ok_or_else(|| AppError::BadRequest("selected work id not found".to_string()))?
        .work;

    let media_types = parse_media_types(payload.media_types)?;

    let repo = SqliteRequestRepository::new(state.pool);
    let manifestation = payload.manifestation;

    let created_requests = repo
        .create_batch(
            media_types
                .into_iter()
                .map(|media_type| CreateRequest {
                    external_work_id: selected_work.external_id.clone(),
                    title: selected_work.title.clone(),
                    author: selected_work.primary_author.clone(),
                    media_type,
                    preferred_language: normalize_optional_text(payload.preferred_language.clone()),
                    manifestation: manifestation.clone(),
                })
                .collect(),
        )
        .await?;

    Ok((StatusCode::CREATED, Json(created_requests)))
}

pub async fn show_request(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<RequestDetailRecord>, AppError> {
    let repo = SqliteRequestRepository::new(state.pool);
    Ok(Json(load_request_detail(&repo, &id).await?))
}

pub async fn retry_request_search(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<RequestDetailRecord>, AppError> {
    SearchWorker::process_request_by_id(state.pool.clone(), state.settings.clone(), &id).await?;
    let repo = SqliteRequestRepository::new(state.pool);
    Ok(Json(load_request_detail(&repo, &id).await?))
}

pub async fn approve_review_candidate(
    State(state): State<AppState>,
    Path((id, candidate_id)): Path<(String, i64)>,
) -> Result<Json<RequestDetailRecord>, AppError> {
    let repo = SqliteRequestRepository::new(state.pool.clone());
    let candidate = repo
        .review_candidate_for_action(&id, candidate_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Review candidate {candidate_id} not found")))?;
    let settings = state
        .settings
        .get_persisted_runtime_settings()
        .await?
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("runtime settings row missing")))?;
    let qb = settings.download_clients.qbittorrent;
    if !qb.enabled {
        return Err(AppError::BadRequest(
            "qBittorrent must be enabled before approving candidates".to_string(),
        ));
    }

    let client = crate::downloads::qbittorrent::QbittorrentClient::new(
        qb.base_url,
        qb.username,
        qb.password.unwrap_or_default(),
    );
    DownloadWorker::dispatch_approved_candidate(&repo, &client, &id, &candidate.candidate).await?;
    repo.clear_review_queue(&id).await?;
    repo.mark_review_approved(&id, candidate_id, &candidate.candidate.external_id)
        .await?;

    Ok(Json(load_request_detail(&repo, &id).await?))
}

pub async fn reject_review_candidate(
    State(state): State<AppState>,
    Path((id, candidate_id)): Path<(String, i64)>,
) -> Result<Json<RequestDetailRecord>, AppError> {
    let repo = SqliteRequestRepository::new(state.pool.clone());
    let candidate = repo
        .review_candidate_for_action(&id, candidate_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Review candidate {candidate_id} not found")))?;
    repo.add_rejected_candidate(&id, &candidate.candidate.external_id)
        .await?;
    repo.remove_review_candidate(&id, candidate_id).await?;
    repo.mark_review_rejected(&id, candidate_id, &candidate.candidate.external_id)
        .await?;

    let remaining = repo.review_queue_for(&id).await?;
    if remaining.is_empty() {
        SearchWorker::process_request_by_id(state.pool.clone(), state.settings.clone(), &id)
            .await?;
    } else {
        repo.update_state(&id, "review").await?;
    }

    Ok(Json(load_request_detail(&repo, &id).await?))
}

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn parse_media_types(media_types: Vec<MediaType>) -> Result<Vec<MediaType>, AppError> {
    if media_types.is_empty() {
        return Err(AppError::BadRequest("no media types selected".to_string()));
    }

    Ok(media_types)
}

async fn load_request_detail(
    repo: &SqliteRequestRepository,
    id: &str,
) -> Result<RequestDetailRecord, AppError> {
    let request = repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Request with ID {} not found", id)))?;
    let events = repo.events_for(id).await?;
    let review_queue = repo.review_queue_for(id).await?;

    Ok(RequestDetailRecord {
        request,
        events,
        review_queue,
    })
}
