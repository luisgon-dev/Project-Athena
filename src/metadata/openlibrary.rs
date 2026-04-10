use anyhow::Result;
use serde::Deserialize;

use crate::domain::catalog::{ResolvedWork, WorkRecord, WorkSearch};

#[derive(Clone)]
pub struct OpenLibraryClient {
    base_url: String,
    http: reqwest::Client,
}

impl OpenLibraryClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        let http = reqwest::Client::builder()
            .user_agent("book-router/0.1")
            .build()
            .expect("reqwest client");

        Self {
            base_url: base_url.into(),
            http,
        }
    }

    pub async fn search_works(&self, title: &str, author: &str) -> Result<WorkSearch> {
        let works = self
            .fetch_search_docs(title, author, "10")
            .await?
            .into_iter()
            .map(|doc| WorkRecord {
                external_id: normalize_work_key(&doc.key),
                title: doc.title,
                primary_author: normalize_author_name(doc.author_name.into_iter().next()),
            })
            .collect();

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
            work: WorkRecord {
                external_id: normalize_work_key(&first.key),
                title: first.title,
                primary_author: normalize_author_name(first.author_name.into_iter().next()),
            },
        })
    }

    pub async fn resolve_work_by_id(&self, external_id: &str) -> Result<Option<ResolvedWork>> {
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

        let response = response.error_for_status()?;

        let payload: WorkResponse = response.json().await?;
        let external_id = payload.external_id();
        let author_key = payload
            .authors
            .into_iter()
            .find_map(|entry| entry.author.map(|author| author.key))
            .unwrap_or_default();
        let primary_author = if author_key.is_empty() {
            unknown_author()
        } else {
            self.fetch_author_name(&author_key).await?
        };

        Ok(Some(ResolvedWork {
            work: WorkRecord {
                external_id,
                title: payload.title,
                primary_author,
            },
        }))
    }

    async fn fetch_search_docs(&self, title: &str, author: &str, limit: &str) -> Result<Vec<SearchDoc>> {
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
}

#[derive(Debug, Deserialize)]
struct WorkResponse {
    key: String,
    title: String,
    #[serde(default)]
    authors: Vec<WorkAuthorEntry>,
}

impl WorkResponse {
    fn external_id(&self) -> String {
        self.key
            .trim_start_matches("/works/")
            .to_string()
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
    value
        .and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .unwrap_or_else(unknown_author)
}

fn unknown_author() -> String {
    "Unknown author".to_string()
}
