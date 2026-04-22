use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use std::time::Duration;

use crate::{
    db::repositories::{SqliteRequestRepository, SqliteSettingsRepository},
    domain::{requests::MediaType, search::ReleaseCandidate, settings::PersistedImportSettings},
    downloads::qbittorrent::{QbittorrentClient, QbittorrentTorrent},
    notifications::send_notification,
    sync::{audiobookshelf::AudiobookshelfClient, calibre::CalibreHook},
    workers::{import_worker::ImportWorker, sync_worker::SyncWorker},
};

pub struct DownloadWorker;

impl DownloadWorker {
    pub fn spawn(pool: sqlx::SqlitePool, settings: SqliteSettingsRepository, interval: Duration) {
        tokio::spawn(async move {
            loop {
                if let Err(error) = Self::poll_once(pool.clone(), settings.clone()).await {
                    tracing::error!(error = %error, "download worker iteration failed");
                }
                tokio::time::sleep(interval).await;
            }
        });
    }

    pub async fn record_approved_candidate(
        repo: &SqliteRequestRepository,
        request_id: &str,
        candidate: &ReleaseCandidate,
    ) -> Result<()> {
        let request = repo
            .find_by_id(request_id)
            .await?
            .with_context(|| format!("request {request_id} not found"))?;

        repo.queue_download(request_id, candidate, category_for(&request.media_type))
            .await
    }

    pub async fn dispatch_approved_candidate(
        repo: &SqliteRequestRepository,
        client: &QbittorrentClient,
        request_id: &str,
        candidate: &ReleaseCandidate,
    ) -> Result<()> {
        let request = repo
            .find_by_id(request_id)
            .await?
            .with_context(|| format!("request {request_id} not found"))?;

        let download_url = candidate
            .download_url
            .as_deref()
            .context("approved candidate did not include a qBittorrent dispatch URL")?;

        let category = category_for(&request.media_type);
        client
            .add_magnet(download_url, request_id, category)
            .await?;
        repo.queue_download(request_id, candidate, category).await
    }

    pub async fn poll_qbittorrent_once(
        repo: &SqliteRequestRepository,
        settings_repo: &SqliteSettingsRepository,
        client: &QbittorrentClient,
        ebooks_root: &Path,
        audiobooks_root: &Path,
        import_settings: &PersistedImportSettings,
        calibre_hook: &CalibreHook,
        audiobookshelf_client: Option<&AudiobookshelfClient>,
        audiobookshelf_library_id: Option<&str>,
    ) -> Result<usize> {
        let queued = repo.queued_downloads().await?;
        let mut processed = 0usize;
        let mut completed_by_category: HashMap<String, Vec<QbittorrentTorrent>> = HashMap::new();

        for download in queued {
            if !completed_by_category.contains_key(&download.category) {
                let torrents = client.list(Some(&download.category)).await?;
                completed_by_category.insert(download.category.clone(), torrents);
            }

            let torrents = completed_by_category
                .get(&download.category)
                .context("missing qBittorrent category cache entry")?;

            let Some(torrent) = torrents
                .iter()
                .find(|torrent| torrent.has_tag(&download.request_id) && torrent.is_completed())
                .cloned()
            else {
                continue;
            };

            let request = repo
                .find_by_id(&download.request_id)
                .await?
                .with_context(|| format!("request {} not found", download.request_id))?;
            let files = collect_payload_files(&torrent)?;

            match request.media_type {
                MediaType::Ebook => {
                    let imported_path = ImportWorker::import_completed_ebook(
                        repo,
                        &download.request_id,
                        &files,
                        ebooks_root,
                        import_settings,
                    )
                    .await?;
                    SyncWorker::sync_ebook(
                        repo,
                        &download.request_id,
                        &imported_path,
                        calibre_hook,
                    )
                    .await?;
                    let _ = send_notification(
                        &settings_repo,
                        "request_imported",
                        "Athena import completed",
                        &format!("Ebook import completed for {}", download.request_id),
                        serde_json::json!({
                            "request_id": download.request_id,
                            "media_type": "ebook",
                            "path": imported_path,
                        }),
                    )
                    .await;
                    processed += 1;
                }
                MediaType::Audiobook => {
                    let imported_path = ImportWorker::import_completed_audiobook(
                        repo,
                        &download.request_id,
                        &files,
                        audiobooks_root,
                        import_settings,
                    )
                    .await?;
                    let client = audiobookshelf_client
                        .context("Audiobookshelf client is not configured for audiobook sync")?;
                    let library_id = audiobookshelf_library_id
                        .context("Audiobookshelf library id is not configured")?;
                    SyncWorker::sync_audiobook(
                        repo,
                        &download.request_id,
                        &imported_path,
                        client,
                        library_id,
                    )
                    .await?;
                    let _ = send_notification(
                        &settings_repo,
                        "request_imported",
                        "Athena import completed",
                        &format!("Audiobook import completed for {}", download.request_id),
                        serde_json::json!({
                            "request_id": download.request_id,
                            "media_type": "audiobook",
                            "path": imported_path,
                        }),
                    )
                    .await;
                    processed += 1;
                }
            }
        }

        Ok(processed)
    }

