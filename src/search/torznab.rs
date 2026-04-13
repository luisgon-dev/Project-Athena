use serde::Deserialize;

use crate::domain::search::ReleaseCandidate;

#[derive(Clone)]
pub struct TorznabClient {
    api_url: String,
    api_key: Option<String>,
    http: reqwest::Client,
}

impl TorznabClient {
    pub fn new(base_url: impl Into<String>, api_key: Option<String>) -> Self {
        Self::new_with_api_path(base_url, api_key, None::<String>)
    }

    pub fn new_with_api_path(
        base_url: impl Into<String>,
        api_key: Option<String>,
        api_path: Option<impl Into<String>>,
    ) -> Self {
        let base_url = base_url.into();
        let api_path = api_path
            .map(Into::into)
            .unwrap_or_else(|| "/api".to_string());
        let api_url = format!(
            "{}{}",
            base_url.trim_end_matches('/'),
            if api_path.starts_with('/') {
                api_path
            } else {
                format!("/{api_path}")
            }
        );

        Self {
            api_url,
            api_key,
            http: reqwest::Client::new(),
        }
    }

    pub async fn search(&self, query: &str) -> anyhow::Result<Vec<ReleaseCandidate>> {
        let mut request = self
            .http
            .get(&self.api_url)
            .query(&[("t", "search"), ("q", query)]);

        if let Some(api_key) = &self.api_key {
            request = request.query(&[("apikey", api_key)]);
        }

        let response = request.send().await?.error_for_status()?;
        let xml = response.text().await?;
        let feed: TorznabFeed = quick_xml::de::from_str(&xml)?;
        Ok(feed
            .channel
            .items
            .into_iter()
            .map(ReleaseCandidate::from)
            .collect())
    }
}

#[derive(Deserialize)]
struct TorznabFeed {
    channel: TorznabChannel,
}

#[derive(Deserialize)]
struct TorznabChannel {
    #[serde(rename = "item", default)]
    items: Vec<TorznabItem>,
}

#[derive(Deserialize)]
struct TorznabItem {
    guid: String,
    title: String,
    size: i64,
    link: Option<String>,
}

impl From<TorznabItem> for ReleaseCandidate {
    fn from(value: TorznabItem) -> Self {
        Self {
            external_id: value.guid,
            source: "torznab".to_string(),
            title: value.title,
            protocol: "torrent".to_string(),
            size_bytes: value.size,
            indexer: "torznab".to_string(),
            download_url: value.link,
        }
    }
}
