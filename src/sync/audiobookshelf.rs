use anyhow::Result;
use serde::Deserialize;

#[derive(Clone)]
pub struct AudiobookshelfClient {
    base_url: String,
    api_key: String,
    http: reqwest::Client,
}

impl AudiobookshelfClient {
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            api_key: api_key.into(),
            http: reqwest::Client::new(),
        }
    }

    pub fn scan_library(&self, library_id: &str) -> reqwest::RequestBuilder {
        self.http
            .post(format!("{}/api/libraries/{library_id}/scan", self.base_url))
            .bearer_auth(&self.api_key)
    }

    pub async fn search_library(
        &self,
        library_id: &str,
        query: &str,
    ) -> Result<Vec<AudiobookshelfLibraryItem>> {
        let response = self
            .http
            .get(format!("{}/api/libraries/{library_id}/search", self.base_url))
            .bearer_auth(&self.api_key)
            .query(&[("q", query)])
            .send()
            .await?
            .error_for_status()?;

        let payload = response.json::<AudiobookshelfSearchResponse>().await?;
        Ok(payload.book)
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct AudiobookshelfSearchResponse {
    #[serde(default)]
    pub book: Vec<AudiobookshelfLibraryItem>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct AudiobookshelfLibraryItem {
    #[serde(rename = "libraryItem")]
    pub library_item: AudiobookshelfLibraryItemData,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct AudiobookshelfLibraryItemData {
    pub id: String,
    pub media: AudiobookshelfMedia,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct AudiobookshelfMedia {
    pub metadata: AudiobookshelfMetadata,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudiobookshelfMetadata {
    pub title: String,
    pub author_name: Option<String>,
    pub narrator_name: Option<String>,
    pub series_name: Option<String>,
}
