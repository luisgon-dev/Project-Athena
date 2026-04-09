use axum::{
    extract::{Form, Path, State},
    http::StatusCode,
    response::{Html, Redirect},
};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::{
    db::repositories::SqliteRequestRepository,
    domain::requests::{CreateRequest, ManifestationPreference, MediaType},
    http::views::{RequestDetailView, RequestsIndexTemplate, RequestsShowTemplate, render},
};

#[derive(Deserialize)]
pub struct CreateRequestForm {
    pub title: String,
    pub author: String,
    pub media_type: String,
    pub preferred_language: Option<String>,
    pub edition_title: Option<String>,
    pub preferred_narrator: Option<String>,
    pub preferred_publisher: Option<String>,
    pub graphic_audio: Option<String>,
}

pub async fn requests_index() -> Html<String> {
    render(RequestsIndexTemplate)
}

pub async fn create_request(
    State(pool): State<SqlitePool>,
    Form(form): Form<CreateRequestForm>,
) -> Result<Redirect, StatusCode> {
    let media_type = MediaType::from_str(&form.media_type).ok_or(StatusCode::BAD_REQUEST)?;
    let repo = SqliteRequestRepository::new(pool);

    let request = repo
        .create(CreateRequest {
            title: form.title,
            author: form.author,
            media_type,
            preferred_language: form.preferred_language,
            manifestation: ManifestationPreference {
                edition_title: form.edition_title,
                preferred_narrator: form.preferred_narrator,
                preferred_publisher: form.preferred_publisher,
                graphic_audio: form.graphic_audio.is_some(),
            },
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Redirect::to(&format!("/requests/{}", request.id)))
}

pub async fn show_request(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<Html<String>, StatusCode> {
    let repo = SqliteRequestRepository::new(pool);
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
