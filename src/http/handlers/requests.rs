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
    let request = repo
        .find_by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Request with ID {} not found", id)))?;
    let events = repo.events_for(&id).await?;

    Ok(Json(RequestDetailRecord { request, events }))
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