    pub async fn poll_once(
        pool: sqlx::SqlitePool,
        settings_repo: SqliteSettingsRepository,
    ) -> Result<usize> {
        let settings = settings_repo
            .get_persisted_runtime_settings()
            .await?
            .context("runtime settings row missing")?;
        let qb = &settings.download_clients.qbittorrent;
        if !qb.enabled {
            return Ok(0);
        }

        let client = QbittorrentClient::new(
            &qb.base_url,
            &qb.username,
            qb.password.clone().unwrap_or_default(),
        );
        let calibre_hook = CalibreHook::new(&settings.import.calibre_command);
        let audiobookshelf = settings.integrations.audiobookshelf.enabled.then(|| {
            AudiobookshelfClient::new(
                &settings.integrations.audiobookshelf.base_url,
                settings
                    .integrations
                    .audiobookshelf
                    .api_key
                    .clone()
                    .unwrap_or_default(),
            )
        });

        Self::poll_qbittorrent_once(
            &SqliteRequestRepository::new(pool),
            &settings_repo,
            &client,
            Path::new(&settings.storage.ebooks_root),
            Path::new(&settings.storage.audiobooks_root),
            &settings.import,
            &calibre_hook,
            audiobookshelf.as_ref(),
            Some(&settings.integrations.audiobookshelf.library_id),
        )
        .await
    }
}

fn category_for(media_type: &MediaType) -> &'static str {
    match media_type {
        MediaType::Ebook => "athena-ebooks",
        MediaType::Audiobook => "athena-audiobooks",
    }
}

fn collect_payload_files(torrent: &QbittorrentTorrent) -> Result<Vec<String>> {
    let root = PathBuf::from(torrent.content_root()?);

    if root.is_file() {
        return Ok(vec![root.display().to_string()]);
    }

    if !root.exists() {
        bail!(
            "qBittorrent payload path does not exist for torrent {}: {}",
            torrent.hash,
            root.display()
        );
    }

    if !root.is_dir() {
        bail!(
            "qBittorrent payload path is neither a file nor a directory for torrent {}: {}",
            torrent.hash,
            root.display()
        );
    }

    let mut files = Vec::new();
    collect_files_recursively(&root, &mut files)?;
    files.sort();

    if files.is_empty() {
        bail!(
            "qBittorrent payload directory was empty for torrent {}: {}",
            torrent.hash,
            root.display()
        );
    }

    Ok(files)
}

fn collect_files_recursively(root: &Path, files: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursively(&path, files)?;
        } else if path.is_file() {
            files.push(path.display().to_string());
        }
    }

    Ok(())
}
