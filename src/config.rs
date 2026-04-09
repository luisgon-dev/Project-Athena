use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub bind_addr: String,
    pub ebooks_root: PathBuf,
    pub audiobooks_root: PathBuf,
}

impl AppConfig {
    pub fn for_tests() -> Self {
        Self {
            bind_addr: "127.0.0.1:0".into(),
            ebooks_root: "/ebooks".into(),
            audiobooks_root: "/audiobooks".into(),
        }
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
