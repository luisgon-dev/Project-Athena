use anyhow::{Result, anyhow, bail};
use reqwest::{
    Client, Url,
    header::{HeaderMap, HeaderValue, ORIGIN, REFERER},
};
use serde::Deserialize;

#[derive(Clone)]
pub struct QbittorrentClient {
    base_url: String,
    username: String,
    password: String,
    http: Client,
}

impl QbittorrentClient {
    pub fn new(
        base_url: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        let base_url = base_url.into().trim_end_matches('/').to_string();
        let url = Url::parse(&base_url).expect("valid qBittorrent base URL");
        let origin = origin_from_url(&url);

        let mut headers = HeaderMap::new();
        headers.insert(
            ORIGIN,
            HeaderValue::from_str(&origin).expect("valid qBittorrent origin header"),
        );
        headers.insert(
            REFERER,
            HeaderValue::from_str(&format!("{origin}/")).expect("valid qBittorrent referer header"),
        );

        let http = Client::builder()
            .cookie_store(true)
            .default_headers(headers)
            .build()
            .expect("qBittorrent reqwest client");

        Self {
            base_url,
            username: username.into(),
            password: password.into(),
            http,
        }
    }

    pub async fn add_magnet(
        &self,
        magnet: &str,
        request_id: &str,
        category: &str,
    ) -> anyhow::Result<()> {
        self.login().await?;

        let form = [
            ("urls", magnet),
            ("category", category),
            ("tags", request_id),
        ];

        self.http
            .post(self.api_url("/api/v2/torrents/add"))
            .form(&form)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn list(&self, category: Option<&str>) -> anyhow::Result<Vec<QbittorrentTorrent>> {
        self.login().await?;

        let mut request = self.http.get(self.api_url("/api/v2/torrents/info"));
        if let Some(category) = category {
            request = request.query(&[("category", category)]);
        }

        let response = request.send().await?.error_for_status()?;

        let torrents = response.json::<Vec<QbittorrentTorrent>>().await?;
        Ok(torrents)
    }

    pub async fn completed_for_tag(
        &self,
        request_id: &str,
        category: &str,
    ) -> anyhow::Result<Vec<QbittorrentTorrent>> {
        let torrents = self.list(Some(category)).await?;
        Ok(torrents
            .into_iter()
            .filter(|torrent| torrent.has_tag(request_id))
            .filter(QbittorrentTorrent::is_completed)
            .collect())
    }

    fn api_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
    async fn login(&self) -> Result<()> {
        let response = self
            .http
            .post(self.api_url("/api/v2/auth/login"))
            .form(&[
                ("username", self.username.as_str()),
                ("password", self.password.as_str()),
            ])
            .send()
            .await?
            .error_for_status()?;

        let body = response.text().await?;
        if !body.trim().is_empty() && body.trim() != "Ok." {
            bail!("qBittorrent login failed: {}", body.trim());
        }

        Ok(())
    }
}

fn origin_from_url(url: &Url) -> String {
    let host = url.host_str().expect("qBittorrent base URL host");
    match url.port() {
        Some(port) => format!("{}://{}:{}", url.scheme(), host, port),
        None => format!("{}://{}", url.scheme(), host),
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct QbittorrentTorrent {
    #[serde(default)]
    pub hash: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub tags: String,
    #[serde(default)]
    pub progress: f32,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub content_path: String,
    #[serde(default)]
    pub save_path: String,
}

impl QbittorrentTorrent {
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags
            .split(',')
            .any(|candidate| candidate.trim() == tag)
    }

    pub fn is_completed(&self) -> bool {
        matches!(
            self.state.as_str(),
            "pausedUP" | "stoppedUP" | "uploading" | "stalledUP" | "queuedUP" | "forcedUP"
        ) || (self.progress >= 1.0 && !self.state.is_empty() && self.state != "checkingUP")
    }

    pub fn content_root(&self) -> Result<&str> {
        if !self.content_path.trim().is_empty() {
            return Ok(self.content_path.as_str());
        }

        if !self.save_path.trim().is_empty() {
            return Ok(self.save_path.as_str());
        }

        Err(anyhow!(
            "qBittorrent torrent {} did not include a content path",
            self.hash
        ))
    }
}
