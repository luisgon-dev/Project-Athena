use anyhow::{Context, Result, anyhow};
use serde_json::Value;

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
            base_url: base_url.into().trim_end_matches('/').to_string(),
            api_key: api_key.into(),
            http: reqwest::Client::new(),
        }
    }

    pub async fn search(&self, query: &str, media_type: &str) -> Result<Vec<ReleaseCandidate>> {
        let response = self
            .http
            .get(format!("{}/api/v1/search", self.base_url))
            .header("X-Api-Key", &self.api_key)
            .query(&[("query", query), ("type", media_type)])
            .send()
            .await?
            .error_for_status()?;

        let body = response.text().await?;
        if body.trim_start().starts_with('<') {
            return Err(anyhow!(
                "Prowlarr returned HTML instead of JSON for query {:?}. \
                 This usually means the request was redirected to a login page, \
                 the base URL is incorrect, or an authentication proxy intercepted the request. \
                 Please verify your Prowlarr base URL and API key.",
                query
            ));
        }
        let payload: Value = serde_json::from_str(&body).with_context(|| {
            format!(
                "error decoding response body for query {:?}: {}",
                query,
                snippet(&body)
            )
        })?;

        let items = payload.as_array().ok_or_else(|| {
            anyhow!(
                "unexpected prowlarr search payload for query {:?}: {}",
                query,
                snippet(&body)
            )
        })?;

        Ok(items
            .iter()
            .filter_map(release_candidate_from_value)
            .collect())
    }
}

fn release_candidate_from_value(value: &Value) -> Option<ReleaseCandidate> {
    let object = value.as_object()?;
    let external_id =
        string_field(object.get("guid")).or_else(|| string_field(object.get("infoUrl")))?;
    let title = string_field(object.get("title"))?;
    let protocol = string_field(object.get("protocol")).unwrap_or_else(|| "torrent".to_string());
    let indexer = string_field(object.get("indexer")).unwrap_or_else(|| "Prowlarr".to_string());
    let size_bytes = i64_field(object.get("size")).unwrap_or_default();
    let download_url = string_field(object.get("downloadUrl"));

    Some(
        ReleaseCandidate {
            external_id,
            source: "prowlarr".to_string(),
            title,
            protocol,
            size_bytes,
            indexer,
            download_url,
            narrator: None,
            graphic_audio: false,
            detected_language: None,
        }
        .with_parsed_metadata(),
    )
}

fn string_field(value: Option<&Value>) -> Option<String> {
    match value {
        Some(Value::String(value)) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        Some(Value::Number(value)) => Some(value.to_string()),
        _ => None,
    }
}

fn i64_field(value: Option<&Value>) -> Option<i64> {
    match value {
        Some(Value::Number(value)) => value
            .as_i64()
            .or_else(|| value.as_u64().and_then(|value| i64::try_from(value).ok())),
        Some(Value::String(value)) => value.trim().parse::<i64>().ok(),
        _ => None,
    }
}

fn snippet(body: &str) -> String {
    const MAX_LEN: usize = 240;
    let compact = body.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.len() <= MAX_LEN {
        compact
    } else {
        format!("{}...", &compact[..MAX_LEN])
    }
}
