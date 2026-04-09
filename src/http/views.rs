use askama::Template;
use axum::response::Html;

use crate::domain::requests::{MediaType, RequestRecord};

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

#[derive(Template)]
#[template(path = "requests/index.html")]
pub struct RequestsIndexTemplate;

#[derive(Template)]
#[template(path = "requests/show.html")]
pub struct RequestsShowTemplate {
    pub request: RequestDetailView,
}
