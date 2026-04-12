use crate::metadata::openlibrary::OpenLibraryClient;
use anyhow::Context;
use serde_json::json;
use sqlx::Row;
use sqlx::SqlitePool;

pub struct BackfillWorker {
    pool: SqlitePool,
    open_library: OpenLibraryClient,
}

impl BackfillWorker {
    pub fn spawn(pool: SqlitePool, open_library: OpenLibraryClient) {
        let worker = Self { pool, open_library };
        tokio::spawn(async move {
            if let Err(e) = worker.run().await {
                tracing::error!(error = %e, "Backfill worker failed");
            }
        });
    }

    async fn run(self) -> anyhow::Result<()> {
        loop {
            match self.process_pending_backfills().await {
                Ok(count) if count > 0 => {
                    tracing::info!(count, "Backfilled requests");
                    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                }
                Ok(_) => {
                    // No pending work, sleep for an hour
                    tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Backfill iteration failed, retrying in 5m");
                    tokio::time::sleep(std::time::Duration::from_secs(300)).await;
                }
            }
        }
    }

    async fn process_pending_backfills(&self) -> anyhow::Result<usize> {
        let legacy_rows = sqlx::query(
            "SELECT id, title, author
             FROM requests
             WHERE external_work_id = ''",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut count = 0;
        for row in legacy_rows {
            let request_id = row.get::<String, _>("id");
            let title = row.get::<String, _>("title");
            let author = row.get::<String, _>("author");
            if let Err(error) = self
                .backfill_request_work_identity(&request_id, &title, &author)
                .await
            {
                tracing::error!(error = %error, request_id = %request_id, "legacy request backfill skipped");
            } else {
                count += 1;
            }
        }

        Ok(count)
    }

    async fn backfill_request_work_identity(
        &self,
        request_id: &str,
        title: &str,
        author: &str,
    ) -> anyhow::Result<()> {
        let resolved = self
            .open_library
            .resolve_work(title, author)
            .await
            .with_context(|| {
                format!("failed to resolve canonical work for request {request_id}")
            })?;
        let external_work_id = resolved.work.external_id.clone();
        let resolved_title = resolved.work.title.clone();
        let resolved_author = resolved.work.primary_author.clone();

        let event_row = sqlx::query(
            "SELECT id, payload_json
             FROM request_events
             WHERE request_id = ?
             ORDER BY id ASC
             LIMIT 1",
        )
        .bind(request_id)
        .fetch_one(&self.pool)
        .await
        .with_context(|| format!("missing request.created event for request {request_id}"))?;
        let event_id = event_row.get::<i64, _>("id");
        let payload_json = event_row.get::<String, _>("payload_json");
        let mut payload: serde_json::Value = serde_json::from_str(&payload_json)
            .with_context(|| format!("invalid request event payload for request {request_id}"))?;

        if let Some(object) = payload.as_object_mut() {
            object.insert("external_work_id".to_string(), json!(&external_work_id));
            object.insert("title".to_string(), json!(&resolved_title));
            object.insert("author".to_string(), json!(&resolved_author));
            object.insert(
                "work".to_string(),
                json!({
                    "external_id": &external_work_id,
                    "title": &resolved_title,
                    "author": &resolved_author,
                }),
            );
        }

        let mut tx = self.pool.begin().await?;

        sqlx::query("UPDATE requests SET external_work_id = ?, title = ?, author = ? WHERE id = ?")
            .bind(&external_work_id)
            .bind(&resolved_title)
            .bind(&resolved_author)
            .bind(request_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query("UPDATE request_events SET payload_json = ? WHERE id = ?")
            .bind(payload.to_string())
            .bind(event_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }
}
