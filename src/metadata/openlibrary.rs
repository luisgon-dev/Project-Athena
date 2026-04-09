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
        Self {
            base_url: base_url.into(),
            http: reqwest::Client::new(),
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
        let first = payload
            .docs
            .into_iter()
            .next()
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
