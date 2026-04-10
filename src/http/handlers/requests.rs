use axum::{
    extract::{Form, Path, Query, State},
    http::StatusCode,
    response::Html,
};
use serde::Deserialize;

use crate::{
    app::AppState,
    db::repositories::SqliteRequestRepository,
    domain::requests::{CreateRequest, ManifestationPreference, MediaType},
    http::views::{
        CreatedRequestView, RequestDetailView, RequestSearchView, RequestsCreatedTemplate,
        RequestsIndexTemplate, RequestsShowTemplate, WorkMatchView, render,
    },
};

#[derive(Deserialize)]
pub struct RequestSearchQuery {
    pub title: Option<String>,
    pub author: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateRequestForm {
    pub selected_work: Option<String>,
    pub ebook: Option<String>,
    pub audiobook: Option<String>,
    pub preferred_language: Option<String>,
    pub edition_title: Option<String>,
    pub preferred_narrator: Option<String>,
    pub preferred_publisher: Option<String>,
    pub graphic_audio: Option<String>,
}

pub async fn requests_index(
    State(state): State<AppState>,
    Query(search): Query<RequestSearchQuery>,
) -> Result<Html<String>, StatusCode> {
    let title = search.title.unwrap_or_default();
    let author = search.author.unwrap_or_default();
    let has_searched = !(title.trim().is_empty() && author.trim().is_empty());
    let (search, matches) = if has_searched {
            let works = state
                .open_library
                .search_works(&title, &author)
                .await
                .map_err(|_| StatusCode::BAD_GATEWAY)?;
            (
                RequestSearchView {
                    title,
                    author,
                },
                works.works.into_iter().map(WorkMatchView::from).collect(),
            )
        } else {
            (RequestSearchView::default(), Vec::new())
        };

    Ok(render(RequestsIndexTemplate {
        search,
        matches,
        has_searched,
    }))
}

pub async fn create_request(
    State(state): State<AppState>,
    Form(form): Form<CreateRequestForm>,
) -> Result<(StatusCode, Html<String>), StatusCode> {
    let selected_work = parse_selected_work(form.selected_work).ok_or(StatusCode::BAD_REQUEST)?;
    let media_types = parse_media_types(form.ebook.is_some(), form.audiobook.is_some())?;
    let repo = SqliteRequestRepository::new(state.pool);
    let manifestation = ManifestationPreference {
        edition_title: normalize_optional_text(form.edition_title),
        preferred_narrator: normalize_optional_text(form.preferred_narrator),
        preferred_publisher: normalize_optional_text(form.preferred_publisher),
        graphic_audio: form.graphic_audio.is_some(),
    };

    let mut created_requests = Vec::with_capacity(media_types.len());
    for media_type in media_types {
        let request = repo
            .create(CreateRequest {
                title: selected_work.title.clone(),
                author: selected_work.author.clone(),
                media_type,
                preferred_language: normalize_optional_text(form.preferred_language.clone()),
                manifestation: manifestation.clone(),
            })
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        created_requests.push(CreatedRequestView::from(request));
    }

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
        if value.trim().is_empty() {
            None
        } else {
            Some(value)
        }
    })
}

#[derive(Clone)]
struct SelectedWork {
    title: String,
    author: String,
}

fn parse_selected_work(value: Option<String>) -> Option<SelectedWork> {
    let value = value?;
    let mut fields = value.splitn(3, '|');
    let external_id = fields.next()?.trim();
    let title = fields.next()?.trim();
    let author = fields.next()?.trim();

    if external_id.is_empty() || title.is_empty() || author.is_empty() {
        return None;
    }

    Some(SelectedWork {
        title: title.to_string(),
        author: author.to_string(),
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
