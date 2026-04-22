use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
};
use serde::Deserialize;
use tracing::warn;

use crate::{
    app::AppState,
    db::repositories::{SqliteRequestRepository, SqliteSubmissionRepository},
    domain::{
        auth::AuthUserRecord,
        catalog::WorkRecord,
        requests::{CreateRequest, MediaType},
        submissions::{
            CreateSubmissionRequest, DuplicateHint, DuplicateSource, RequestSubmissionDetailRecord,
            RequestSubmissionRecord, ResolveManualSubmissionRequest, SubmissionSearchCandidate,
            SubmissionSearchResult, SubmissionStatus,
        },
    },
    http::{
        auth::{require_admin, require_user},
        error::AppError,
    },
    notifications::send_notification,
    sync::audiobookshelf::{AudiobookshelfClient, AudiobookshelfLibraryItem},
    workers::search_worker::SearchWorker,
};

#[derive(Deserialize)]
pub struct SubmissionSearchQuery {
    pub title: Option<String>,
    pub author: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct SubmissionListQuery {
    pub all: Option<bool>,
}

pub async fn search_submissions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(search): Query<SubmissionSearchQuery>,
) -> Result<Json<SubmissionSearchResult>, AppError> {
    require_user(&state, &headers).await?;

    let title = search.title.unwrap_or_default();
    let author = search.author.unwrap_or_default();
    if title.trim().is_empty() && author.trim().is_empty() {
        return Ok(Json(SubmissionSearchResult { works: Vec::new() }));
    }

    let works = state
        .open_library_client()
        .await?
        .search_works(&title, &author)
        .await?;

    let mut results = Vec::with_capacity(works.works.len());
    for work in works.works {
        let duplicate_hints = duplicate_hints_for(&state, &work, true).await?;
        results.push(SubmissionSearchCandidate {
            work,
            duplicate_hints,
        });
    }

    Ok(Json(SubmissionSearchResult { works: results }))
}

pub async fn submissions_index(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<SubmissionListQuery>,
) -> Result<Json<Vec<RequestSubmissionRecord>>, AppError> {
    let user = require_user(&state, &headers).await?;
    let submissions = SqliteSubmissionRepository::new(state.pool)
        .list_for_user(&user, query.all.unwrap_or(false))
        .await?;
    Ok(Json(submissions))
}

pub async fn create_submission(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateSubmissionRequest>,
) -> Result<(StatusCode, Json<RequestSubmissionDetailRecord>), AppError> {
    let user = require_user(&state, &headers).await?;
    if payload.media_types.is_empty() {
        return Err(AppError::BadRequest("no media types selected".to_string()));
    }

    let submission_repo = SqliteSubmissionRepository::new(state.pool.clone());
    let request_repo = SqliteRequestRepository::new(state.pool.clone());
    let requires_admin_approval = !user.role.can_auto_acquire();

    let detail = match payload.intake_mode.clone().unwrap_or_default() {
        crate::domain::submissions::SubmissionIntakeMode::Metadata => {
            let selected_work_id = normalize_optional_text(payload.selected_work_id.clone())
                .ok_or_else(|| AppError::BadRequest("missing selected_work_id".to_string()))?;
            let selected_work = state
                .open_library_client()
                .await?
                .resolve_work_by_id(&selected_work_id)
                .await?
                .ok_or_else(|| AppError::BadRequest("selected work id not found".to_string()))?
                .work;

            enforce_duplicates(&state, &user, &payload, &selected_work).await?;

            let submission = submission_repo
                .create_submission(
                    &user,
                    &payload,
                    &selected_work.title,
                    &selected_work.primary_author,
                    Some(&selected_work.external_id),
                    if requires_admin_approval {
                        SubmissionStatus::Submitted
                    } else {
                        SubmissionStatus::Approved
                    },
                    requires_admin_approval,
                )
                .await?;

            let requests = payload
                .media_types
                .iter()
                .cloned()
                .map(|media_type| CreateRequest {
                    external_work_id: selected_work.external_id.clone(),
                    title: selected_work.title.clone(),
                    author: selected_work.primary_author.clone(),
                    media_type,
                    preferred_language: normalize_optional_text(payload.preferred_language.clone()),
                    manifestation: payload.manifestation.clone(),
                })
                .collect();
            request_repo
                .create_batch_linked(
                    requests,
                    Some(&submission.id),
                    Some(&user.id),
                    requires_admin_approval,
                )
                .await?;
            submission_repo
                .append_event(
                    &submission.id,
                    "requests_created",
                    serde_json::json!({ "requires_admin_approval": requires_admin_approval }),
                )
                .await?;
            submission_repo
                .detail_for(&submission.id)
                .await?
                .ok_or_else(|| AppError::Internal(anyhow::anyhow!("submission disappeared")))?
        }
        crate::domain::submissions::SubmissionIntakeMode::Manual => {
            let title = normalize_optional_text(payload.title.clone())
                .ok_or_else(|| AppError::BadRequest("manual submissions require a title".to_string()))?;
            let author = normalize_optional_text(payload.author.clone())
                .ok_or_else(|| AppError::BadRequest("manual submissions require an author".to_string()))?;
            let submission = submission_repo
                .create_submission(
                    &user,
                    &payload,
                    &title,
                    &author,
                    None,
                    SubmissionStatus::PendingResolution,
                    requires_admin_approval,
                )
                .await?;
            submission_repo
                .append_event(
                    &submission.id,
                    "manual_resolution_required",
                    serde_json::json!({}),
                )
                .await?;
            submission_repo
                .detail_for(&submission.id)
                .await?
                .ok_or_else(|| AppError::Internal(anyhow::anyhow!("submission disappeared")))?
        }
    };

    if let Err(error) = send_notification(
        &state.settings,
        "submission_created",
        "Athena submission created",
        &format!("{} by {}", detail.submission.title, detail.submission.requested_by_username),
        serde_json::json!({
            "submission_id": detail.submission.id,
            "status": detail.submission.status,
            "intake_mode": detail.submission.intake_mode,
        }),
    )
    .await
    {
        warn!(error = %error, submission_id = %detail.submission.id, "failed to send submission notification");
    }

    Ok((StatusCode::CREATED, Json(detail)))
}

