use anyhow::Result;
use serde::Deserialize;

use crate::domain::catalog::{ResolvedWork, WorkRecord, WorkSearch};

#[derive(Clone)]
pub struct OpenLibraryClient {
    base_url: String,
    covers_base_url: String,
    http: reqwest::Client,
}

impl OpenLibraryClient {
    pub fn new(base_url: impl Into<String>, covers_base_url: impl Into<String>) -> Self {
        let http = reqwest::Client::builder()
            .user_agent("book-router/0.1")
            .build()
            .expect("reqwest client");

        Self {
            base_url: base_url.into(),
            covers_base_url: covers_base_url.into(),
            http,
        }
    }

    pub async fn search_works(&self, title: &str, author: &str) -> Result<WorkSearch> {
        let docs = self.fetch_search_docs(title, author, "10").await?;
        let mut works: Vec<WorkRecord> = docs.into_iter().map(WorkRecord::from).collect();

        for work in works.iter_mut().take(5) {
            if let Ok(Some(detail)) = self.fetch_work_detail(&work.external_id).await {
                work.merge_detail(detail);
            }
        }

        Ok(WorkSearch { works })
    }

    pub async fn resolve_work(&self, title: &str, author: &str) -> Result<ResolvedWork> {
        let expected_title = normalize_text(title);
        let expected_author = normalize_text(author);
        let first = self
            .fetch_search_docs(title, author, "5")
            .await?
            .into_iter()
            .find(|doc| {
                normalize_text(&doc.title) == expected_title
                    && (expected_author.is_empty()
                        || doc
                            .author_name
                            .iter()
                            .any(|candidate| normalize_text(candidate) == expected_author))
            })
            .ok_or_else(|| anyhow::anyhow!("no work match"))?;

        Ok(ResolvedWork {
            work: WorkRecord::from(first),
        })
    }

    pub async fn resolve_work_by_id(&self, external_id: &str) -> Result<Option<ResolvedWork>> {
        let Some(payload) = self.fetch_work_payload(external_id).await? else {
            return Ok(None);
        };
        let author_key = payload
            .authors
            .iter()
            .find_map(|entry| entry.author.as_ref().map(|author| author.key.clone()))
            .unwrap_or_default();
        let primary_author = if author_key.is_empty() {
            unknown_author()
        } else {
            self.fetch_author_name(&author_key).await?
        };
        let external_id = payload.external_id();
        let first_publish_year = payload.first_publish_year();
        let cover_id = payload.covers.first().copied();
        let title = payload.title;
        let description = payload.description.and_then(DescriptionField::into_text);
        let subjects = normalize_subjects(payload.subjects);

        Ok(Some(ResolvedWork {
            work: WorkRecord {
                external_id,
                title,
                primary_author,
                first_publish_year,
                description,
                cover_id,
                subjects,
                edition_count: None,
            },
        }))
    }

    pub async fn fetch_cover(&self, cover_id: i64, size: CoverSize) -> Result<Option<CoverImage>> {
        let response = self
            .http
            .get(format!(
                "{}/b/id/{}-{}.jpg",
                self.covers_base_url.trim_end_matches('/'),
                cover_id,
                size.as_str()
            ))
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        let response = response.error_for_status()?;
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();
        let bytes = response.bytes().await?.to_vec();

        Ok(Some(CoverImage {
            content_type,
            bytes,
        }))
    }

    async fn fetch_search_docs(
        &self,
        title: &str,
        author: &str,
        limit: &str,
    ) -> Result<Vec<SearchDoc>> {
        let response = self
            .http
            .get(format!(
                "{}/search.json",
                self.base_url.trim_end_matches('/')
            ))
            .query(&[("title", title), ("author", author), ("limit", limit)])
            .send()
            .await?
            .error_for_status()?;

        let payload: SearchResponse = response.json().await?;
        Ok(payload.docs)
    }

    async fn fetch_work_detail(&self, external_id: &str) -> Result<Option<WorkDetail>> {
        let Some(payload) = self.fetch_work_payload(external_id).await? else {
            return Ok(None);
        };
        let first_publish_year = payload.first_publish_year();
        let description = payload.description.and_then(DescriptionField::into_text);
        let cover_id = payload.covers.first().copied();
        let subjects = normalize_subjects(payload.subjects);

        Ok(Some(WorkDetail {
            description,
            first_publish_year,
            cover_id,
            subjects,
        }))
    }

