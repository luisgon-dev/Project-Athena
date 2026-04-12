use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};

use crate::{
    db::repositories::SqliteRequestRepository,
    domain::{requests::MediaType, search::ReleaseCandidate},
    downloads::qbittorrent::{QbittorrentClient, QbittorrentTorrent},
    sync::calibre::CalibreHook,
    workers::{import_worker::ImportWorker, sync_worker::SyncWorker},
};

pub struct DownloadWorker;

impl DownloadWorker {
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
        client: &QbittorrentClient,
        ebooks_root: &Path,
        calibre_hook: &CalibreHook,
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
                    )
                    .await?;
                    SyncWorker::sync_ebook(
                        repo,
                        &download.request_id,
                        &imported_path,
                        calibre_hook,
                    )
                    .await?;
                    processed += 1;
                }
                MediaType::Audiobook => {
                    bail!("qBittorrent polling for audiobook imports is not implemented yet")
                }
            }
        }

        Ok(processed)
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
