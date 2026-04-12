use std::path::{Path, PathBuf};
use std::{fs, io};

use anyhow::{Context, Result, bail};

use crate::{
    db::repositories::SqliteRequestRepository,
    domain::{imports::ImportMediaType, requests::MediaType},
    importer::{classify::classify_payload, move_plan::build_move_plan},
};

pub struct ImportWorker;

impl ImportWorker {
    pub async fn import_completed_ebook(
        repo: &SqliteRequestRepository,
        request_id: &str,
        files: &[String],
        ebooks_root: &Path,
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

        let destination = build_move_plan(ebooks_root, &request.author, &request.title, &leaf_name);
        move_file(&PathBuf::from(&source_file), &destination)?;

        repo.complete_download(request_id, files).await?;
        repo.mark_import_succeeded(request_id, &destination).await?;

        Ok(destination)
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