    async fn fetch_work_payload(&self, external_id: &str) -> Result<Option<WorkResponse>> {
        let response = self
            .http
            .get(format!(
                "{}/works/{}.json",
                self.base_url.trim_end_matches('/'),
                external_id
            ))
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        Ok(Some(response.error_for_status()?.json().await?))
    }

    async fn fetch_author_name(&self, author_key: &str) -> Result<String> {
        let response = self
            .http
            .get(format!(
                "{}{}.json",
                self.base_url.trim_end_matches('/'),
                author_key
            ))
            .send()
            .await?
            .error_for_status()?;

        let payload: AuthorResponse = response.json().await?;
        Ok(normalize_author_name(Some(payload.name)))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoverImage {
    pub content_type: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoverSize {
    Small,
    Medium,
    Large,
}

impl CoverSize {
    pub fn from_query_value(value: Option<&str>) -> Self {
        match value.unwrap_or("M") {
            "S" | "s" => Self::Small,
            "L" | "l" => Self::Large,
            _ => Self::Medium,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Small => "S",
            Self::Medium => "M",
            Self::Large => "L",
        }
    }
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    docs: Vec<SearchDoc>,
}

#[derive(Debug, Deserialize)]
struct SearchDoc {
    key: String,
    title: String,
    #[serde(default)]
    author_name: Vec<String>,
    first_publish_year: Option<i32>,
    cover_i: Option<i64>,
    #[serde(default)]
    subject: Vec<String>,
    edition_count: Option<u32>,
}

impl From<SearchDoc> for WorkRecord {
    fn from(doc: SearchDoc) -> Self {
        Self {
            external_id: normalize_work_key(&doc.key),
            title: doc.title,
            primary_author: normalize_author_name(doc.author_name.into_iter().next()),
            first_publish_year: doc.first_publish_year,
            description: None,
            cover_id: doc.cover_i,
            subjects: normalize_subjects(doc.subject),
            edition_count: doc.edition_count,
        }
    }
}

#[derive(Debug, Deserialize)]
struct WorkResponse {
    key: String,
    title: String,
    #[serde(default)]
    authors: Vec<WorkAuthorEntry>,
    description: Option<DescriptionField>,
    #[serde(default)]
    subjects: Vec<String>,
    #[serde(default)]
    covers: Vec<i64>,
    first_publish_date: Option<String>,
}

impl WorkResponse {
    fn external_id(&self) -> String {
        normalize_work_key(&self.key)
    }

    fn first_publish_year(&self) -> Option<i32> {
        self.first_publish_date
            .as_deref()
            .and_then(parse_year_prefix)
    }
}

#[derive(Debug, Deserialize)]
struct WorkAuthorEntry {
    author: Option<AuthorKey>,
}

#[derive(Debug, Deserialize)]
struct AuthorKey {
    key: String,
}

#[derive(Debug, Deserialize)]
struct AuthorResponse {
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum DescriptionField {
    Text(String),
    Value { value: String },
}

impl DescriptionField {
    fn into_text(self) -> Option<String> {
        match self {
            Self::Text(value) => normalize_optional_text(Some(value)),
            Self::Value { value } => normalize_optional_text(Some(value)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WorkDetail {
    description: Option<String>,
    first_publish_year: Option<i32>,
    cover_id: Option<i64>,
    subjects: Vec<String>,
}

impl WorkRecord {
    fn merge_detail(&mut self, detail: WorkDetail) {
        if self.description.is_none() {
            self.description = detail.description;
        }
        if self.first_publish_year.is_none() {
            self.first_publish_year = detail.first_publish_year;
        }
        if self.cover_id.is_none() {
            self.cover_id = detail.cover_id;
        }
        if self.subjects.is_empty() {
            self.subjects = detail.subjects;
        }
    }
}

fn normalize_text(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn normalize_work_key(value: &str) -> String {
    value.trim_start_matches("/works/").to_string()
}

fn normalize_author_name(value: Option<String>) -> String {
    normalize_optional_text(value).unwrap_or_else(unknown_author)
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

fn normalize_subjects(subjects: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::new();

    for subject in subjects {
        let Some(subject) = normalize_optional_text(Some(subject)) else {
            continue;
        };
        if normalized.iter().any(|existing| existing == &subject) {
            continue;
        }
        normalized.push(subject);
        if normalized.len() == 5 {
            break;
        }
    }

    normalized
}

fn parse_year_prefix(value: &str) -> Option<i32> {
    let prefix: String = value.chars().take_while(|ch| ch.is_ascii_digit()).collect();
    if prefix.len() == 4 {
        prefix.parse().ok()
    } else {
        None
    }
}

fn unknown_author() -> String {
    "Unknown author".to_string()
}
