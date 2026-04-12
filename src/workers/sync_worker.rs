use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::{
    db::repositories::SqliteRequestRepository,
    sync::calibre::CalibreHook,
};

pub struct SyncWorker;

impl SyncWorker {
    pub async fn sync_ebook(
        repo: &SqliteRequestRepository,
        request_id: &str,
        imported_path: &Path,
        hook: &CalibreHook,
    ) -> Result<()> {
        let request = repo
            .find_by_id(request_id)
            .await?
            .with_context(|| format!("request {request_id} not found"))?;

        if request.state != "imported" {
            bail!("request {request_id} must be imported before sync");
        }

        let status = hook
            .add_book_command(&imported_path.to_string_lossy())
            .status()
            .with_context(|| format!("failed to execute calibre hook for request {request_id}"))?;

        if !status.success() {
            bail!("calibre hook failed for request {request_id}");
        }

        repo.mark_sync_succeeded(request_id, imported_path).await
    }
}