pub async fn show_submission(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<RequestSubmissionDetailRecord>, AppError> {
    let user = require_user(&state, &headers).await?;
    let detail = SqliteSubmissionRepository::new(state.pool)
        .detail_for(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("submission {id} not found")))?;

    if !user.role.is_admin() && detail.submission.requested_by_user_id != user.id {
        return Err(AppError::Forbidden("submission access denied".to_string()));
    }

    Ok(Json(detail))
}

pub async fn resolve_submission(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<ResolveManualSubmissionRequest>,
) -> Result<Json<RequestSubmissionDetailRecord>, AppError> {
    require_admin(&state, &headers).await?;
    let submission_repo = SqliteSubmissionRepository::new(state.pool.clone());
    let request_repo = SqliteRequestRepository::new(state.pool.clone());
    let submission = submission_repo
        .find_by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("submission {id} not found")))?;

    let selected_work = state
        .open_library_client()
        .await?
        .resolve_work_by_id(&payload.selected_work_id)
        .await?
        .ok_or_else(|| AppError::BadRequest("selected work id not found".to_string()))?
        .work;

    let requests = submission
        .media_types
        .iter()
        .cloned()
        .map(|media_type| CreateRequest {
            external_work_id: selected_work.external_id.clone(),
            title: selected_work.title.clone(),
            author: selected_work.primary_author.clone(),
            media_type,
            preferred_language: submission.preferred_language.clone(),
            manifestation: submission.manifestation.clone(),
        })
        .collect();

    request_repo
        .create_batch_linked(
            requests,
            Some(&submission.id),
            Some(&submission.requested_by_user_id),
            submission.requires_admin_approval,
        )
        .await?;
    submission_repo
        .resolve_metadata(
            &submission.id,
            &selected_work.external_id,
            &selected_work.title,
            &selected_work.primary_author,
            if submission.requires_admin_approval {
                SubmissionStatus::Resolved
            } else {
                SubmissionStatus::Approved
            },
        )
        .await?;
    submission_repo
        .append_event(
            &submission.id,
            "resolved",
            serde_json::json!({
                "external_work_id": selected_work.external_id,
                "title": selected_work.title,
                "author": selected_work.primary_author,
            }),
        )
        .await?;

    if let Err(error) = send_notification(
        &state.settings,
        "submission_resolved",
        "Athena submission resolved",
        &format!("{} resolved to canonical metadata", submission.title),
        serde_json::json!({ "submission_id": submission.id }),
    )
    .await
    {
        warn!(error = %error, submission_id = %submission.id, "failed to send resolve notification");
    }

    Ok(Json(
        submission_repo
            .detail_for(&submission.id)
            .await?
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("submission disappeared")))?,
    ))
}

