use anyhow::Result;
use serde::Deserialize;

use crate::domain::catalog::{ResolvedWork, WorkRecord};

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

    pub async fn resolve_work(&self, title: &str, author: &str) -> Result<ResolvedWork> {
        let response = self
            .http
            .get(format!(
                "{}/search.json",
                self.base_url.trim_end_matches('/')
            ))
            .query(&[("title", title), ("author", author), ("limit", "5")])
            .send()
            .await?
            .error_for_status()?;

        let payload: SearchResponse = response.json().await?;
        let expected_title = normalize_text(title);
        let expected_author = normalize_text(author);
        let first = payload
            .docs
            .into_iter()
            .find(|doc| {
                normalize_text(&doc.title) == expected_title
                    && doc
                        .author_name
                        .first()
                        .is_some_and(|candidate| normalize_text(candidate) == expected_author)
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
