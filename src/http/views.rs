use askama::Template;
use axum::response::Html;

use crate::domain::{
    catalog::WorkRecord,
    requests::{MediaType, RequestRecord},
};

pub fn render<T: Template>(template: T) -> Html<String> {
    Html(
        template
            .render()
            .expect("template rendering should succeed"),
    )
}

pub fn media_type_label(media_type: &MediaType) -> &'static str {
    match media_type {
        MediaType::Ebook => "Ebook",
        MediaType::Audiobook => "Audiobook",
    }
}

#[derive(Clone, Debug)]
pub struct RequestDetailView {
    pub id: String,
    pub title: String,
    pub author: String,
    pub media_type_label: &'static str,
    pub preferred_language: String,
    pub edition_title: String,
    pub preferred_narrator: String,
    pub preferred_publisher: String,
    pub graphic_audio: bool,
    pub state: String,
    pub created_at: String,
}

impl From<RequestRecord> for RequestDetailView {
    fn from(record: RequestRecord) -> Self {
        Self {
            id: record.id,
            title: record.title,
            author: record.author,
            media_type_label: media_type_label(&record.media_type),
            preferred_language: record
                .preferred_language
                .unwrap_or_else(|| "Any".to_string()),
            edition_title: record
                .manifestation
                .edition_title
                .unwrap_or_else(|| "Any".to_string()),
            preferred_narrator: record
                .manifestation
                .preferred_narrator
                .unwrap_or_else(|| "Any".to_string()),
            preferred_publisher: record
                .manifestation
                .preferred_publisher
                .unwrap_or_else(|| "Any".to_string()),
            graphic_audio: record.manifestation.graphic_audio,
            state: record.state,
            created_at: record.created_at,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct RequestSearchView {
    pub title: String,
    pub author: String,
}

#[derive(Clone, Debug)]
pub struct WorkMatchView {
    pub selection_value: String,
    pub title: String,
    pub author: String,
}

impl From<WorkRecord> for WorkMatchView {
    fn from(work: WorkRecord) -> Self {
        Self {
            selection_value: format!(
                "{}|{}|{}",
                work.external_id, work.title, work.primary_author
            ),
            title: work.title,
            author: work.primary_author,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CreatedRequestView {
    pub id: String,
    pub title: String,
    pub author: String,
    pub media_type_label: &'static str,
}

impl From<RequestRecord> for CreatedRequestView {
    fn from(record: RequestRecord) -> Self {
        Self {
            id: record.id,
            title: record.title,
            author: record.author,
            media_type_label: media_type_label(&record.media_type),
        }
    }
}

#[derive(Template)]
#[template(path = "requests/index.html")]
pub struct RequestsIndexTemplate {
    pub search: RequestSearchView,
    pub matches: Vec<WorkMatchView>,
    pub has_searched: bool,
}

#[derive(Template)]
#[template(path = "requests/show.html")]
pub struct RequestsShowTemplate {
    pub request: RequestDetailView,
}

#[derive(Template)]
#[template(path = "requests/created.html")]
pub struct RequestsCreatedTemplate {
    pub requests: Vec<CreatedRequestView>,
}
