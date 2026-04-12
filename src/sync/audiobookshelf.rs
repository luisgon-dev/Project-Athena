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
}
