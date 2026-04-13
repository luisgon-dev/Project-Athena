use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::config::AppConfig;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StorageSettingsRecord {
    pub ebooks_root: String,
    pub audiobooks_root: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StorageSettingsUpdate {
    pub ebooks_root: Option<String>,
    pub audiobooks_root: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MetadataSettingsRecord {
    pub base_url: String,
    pub cover_base_url: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MetadataSettingsUpdate {
    pub base_url: Option<String>,
    pub cover_base_url: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct QbittorrentSettingsRecord {
    pub enabled: bool,
    pub base_url: String,
    pub username: String,
    pub category_ebook: String,
    pub category_audiobook: String,
    pub has_password: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct QbittorrentSettingsUpdate {
    pub enabled: Option<bool>,
    pub base_url: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    #[serde(default)]
    pub clear_password: bool,
    pub category_ebook: Option<String>,
    pub category_audiobook: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProwlarrIntegrationRecord {
    pub enabled: bool,
    pub sync_enabled: bool,
    pub base_url: String,
    pub has_api_key: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProwlarrIntegrationUpdate {
    pub enabled: Option<bool>,
    pub sync_enabled: Option<bool>,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    #[serde(default)]
    pub clear_api_key: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ImportSettingsRecord {
    pub naming_template: String,
    pub calibre_command: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ImportSettingsUpdate {
    pub naming_template: Option<String>,
    pub calibre_command: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AcquisitionSettingsRecord {
    pub minimum_score: f32,
    pub auto_acquire_score: f32,
    pub preferred_language: Option<String>,
    pub blocked_terms: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AcquisitionSettingsUpdate {
    pub minimum_score: Option<f32>,
    pub auto_acquire_score: Option<f32>,
    pub preferred_language: Option<String>,
    pub blocked_terms: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DownloadClientSettingsRecord {
    pub qbittorrent: QbittorrentSettingsRecord,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DownloadClientSettingsUpdate {
    pub qbittorrent: Option<QbittorrentSettingsUpdate>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct IntegrationSettingsRecord {
    pub prowlarr: ProwlarrIntegrationRecord,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct IntegrationSettingsUpdate {
    pub prowlarr: Option<ProwlarrIntegrationUpdate>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RuntimeSettingsRecord {
    pub storage: StorageSettingsRecord,
    pub metadata: MetadataSettingsRecord,
    pub download_clients: DownloadClientSettingsRecord,
    pub integrations: IntegrationSettingsRecord,
    pub import: ImportSettingsRecord,
    pub acquisition: AcquisitionSettingsRecord,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RuntimeSettingsUpdate {
    pub storage: Option<StorageSettingsUpdate>,
    pub metadata: Option<MetadataSettingsUpdate>,
    pub download_clients: Option<DownloadClientSettingsUpdate>,
    pub integrations: Option<IntegrationSettingsUpdate>,
    pub import: Option<ImportSettingsUpdate>,
    pub acquisition: Option<AcquisitionSettingsUpdate>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SyncedIndexerRecord {
    pub id: i64,
    pub prowlarr_indexer_id: Option<i64>,
    pub name: String,
    pub enabled: bool,
    pub implementation: String,
    pub protocol: Option<String>,
    pub base_url: Option<String>,
    pub categories: Vec<i64>,
    pub last_synced_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ConnectionTestResult {
    pub ok: bool,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedQbittorrentSettings {
    pub enabled: bool,
    pub base_url: String,
    pub username: String,
    pub password: Option<String>,
    pub category_ebook: String,
    pub category_audiobook: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedProwlarrIntegrationSettings {
    pub enabled: bool,
    pub sync_enabled: bool,
    pub base_url: String,
    pub api_key: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedImportSettings {
    pub naming_template: String,
    pub calibre_command: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PersistedAcquisitionSettings {
    pub minimum_score: f32,
    pub auto_acquire_score: f32,
    pub preferred_language: Option<String>,
    pub blocked_terms: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PersistedRuntimeSettings {
    pub storage: StorageSettingsRecord,
    pub metadata: MetadataSettingsRecord,
    pub download_clients: PersistedDownloadClientSettings,
    pub integrations: PersistedIntegrationSettings,
    pub import: PersistedImportSettings,
    pub acquisition: PersistedAcquisitionSettings,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedDownloadClientSettings {
    pub qbittorrent: PersistedQbittorrentSettings,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedIntegrationSettings {
    pub prowlarr: PersistedProwlarrIntegrationSettings,
}

impl PersistedRuntimeSettings {
    pub fn from_config(config: &AppConfig) -> Self {
        let qb = config.qbittorrent.clone();

        Self {
            storage: StorageSettingsRecord {
                ebooks_root: config.ebooks_root.display().to_string(),
                audiobooks_root: config.audiobooks_root.display().to_string(),
            },
            metadata: MetadataSettingsRecord {
                base_url: config.metadata_base_url.clone(),
                cover_base_url: config.cover_base_url.clone(),
            },
            download_clients: PersistedDownloadClientSettings {
                qbittorrent: PersistedQbittorrentSettings {
                    enabled: qb.is_some(),
                    base_url: qb
                        .as_ref()
                        .map(|value| value.base_url.clone())
                        .unwrap_or_default(),
                    username: qb
                        .as_ref()
                        .map(|value| value.username.clone())
                        .unwrap_or_default(),
                    password: qb.as_ref().map(|value| value.password.clone()),
                    category_ebook: "athena-ebooks".to_string(),
                    category_audiobook: "athena-audiobooks".to_string(),
                },
            },
            integrations: PersistedIntegrationSettings {
                prowlarr: PersistedProwlarrIntegrationSettings {
                    enabled: false,
                    sync_enabled: false,
                    base_url: String::new(),
                    api_key: None,
                },
            },
            import: PersistedImportSettings {
                naming_template: "{author}/{title}/{title}".to_string(),
                calibre_command: "calibredb".to_string(),
            },
            acquisition: PersistedAcquisitionSettings {
                minimum_score: 0.7,
                auto_acquire_score: 0.93,
                preferred_language: None,
                blocked_terms: Vec::new(),
            },
        }
    }

    pub fn to_record(&self) -> RuntimeSettingsRecord {
        RuntimeSettingsRecord {
            storage: self.storage.clone(),
            metadata: self.metadata.clone(),
            download_clients: DownloadClientSettingsRecord {
                qbittorrent: QbittorrentSettingsRecord {
                    enabled: self.download_clients.qbittorrent.enabled,
                    base_url: self.download_clients.qbittorrent.base_url.clone(),
                    username: self.download_clients.qbittorrent.username.clone(),
                    category_ebook: self.download_clients.qbittorrent.category_ebook.clone(),
                    category_audiobook: self
                        .download_clients
                        .qbittorrent
                        .category_audiobook
                        .clone(),
                    has_password: self.download_clients.qbittorrent.password.is_some(),
                },
            },
            integrations: IntegrationSettingsRecord {
                prowlarr: ProwlarrIntegrationRecord {
                    enabled: self.integrations.prowlarr.enabled,
                    sync_enabled: self.integrations.prowlarr.sync_enabled,
                    base_url: self.integrations.prowlarr.base_url.clone(),
                    has_api_key: self.integrations.prowlarr.api_key.is_some(),
                },
            },
            import: ImportSettingsRecord {
                naming_template: self.import.naming_template.clone(),
                calibre_command: self.import.calibre_command.clone(),
            },
            acquisition: AcquisitionSettingsRecord {
                minimum_score: self.acquisition.minimum_score,
                auto_acquire_score: self.acquisition.auto_acquire_score,
                preferred_language: self.acquisition.preferred_language.clone(),
                blocked_terms: self.acquisition.blocked_terms.clone(),
            },
        }
    }

    pub fn apply_update(&mut self, update: RuntimeSettingsUpdate) {
        if let Some(storage) = update.storage {
            if let Some(value) = storage.ebooks_root {
                self.storage.ebooks_root = value;
            }
            if let Some(value) = storage.audiobooks_root {
                self.storage.audiobooks_root = value;
            }
        }

        if let Some(metadata) = update.metadata {
            if let Some(value) = metadata.base_url {
                self.metadata.base_url = value;
            }
            if let Some(value) = metadata.cover_base_url {
                self.metadata.cover_base_url = value;
            }
        }

        if let Some(download_clients) = update.download_clients
            && let Some(qb) = download_clients.qbittorrent
        {
            if let Some(value) = qb.enabled {
                self.download_clients.qbittorrent.enabled = value;
            }
            if let Some(value) = qb.base_url {
                self.download_clients.qbittorrent.base_url = value;
            }
            if let Some(value) = qb.username {
                self.download_clients.qbittorrent.username = value;
            }
            if let Some(value) = qb.category_ebook {
                self.download_clients.qbittorrent.category_ebook = value;
            }
            if let Some(value) = qb.category_audiobook {
                self.download_clients.qbittorrent.category_audiobook = value;
            }
            if qb.clear_password {
                self.download_clients.qbittorrent.password = None;
            } else if let Some(value) = qb.password {
                self.download_clients.qbittorrent.password = if value.is_empty() {
                    self.download_clients.qbittorrent.password.clone()
                } else {
                    Some(value)
                };
            }
        }

        if let Some(integrations) = update.integrations
            && let Some(prowlarr) = integrations.prowlarr
        {
            if let Some(value) = prowlarr.enabled {
                self.integrations.prowlarr.enabled = value;
            }
            if let Some(value) = prowlarr.sync_enabled {
                self.integrations.prowlarr.sync_enabled = value;
            }
            if let Some(value) = prowlarr.base_url {
                self.integrations.prowlarr.base_url = value;
            }
            if prowlarr.clear_api_key {
                self.integrations.prowlarr.api_key = None;
            } else if let Some(value) = prowlarr.api_key {
                self.integrations.prowlarr.api_key = if value.is_empty() {
                    self.integrations.prowlarr.api_key.clone()
                } else {
                    Some(value)
                };
            }
        }

        if let Some(import_settings) = update.import {
            if let Some(value) = import_settings.naming_template {
                self.import.naming_template = value;
            }
            if let Some(value) = import_settings.calibre_command {
                self.import.calibre_command = value;
            }
        }

        if let Some(acquisition) = update.acquisition {
            if let Some(value) = acquisition.minimum_score {
                self.acquisition.minimum_score = value;
            }
            if let Some(value) = acquisition.auto_acquire_score {
                self.acquisition.auto_acquire_score = value;
            }
            if let Some(value) = acquisition.preferred_language {
                self.acquisition.preferred_language = normalize_optional_text(Some(value));
            }
            if let Some(value) = acquisition.blocked_terms {
                self.acquisition.blocked_terms = value;
            }
        }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        validate_absolute_path(&self.storage.ebooks_root, "ebooks root")?;
        validate_absolute_path(&self.storage.audiobooks_root, "audiobooks root")?;
        validate_url(&self.metadata.base_url, "metadata base url")?;
        validate_url(&self.metadata.cover_base_url, "cover base url")?;

        let qb = &self.download_clients.qbittorrent;
        if qb.enabled {
            validate_url(&qb.base_url, "qBittorrent base url")?;
            require_non_empty(&qb.username, "qBittorrent username")?;
            if qb.password.as_deref().unwrap_or_default().trim().is_empty() {
                anyhow::bail!("qBittorrent password must not be empty when enabled");
            }
        }
        require_non_empty(&qb.category_ebook, "qBittorrent ebook category")?;
        require_non_empty(&qb.category_audiobook, "qBittorrent audiobook category")?;

        let prowlarr = &self.integrations.prowlarr;
        if prowlarr.enabled || prowlarr.sync_enabled {
            validate_url(&prowlarr.base_url, "Prowlarr base url")?;
            if prowlarr
                .api_key
                .as_deref()
                .unwrap_or_default()
                .trim()
                .is_empty()
            {
                anyhow::bail!("Prowlarr api key must not be empty when enabled");
            }
        }

        validate_naming_template(&self.import.naming_template)?;

        if !(0.0..=1.0).contains(&self.acquisition.minimum_score) {
            anyhow::bail!("minimum score must be between 0.0 and 1.0");
        }
        if !(0.0..=1.0).contains(&self.acquisition.auto_acquire_score) {
            anyhow::bail!("auto acquire score must be between 0.0 and 1.0");
        }
        if self.acquisition.auto_acquire_score < self.acquisition.minimum_score {
            anyhow::bail!("auto acquire score must be greater than or equal to minimum score");
        }

        Ok(())
    }
}

fn validate_absolute_path(value: &str, label: &str) -> anyhow::Result<()> {
    let path = std::path::Path::new(value);
    if !path.is_absolute() {
        anyhow::bail!("{label} must be an absolute path");
    }

    Ok(())
}

fn validate_url(value: &str, label: &str) -> anyhow::Result<()> {
    if value.trim().is_empty() {
        anyhow::bail!("{label} must not be empty");
    }

    reqwest::Url::parse(value.trim())
        .map_err(|_| anyhow::anyhow!("{label} must be a valid URL"))?;
    Ok(())
}

fn require_non_empty(value: &str, label: &str) -> anyhow::Result<()> {
    if value.trim().is_empty() {
        anyhow::bail!("{label} must not be empty");
    }

    Ok(())
}

fn validate_naming_template(value: &str) -> anyhow::Result<()> {
    require_non_empty(value, "import naming template")?;

    if !value.contains("{author}") || !value.contains("{title}") {
        anyhow::bail!("import naming template must include both {{author}} and {{title}}");
    }

    Ok(())
}

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}