pub async fn approve_submission(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<RequestSubmissionDetailRecord>, AppError> {
    require_admin(&state, &headers).await?;
    let submission_repo = SqliteSubmissionRepository::new(state.pool.clone());
    let request_repo = SqliteRequestRepository::new(state.pool.clone());

    request_repo.set_submission_approval(&id, false).await?;
    submission_repo.update_status(&id, SubmissionStatus::Approved).await?;
    submission_repo
        .append_event(&id, "approved", serde_json::json!({}))
        .await?;

    if let Err(error) = send_notification(
        &state.settings,
        "submission_approved",
        "Athena submission approved",
        &format!("Submission {id} approved"),
        serde_json::json!({ "submission_id": id }),
    )
    .await
    {
        warn!(error = %error, submission_id = %id, "failed to send approval notification");
    }

    let linked_requests = request_repo.linked_requests_for_submission(&id).await?;
    for request in linked_requests {
        let _ = SearchWorker::process_request_by_id(
            state.pool.clone(),
            state.settings.clone(),
            &request.id,
        )
        .await;
    }

    Ok(Json(
        submission_repo
            .detail_for(&id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("submission {id} not found")))?,
    ))
}

pub async fn reject_submission(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<RequestSubmissionDetailRecord>, AppError> {
    require_admin(&state, &headers).await?;
    let submission_repo = SqliteSubmissionRepository::new(state.pool.clone());
    let request_repo = SqliteRequestRepository::new(state.pool.clone());

    submission_repo.update_status(&id, SubmissionStatus::Rejected).await?;
    request_repo.set_submission_request_state(&id, "rejected").await?;
    for request in request_repo.linked_requests_for_submission(&id).await? {
        let _ = request_repo.clear_review_queue(&request.id).await;
    }
    submission_repo
        .append_event(&id, "rejected", serde_json::json!({}))
        .await?;

    if let Err(error) = send_notification(
        &state.settings,
        "submission_rejected",
        "Athena submission rejected",
        &format!("Submission {id} rejected"),
        serde_json::json!({ "submission_id": id }),
    )
    .await
    {
        warn!(error = %error, submission_id = %id, "failed to send rejection notification");
    }

    Ok(Json(
        submission_repo
            .detail_for(&id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("submission {id} not found")))?,
    ))
}

async fn duplicate_hints_for(
    state: &AppState,
    work: &WorkRecord,
    include_abs: bool,
) -> Result<Vec<DuplicateHint>, AppError> {
    let request_repo = SqliteRequestRepository::new(state.pool.clone());
    let mut hints = Vec::new();

    for request in request_repo
        .list_requests_by_title_author(&work.title, &work.primary_author)
        .await?
    {
        let source = if request.state == "imported" || request.state == "synced" {
            DuplicateSource::AthenaImported
        } else {
            DuplicateSource::AthenaRequested
        };
        hints.push(DuplicateHint {
            source,
            label: format!("Athena {}", request.state),
            detail: Some(format!("{} {}", media_label(&request.media_type), request.id)),
        });
    }

    if include_abs {
        let settings = state
            .settings
            .get_persisted_runtime_settings()
            .await?
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("runtime settings row missing")))?;
        let abs = &settings.integrations.audiobookshelf;
        if abs.enabled && abs.mark_existing_during_search {
            if let Some(api_key) = &abs.api_key {
                let client = AudiobookshelfClient::new(&abs.base_url, api_key);
                let query = format!("{} {}", work.title, work.primary_author);
                if let Ok(items) = client.search_library(&abs.library_id, &query).await {
                    for item in items
                        .into_iter()
                        .filter(|item| abs_matches_work(item, work))
                        .take(3)
                    {
                        hints.push(DuplicateHint {
                            source: DuplicateSource::Audiobookshelf,
                            label: "Already in Audiobookshelf".to_string(),
                            detail: Some(item.library_item.id),
                        });
                    }
                }
            }
        }
    }

    Ok(hints)
}

async fn enforce_duplicates(
    state: &AppState,
    user: &AuthUserRecord,
    payload: &CreateSubmissionRequest,
    work: &WorkRecord,
) -> Result<(), AppError> {
    if !payload.media_types.contains(&MediaType::Audiobook) {
        return Ok(());
    }

    let duplicate_hints = duplicate_hints_for(state, work, true).await?;
    let has_blocking_duplicate = duplicate_hints
        .iter()
        .any(|hint| matches!(hint.source, DuplicateSource::AthenaImported | DuplicateSource::Audiobookshelf));

    if has_blocking_duplicate && !(payload.allow_duplicate && user.role.is_admin()) {
        return Err(AppError::BadRequest(
            "duplicate audiobook suppressed; only admins can override".to_string(),
        ));
    }

    Ok(())
}

fn abs_matches_work(item: &AudiobookshelfLibraryItem, work: &WorkRecord) -> bool {
    let metadata = &item.library_item.media.metadata;
    normalize_for_match(&metadata.title) == normalize_for_match(&work.title)
        && normalize_for_match(metadata.author_name.as_deref().unwrap_or_default())
            == normalize_for_match(&work.primary_author)
}

fn normalize_for_match(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
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

fn media_label(media_type: &MediaType) -> &'static str {
    match media_type {
        MediaType::Ebook => "Ebook",
        MediaType::Audiobook => "Audiobook",
    }
}
