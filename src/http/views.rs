use askama::Template;
use axum::response::Html;

use crate::domain::{
    catalog::WorkRecord,
    requests::{MediaType, RequestListRecord, RequestRecord},
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
    pub external_work_id: String,
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
            external_work_id: if record.external_work_id.trim().is_empty() {
                "Unresolved".to_string()
            } else {
                record.external_work_id
            },
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

#[derive(Clone, Debug)]
pub struct RequestListView {
    pub id: String,
    pub title: String,
    pub author: String,
    pub media_type_label: &'static str,
    pub state: String,
    pub created_at: String,
}

impl From<RequestListRecord> for RequestListView {
    fn from(record: RequestListRecord) -> Self {
        Self {
            id: record.id,
            title: record.title,
            author: record.author,
            media_type_label: media_type_label(&record.media_type),
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
    pub external_id: String,
    pub title: String,
    pub author: String,
    pub first_publish_year: Option<i32>,
    pub description_excerpt: String,
    pub has_description: bool,
    pub cover_url: String,
    pub has_cover: bool,
    pub subjects: Vec<String>,
    pub edition_count_label: String,
    pub has_edition_count: bool,
}

impl From<WorkRecord> for WorkMatchView {
    fn from(work: WorkRecord) -> Self {
        let description_excerpt = excerpt_text(work.description.as_deref().unwrap_or(""), 220);
        let has_description = !description_excerpt.is_empty();
        let cover_url = work
            .cover_id
            .map(|cover_id| format!("/covers/openlibrary/{cover_id}"))
            .unwrap_or_default();
        let has_cover = !cover_url.is_empty();
        let edition_count_label = work
            .edition_count
            .map(|count| format!("{count} edition{}", if count == 1 { "" } else { "s" }))
            .unwrap_or_default();
        let has_edition_count = !edition_count_label.is_empty();

        Self {
            external_id: work.external_id,
            title: work.title,
            author: work.primary_author,
            first_publish_year: work.first_publish_year,
            description_excerpt,
            has_description,
            cover_url,
            has_cover,
            subjects: work.subjects,
            edition_count_label,
            has_edition_count,
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
    pub requests: Vec<RequestListView>,
}

#[derive(Template)]
#[template(path = "requests/new.html")]
pub struct RequestsNewTemplate {
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

fn excerpt_text(value: &str, limit: usize) -> String {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut chars = normalized.chars();
    let excerpt: String = chars.by_ref().take(limit).collect();
    if excerpt.chars().count() == normalized.chars().count() {
        normalized
    } else {
        format!("{}...", excerpt.trim_end())
    }
}
