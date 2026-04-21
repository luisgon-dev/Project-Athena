use std::path::{Path, PathBuf};
use std::{fs, io};

use anyhow::{Context, Result, bail};

use crate::{
    db::repositories::SqliteRequestRepository,
    domain::{
        imports::ImportMediaType,
        requests::MediaType,
        settings::{EbookImportMode, PersistedImportSettings},
    },
    importer::{
        classify::classify_payload,
        move_plan::{build_audiobook_root, build_ebook_move_plan, normalize_path_segment},
    },
};

pub struct ImportWorker;

impl ImportWorker {
    pub async fn import_completed_ebook(
        repo: &SqliteRequestRepository,
        request_id: &str,
        files: &[String],
        ebooks_root: &Path,
        import_settings: &PersistedImportSettings,
    ) -> Result<PathBuf> {
        let request = repo
            .find_by_id(request_id)
            .await?
            .with_context(|| format!("request {request_id} not found"))?;

        if request.media_type != MediaType::Ebook {
            bail!("request {request_id} is not an ebook request");
        }

        let classification = classify_payload(files);
        if classification.media_type != ImportMediaType::Ebook {
            bail!("download payload was not classified as ebook");
        }

        let source_file = files
            .iter()
            .find(|file| {
                file.ends_with(".epub") || file.ends_with(".pdf") || file.ends_with(".azw3")
            })
            .cloned()
            .context("ebook payload did not contain a supported source file")?;

        let leaf_name = Path::new(&source_file)
            .file_name()
            .map(|file_name| file_name.to_string_lossy().to_string())
            .context("ebook payload did not contain a supported file leaf")?;

        let destination = match import_settings.ebook_import_mode {
            EbookImportMode::Managed => build_ebook_move_plan(
                ebooks_root,
                &import_settings.ebook_naming_template,
                &request.author,
                &request.title,
                &leaf_name,
            ),
            EbookImportMode::Passthrough => {
                let passthrough_root = import_settings
                    .ebook_passthrough_root
                    .as_deref()
                    .context("ebook passthrough root is not configured")?;
                PathBuf::from(passthrough_root)
                    .join(request_id)
                    .join(normalize_path_segment(&leaf_name))
            }
        };
        move_file(&PathBuf::from(&source_file), &destination)?;

        repo.complete_download(request_id, files).await?;
        repo.mark_import_succeeded(request_id, &destination).await?;

        Ok(destination)
    }

    pub async fn import_completed_audiobook(
        repo: &SqliteRequestRepository,
        request_id: &str,
        files: &[String],
        audiobooks_root: &Path,
        import_settings: &PersistedImportSettings,
    ) -> Result<PathBuf> {
        let request = repo
            .find_by_id(request_id)
            .await?
            .with_context(|| format!("request {request_id} not found"))?;

        if request.media_type != MediaType::Audiobook {
            bail!("request {request_id} is not an audiobook request");
        }

        let classification = classify_payload(files);
        if classification.media_type != ImportMediaType::Audiobook {
            bail!("download payload was not classified as audiobook");
        }

        let source_paths = files.iter().map(PathBuf::from).collect::<Vec<_>>();
        let destination_dir = build_audiobook_root(
            audiobooks_root,
            &import_settings.audiobook_layout_preset,
            &request.author,
            &request.title,
        );
        let common_root = common_parent_dir(&source_paths)
            .context("audiobook payload did not have a common source root")?;

        for source in &source_paths {
            let relative = source
                .strip_prefix(&common_root)
                .ok()
                .filter(|path| !path.as_os_str().is_empty())
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    source
                        .file_name()
                        .map(PathBuf::from)
                        .unwrap_or_else(|| PathBuf::from("audio.bin"))
                });
            let destination = destination_dir.join(sanitize_relative_path(&relative));
            move_file(source, &destination)?;
        }

        repo.complete_download(request_id, files).await?;
        repo.mark_import_succeeded(request_id, &destination_dir)
            .await?;

        Ok(destination_dir)
    }
}

fn move_file(source: &Path, destination: &Path) -> Result<()> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }

    match fs::rename(source, destination) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::CrossesDevices => {
            fs::copy(source, destination)?;
            fs::remove_file(source)?;
            Ok(())
        }
        Err(error) => Err(error.into()),
    }
}

fn common_parent_dir(paths: &[PathBuf]) -> Option<PathBuf> {
    let first_parent = paths.first()?.parent()?.to_path_buf();
    let mut common = first_parent;

    while !paths.iter().all(|path| path.starts_with(&common)) {
        common = common.parent()?.to_path_buf();
    }

    Some(common)
}

fn sanitize_relative_path(path: &Path) -> PathBuf {
    let mut sanitized = PathBuf::new();
    for component in path.components() {
        let segment = component.as_os_str().to_string_lossy();
        sanitized.push(sanitize_segment(&segment));
    }
    sanitized
}

fn sanitize_segment(value: &str) -> String {
    normalize_path_segment(value)
}
