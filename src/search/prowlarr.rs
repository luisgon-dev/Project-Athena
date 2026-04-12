use serde::Deserialize;

use crate::domain::search::ReleaseCandidate;

#[derive(Clone)]
pub struct ProwlarrClient {
    base_url: String,
    api_key: String,
    http: reqwest::Client,
}

impl ProwlarrClient {
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            api_key: api_key.into(),
            http: reqwest::Client::new(),
        }
    }

    pub async fn search(
        &self,
        query: &str,
        media_type: &str,
    ) -> anyhow::Result<Vec<ReleaseCandidate>> {
        let response = self
            .http
            .get(format!("{}/api/v1/search", self.base_url))
            .header("X-Api-Key", &self.api_key)
            .query(&[("query", query), ("type", media_type)])
            .send()
            .await?
            .error_for_status()?;

        let items: Vec<ProwlarrItem> = response.json().await?;
        Ok(items.into_iter().map(ReleaseCandidate::from).collect())
    }
}

#[derive(Deserialize)]
struct ProwlarrItem {
    guid: String,
    title: String,
    size: i64,
    protocol: String,
    indexer: String,
    #[serde(rename = "downloadUrl")]
    download_url: Option<String>,
}

impl From<ProwlarrItem> for ReleaseCandidate {
    fn from(value: ProwlarrItem) -> Self {
        Self {
            external_id: value.guid,
            source: "prowlarr".to_string(),
            title: value.title,
            protocol: value.protocol,
            size_bytes: value.size,
            indexer: value.indexer,
            download_url: value.download_url,
        }
    }
}
