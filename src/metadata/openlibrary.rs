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
                external_id: doc.key,
                title: doc.title,
                primary_author: doc.author_name.into_iter().next().unwrap_or_default(),
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
                    && doc
                        .author_name
                        .iter()
                        .any(|candidate| normalize_text(candidate) == expected_author)
            })
            .ok_or_else(|| anyhow::anyhow!("no work match"))?;

        Ok(ResolvedWork {
            work: WorkRecord {
                external_id: first.key,
                title: first.title,
                primary_author: first.author_name.into_iter().next().unwrap_or_default(),
            },
        })
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

fn normalize_text(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}
