use axum::{
    extract::{Form, Path, Query, State},
    http::StatusCode,
    response::Html,
};
use serde::Deserialize;
use tracing::warn;

use crate::{
    app::AppState,
    db::repositories::SqliteRequestRepository,
    domain::requests::{CreateRequest, ManifestationPreference, MediaType},
    http::views::{
        CreatedRequestView, RequestDetailView, RequestListView, RequestSearchView,
        RequestsCreatedTemplate, RequestsIndexTemplate, RequestsNewTemplate, RequestsShowTemplate,
        WorkMatchView, render,
    },
};

#[derive(Deserialize)]
pub struct RequestSearchQuery {
    pub title: Option<String>,
    pub author: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateRequestForm {
    pub selected_work_id: Option<String>,
    pub ebook: Option<String>,
    pub audiobook: Option<String>,
    pub preferred_language: Option<String>,
    pub edition_title: Option<String>,
    pub preferred_narrator: Option<String>,
    pub preferred_publisher: Option<String>,
    pub graphic_audio: Option<String>,
}

pub async fn requests_index(State(state): State<AppState>) -> Result<Html<String>, StatusCode> {
    let repo = SqliteRequestRepository::new(state.pool);
    let requests = repo
        .list()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(RequestListView::from)
        .collect();

    Ok(render(RequestsIndexTemplate { requests }))
}

pub async fn new_request(
    State(state): State<AppState>,
    Query(search): Query<RequestSearchQuery>,
) -> Result<Html<String>, StatusCode> {
    let title = search.title.unwrap_or_default();
    let author = search.author.unwrap_or_default();
    let has_searched = !(title.trim().is_empty() && author.trim().is_empty());
    let (search, matches) = if has_searched {
        let works = match state.open_library.search_works(&title, &author).await {
            Ok(works) => works,
            Err(error) => {
                warn!(%title, %author, error = %error, "metadata search failed");
                return Err(StatusCode::BAD_GATEWAY);
            }
        };
        (
            RequestSearchView { title, author },
            works.works.into_iter().map(WorkMatchView::from).collect(),
        )
    } else {
        (RequestSearchView::default(), Vec::new())
    };

    Ok(render(RequestsNewTemplate {
        search,
        matches,
        has_searched,
    }))
}

pub async fn create_request(
    State(state): State<AppState>,
    Form(form): Form<CreateRequestForm>,
) -> Result<(StatusCode, Html<String>), StatusCode> {
    let selected_work_id = match normalize_optional_text(form.selected_work_id) {
        Some(selected_work_id) => selected_work_id,
        None => {
            warn!("request creation rejected: missing selected_work_id");
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    let selected_work = match state
        .open_library
        .resolve_work_by_id(&selected_work_id)
        .await
    {
        Ok(Some(resolved)) => resolved.work,
        Ok(None) => {
            warn!(%selected_work_id, "request creation rejected: selected work id not found");
            return Err(StatusCode::BAD_REQUEST);
        }
        Err(error) => {
            warn!(%selected_work_id, error = %error, "request creation failed during work resolution");
            return Err(StatusCode::BAD_GATEWAY);
        }
    };
    let media_types = match parse_media_types(form.ebook.is_some(), form.audiobook.is_some()) {
        Ok(media_types) => media_types,
        Err(status) => {
            warn!(%selected_work_id, "request creation rejected: no media types selected");
            return Err(status);
        }
    };
    let repo = SqliteRequestRepository::new(state.pool);
    let manifestation = ManifestationPreference {
        edition_title: normalize_optional_text(form.edition_title),
        preferred_narrator: normalize_optional_text(form.preferred_narrator),
        preferred_publisher: normalize_optional_text(form.preferred_publisher),
        graphic_audio: form.graphic_audio.is_some(),
    };

    let created_requests = repo
        .create_batch(
            media_types
                .into_iter()
                .map(|media_type| CreateRequest {
                    external_work_id: selected_work.external_id.clone(),
                    title: selected_work.title.clone(),
                    author: selected_work.primary_author.clone(),
                    media_type,
                    preferred_language: normalize_optional_text(form.preferred_language.clone()),
                    manifestation: manifestation.clone(),
                })
                .collect(),
        )
        .await
        .map_err(|error| {
            warn!(%selected_work_id, error = %error, "request creation failed while persisting requests");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .into_iter()
        .map(CreatedRequestView::from)
        .collect();

    Ok((
        StatusCode::CREATED,
        render(RequestsCreatedTemplate {
            requests: created_requests,
        }),
    ))
}

pub async fn show_request(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Html<String>, StatusCode> {
    let repo = SqliteRequestRepository::new(state.pool);
    let Some(request) = repo
        .find_by_id(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::NOT_FOUND);
    };

    Ok(render(RequestsShowTemplate {
        request: RequestDetailView::from(request),
    }))
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

fn parse_media_types(ebook: bool, audiobook: bool) -> Result<Vec<MediaType>, StatusCode> {
    let mut media_types = Vec::with_capacity(2);
    if ebook {
        media_types.push(MediaType::Ebook);
    }
    if audiobook {
        media_types.push(MediaType::Audiobook);
    }

    if media_types.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(media_types)
}
