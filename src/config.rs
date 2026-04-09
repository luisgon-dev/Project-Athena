use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub bind_addr: String,
    pub ebooks_root: PathBuf,
    pub audiobooks_root: PathBuf,
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
        }
    }

    fn from_env_with<F>(get: F) -> anyhow::Result<Self>
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

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::AppConfig;

    #[test]
    fn from_env_with_uses_runtime_values() {
        let config = AppConfig::from_env_with(|key| match key {
            "BIND_ADDR" => Some("127.0.0.1:7777".into()),
            "EBOOKS_ROOT" => Some("/var/lib/books".into()),
            "AUDIOBOOKS_ROOT" => Some("/var/lib/audiobooks".into()),
            _ => None,
        })
        .unwrap();

        assert_eq!(config.bind_addr, "127.0.0.1:7777");
        assert_eq!(config.ebooks_root, PathBuf::from("/var/lib/books"));
        assert_eq!(config.audiobooks_root, PathBuf::from("/var/lib/audiobooks"));
    }
}
