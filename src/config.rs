use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DatabaseTarget {
    Memory,
    File(PathBuf),
}

impl DatabaseTarget {
    pub fn memory() -> Self {
        Self::Memory
    }

    pub fn file(path: impl Into<PathBuf>) -> Self {
        Self::File(path.into())
    }

    pub fn from_env_path(value: Option<String>) -> Self {
        value
            .map(PathBuf::from)
            .map(Self::File)
            .unwrap_or_else(|| Self::file("/data/book-router/book-router.sqlite"))
    }

    pub fn file_path(&self) -> Option<&Path> {
        match self {
            Self::Memory => None,
            Self::File(path) => Some(path.as_path()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QbittorrentConfig {
    pub base_url: String,
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub bind_addr: String,
    pub ebooks_root: PathBuf,
    pub audiobooks_root: PathBuf,
    pub database: DatabaseTarget,
    pub metadata_base_url: String,
    pub cover_base_url: String,
    pub qbittorrent: Option<QbittorrentConfig>,
    pub enable_fulfillment_workers: bool,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        Self::from_env_with(|key| std::env::var(key).ok())
    }

    pub fn for_tests() -> Self {
        Self {
            bind_addr: "127.0.0.1:0".into(),
            ebooks_root: "/ebooks".into(),
            audiobooks_root: "/audiobooks".into(),
            database: DatabaseTarget::memory(),
            metadata_base_url: "https://openlibrary.org".into(),
            cover_base_url: "https://covers.openlibrary.org".into(),
            qbittorrent: None,
            enable_fulfillment_workers: false,
        }
    }

    pub fn for_tests_with_database_path(path: impl Into<PathBuf>) -> Self {
        Self {
            database: DatabaseTarget::file(path),
            ..Self::for_tests()
        }
    }

    pub fn from_env_with<F>(get: F) -> anyhow::Result<Self>
    where
        F: Fn(&str) -> Option<String>,
    {
        let config = Self {
            bind_addr: get("BIND_ADDR").unwrap_or_else(|| "127.0.0.1:3000".into()),
            ebooks_root: get("EBOOKS_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/ebooks")),
            audiobooks_root: get("AUDIOBOOKS_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/audiobooks")),
            database: DatabaseTarget::from_env_path(get("DATABASE_PATH")),
            metadata_base_url: get("METADATA_BASE_URL")
                .unwrap_or_else(|| "https://openlibrary.org".into()),
            cover_base_url: get("COVER_BASE_URL")
                .unwrap_or_else(|| "https://covers.openlibrary.org".into()),
            qbittorrent: qbittorrent_from_env(&get),
            enable_fulfillment_workers: true,
        };

        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        for path in [&self.ebooks_root, &self.audiobooks_root] {
            if !path.is_absolute() {
                anyhow::bail!("media roots must be absolute paths");
            }
        }

        if let Some(path) = self.database.file_path() {
            if !path.is_absolute() {
                anyhow::bail!("sqlite database path must be an absolute path");
            }
        }

        if self.metadata_base_url.trim().is_empty() {
            anyhow::bail!("metadata base url must not be empty");
        }

        if self.cover_base_url.trim().is_empty() {
            anyhow::bail!("cover base url must not be empty");
        }

        if let Some(qbittorrent) = &self.qbittorrent {
            if qbittorrent.base_url.trim().is_empty() {
                anyhow::bail!("qbittorrent base url must not be empty");
            }
            if qbittorrent.username.trim().is_empty() {
                anyhow::bail!("qbittorrent username must not be empty");
            }
            if qbittorrent.password.trim().is_empty() {
                anyhow::bail!("qbittorrent password must not be empty");
            }
        }

        Ok(())
    }

    pub fn with_metadata_base_url(mut self, value: impl Into<String>) -> Self {
        self.metadata_base_url = value.into();
        self
    }

    pub fn with_cover_base_url(mut self, value: impl Into<String>) -> Self {
        self.cover_base_url = value.into();
        self
    }

    pub fn with_fulfillment_workers_enabled(mut self, value: bool) -> Self {
        self.enable_fulfillment_workers = value;
        self
    }
}

fn qbittorrent_from_env<F>(get: &F) -> Option<QbittorrentConfig>
where
    F: Fn(&str) -> Option<String>,
{
    let base_url = get("QBITTORRENT_BASE_URL")?;
    let username = get("QBITTORRENT_USERNAME").unwrap_or_default();
    let password = get("QBITTORRENT_PASSWORD").unwrap_or_default();

    Some(QbittorrentConfig {
        base_url,
        username,
        password,
    })
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{AppConfig, DatabaseTarget};

    #[test]
    fn from_env_with_uses_runtime_values() {
        let config = AppConfig::from_env_with(|key| match key {
            "BIND_ADDR" => Some("127.0.0.1:7777".into()),
            "EBOOKS_ROOT" => Some("/var/lib/books".into()),
            "AUDIOBOOKS_ROOT" => Some("/var/lib/audiobooks".into()),
            "DATABASE_PATH" => Some("/var/lib/book-router/book-router.sqlite".into()),
            "METADATA_BASE_URL" => Some("https://metadata.example.test".into()),
            "COVER_BASE_URL" => Some("https://covers.example.test".into()),
            _ => None,
        })
        .unwrap();

        assert_eq!(config.bind_addr, "127.0.0.1:7777");
        assert_eq!(config.ebooks_root, PathBuf::from("/var/lib/books"));
        assert_eq!(config.audiobooks_root, PathBuf::from("/var/lib/audiobooks"));
        assert_eq!(config.metadata_base_url, "https://metadata.example.test");
        assert_eq!(config.cover_base_url, "https://covers.example.test");
        assert!(matches!(
            config.database,
            DatabaseTarget::File(ref path)
                if path == Path::new("/var/lib/book-router/book-router.sqlite")
        ));
    }
}
