use anyhow::{Context, Result};
use serde_json::{Value, json};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::config::AppConfig;
use crate::domain::{
    events::{RequestEventKind, RequestEventRecord},
    requests::{CreateRequest, RequestListRecord, RequestRecord},
    search::{ReleaseCandidate, ReviewQueueEntry, ScoredCandidate},
    settings::{
        PersistedRuntimeSettings, RuntimeSettingsRecord, RuntimeSettingsUpdate, SyncedIndexerRecord,
    },
};

pub struct SqliteRequestRepository {
    pool: SqlitePool,
}

#[derive(Clone)]
pub struct SqliteSettingsRepository {
    pool: SqlitePool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueuedDownloadRecord {
    pub request_id: String,
    pub category: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchIndexerRecord {
    pub id: i64,
    pub implementation: String,
    pub protocol: Option<String>,
    pub base_url: String,
    pub api_path: Option<String>,
    pub api_key: Option<String>,
    pub enabled: bool,
}

impl SqliteRequestRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, request: CreateRequest) -> Result<RequestRecord> {
        let mut created = self.create_batch(vec![request]).await?;
        created
            .pop()
            .ok_or_else(|| anyhow::anyhow!("create_batch returned no requests"))
    }

    pub async fn create_batch(&self, requests: Vec<CreateRequest>) -> Result<Vec<RequestRecord>> {
        if requests.is_empty() {
            return Ok(Vec::new());
        }

        let mut tx = self.pool.begin().await?;
        let mut created = Vec::with_capacity(requests.len());

        for request in requests {
            if request.external_work_id.trim().is_empty() {
                anyhow::bail!("external work id cannot be empty");
            }

            let id = Uuid::new_v4().to_string();
            let external_work_id = request.external_work_id.clone();
            let title = request.title.clone();
            let author = request.author.clone();
            let media_type = request.media_type.as_str();
            let preferred_language = request.preferred_language.clone();
            let manifestation = request.manifestation.clone();

            sqlx::query(
                "INSERT INTO requests (id, external_work_id, title, author, media_type, preferred_language, state, created_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)",
            )
            .bind(&id)
            .bind(&external_work_id)
            .bind(&title)
            .bind(&author)
            .bind(media_type)
            .bind(preferred_language.as_deref())
            .bind("requested")
            .execute(&mut *tx)
            .await?;

            let payload_json = json!({
                "request_id": id,
                "external_work_id": &external_work_id,
                "work": {
                    "external_id": &external_work_id,
                    "title": &title,
                    "author": &author,
                },
                "title": &title,
                "author": &author,
                "media_type": media_type,
                "preferred_language": preferred_language,
                "manifestation": {
                    "edition_title": manifestation.edition_title,
                    "preferred_narrator": manifestation.preferred_narrator,
                    "preferred_publisher": manifestation.preferred_publisher,
                    "graphic_audio": manifestation.graphic_audio,
                }
            })
            .to_string();

            sqlx::query(
                "INSERT INTO request_events (request_id, kind, payload_json, created_at)
                 VALUES (?, ?, ?, CURRENT_TIMESTAMP)",
            )
            .bind(&id)
            .bind(RequestEventKind::Created.as_str())
            .bind(payload_json)
            .execute(&mut *tx)
            .await?;

            let row = sqlx::query(
                "SELECT id, external_work_id, title, author, media_type, preferred_language, state, created_at
                 FROM requests
                 WHERE id = ?",
            )
            .bind(&id)
            .fetch_one(&mut *tx)
            .await?;

            created.push(RequestRecord {
                id: row.get::<String, _>("id"),
                external_work_id: row.get::<String, _>("external_work_id"),
                title: row.get::<String, _>("title"),
                author: row.get::<String, _>("author"),
                media_type: match crate::domain::requests::MediaType::from_str(
                    row.get::<String, _>("media_type").as_str(),
                ) {
                    Some(media_type) => media_type,
                    None => anyhow::bail!("unknown media type stored in requests"),
                },
                preferred_language: row.get::<Option<String>, _>("preferred_language"),
                manifestation,
                state: row.get::<String, _>("state"),
                created_at: row.get::<String, _>("created_at"),
            });
        }

        tx.commit().await?;

        Ok(created)
    }

    pub async fn find_by_id(&self, request_id: impl AsRef<str>) -> Result<Option<RequestRecord>> {
        let row = sqlx::query(
            "SELECT id, external_work_id, title, author, media_type, preferred_language, state, created_at
             FROM requests
             WHERE id = ?",
        )
        .bind(request_id.as_ref())
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let payload_json = sqlx::query(
            "SELECT payload_json FROM request_events WHERE request_id = ? ORDER BY id ASC LIMIT 1",
        )
        .bind(request_id.as_ref())
        .fetch_one(&self.pool)
        .await?
        .get::<String, _>("payload_json");

        let payload: serde_json::Value = serde_json::from_str(&payload_json)?;
        let manifestation = manifestation_from_payload(&payload);

        Ok(Some(RequestRecord {
            id: row.get::<String, _>("id"),
            external_work_id: row.get::<String, _>("external_work_id"),
            title: row.get::<String, _>("title"),
            author: row.get::<String, _>("author"),
            media_type: match crate::domain::requests::MediaType::from_str(
                row.get::<String, _>("media_type").as_str(),
            ) {
                Some(media_type) => media_type,
                None => anyhow::bail!("unknown media type stored in requests"),
            },
            preferred_language: row.get::<Option<String>, _>("preferred_language"),
            manifestation,
            state: row.get::<String, _>("state"),
            created_at: row.get::<String, _>("created_at"),
        }))
    }

    pub async fn events_for(&self, request_id: impl AsRef<str>) -> Result<Vec<RequestEventRecord>> {
        let rows = sqlx::query(
            "SELECT id, request_id, kind, payload_json, created_at
             FROM request_events
             WHERE request_id = ?
             ORDER BY id ASC",
        )
        .bind(request_id.as_ref())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(RequestEventRecord {
                    id: row.get::<i64, _>("id"),
                    request_id: row.get::<String, _>("request_id"),
                    kind: RequestEventKind::from_db(row.get::<String, _>("kind"))?,
                    payload_json: row.get::<String, _>("payload_json"),
                    created_at: row.get::<String, _>("created_at"),
                })
            })
            .collect()
    }

    pub async fn list(&self) -> Result<Vec<RequestListRecord>> {
        let rows = sqlx::query(
            "SELECT id, title, author, media_type, state, created_at
             FROM requests
             ORDER BY datetime(created_at) DESC, id DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(RequestListRecord {
                    id: row.get::<String, _>("id"),
                    title: row.get::<String, _>("title"),
                    author: row.get::<String, _>("author"),
                    media_type: match crate::domain::requests::MediaType::from_str(
                        row.get::<String, _>("media_type").as_str(),
                    ) {
                        Some(media_type) => media_type,
                        None => anyhow::bail!("unknown media type stored in requests"),
                    },
                    state: row.get::<String, _>("state"),
                    created_at: row.get::<String, _>("created_at"),
                })
            })
            .collect()
    }

    pub async fn list_pending_search_requests(&self) -> Result<Vec<RequestRecord>> {
        let rows = sqlx::query(
            "SELECT id
             FROM requests
             WHERE state IN ('requested', 'no_match')
             ORDER BY datetime(created_at) ASC, id ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut requests = Vec::with_capacity(rows.len());
        for row in rows {
            let request_id = row.get::<String, _>("id");
            let request = self.find_by_id(&request_id).await?.ok_or_else(|| {
                anyhow::anyhow!("request {request_id} disappeared during search scan")
            })?;
            requests.push(request);
        }

        Ok(requests)
    }

    pub async fn update_state(&self, request_id: &str, state: &str) -> Result<()> {
        sqlx::query("UPDATE requests SET state = ? WHERE id = ?")
            .bind(state)
            .bind(request_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn mark_search_completed(
        &self,
        request_id: &str,
        outcome: &str,
        candidates_seen: usize,
        qualified_candidates: usize,
        top_score: Option<f32>,
    ) -> Result<()> {
        let next_state = match outcome {
            "auto_acquire" => "queued",
            "review" => "review",
            "no_match" => "no_match",
            _ => "requested",
        };
        self.update_state(request_id, next_state).await?;
        self.append_event(
            request_id,
            RequestEventKind::SearchCompleted,
            json!({
                "outcome": outcome,
                "candidates_seen": candidates_seen,
                "qualified_candidates": qualified_candidates,
                "top_score": top_score,
            }),
        )
        .await
    }

    pub async fn mark_review_queued(
        &self,
        request_id: &str,
        queued_candidates: usize,
        top_score: Option<f32>,
    ) -> Result<()> {
        self.update_state(request_id, "review").await?;
        self.append_event(
            request_id,
            RequestEventKind::ReviewQueued,
            json!({
                "queued_candidates": queued_candidates,
                "top_score": top_score,
            }),
        )
        .await
    }

    pub async fn clear_review_queue(&self, request_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM review_queue WHERE request_id = ?")
            .bind(request_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn add_rejected_candidate(
        &self,
        request_id: &str,
        candidate_external_id: &str,
    ) -> Result<()> {
        sqlx::query(
            "INSERT OR IGNORE INTO rejected_candidates (request_id, candidate_external_id)
             VALUES (?, ?)",
        )
        .bind(request_id)
        .bind(candidate_external_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn rejected_candidate_ids(&self, request_id: &str) -> Result<Vec<String>> {
        let rows = sqlx::query(
            "SELECT candidate_external_id
             FROM rejected_candidates
             WHERE request_id = ?",
        )
        .bind(request_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| row.get::<String, _>("candidate_external_id"))
            .collect())
    }

    pub async fn enqueue_review_candidate(
        &self,
        request_id: &str,
        candidate: &ReleaseCandidate,
        scored: &ScoredCandidate,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO review_queue (
                request_id,
                candidate_external_id,
                candidate_source,
                candidate_title,
                candidate_protocol,
                candidate_size_bytes,
                candidate_indexer,
                candidate_download_url,
                score,
                explanation_json
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(request_id)
        .bind(&candidate.external_id)
        .bind(&candidate.source)
        .bind(&candidate.title)
        .bind(&candidate.protocol)
        .bind(candidate.size_bytes)
        .bind(&candidate.indexer)
        .bind(candidate.download_url.as_deref())
        .bind(scored.score)
        .bind(serde_json::to_string(&scored.explanation)?)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn review_queue_for(&self, request_id: &str) -> Result<Vec<ReviewQueueEntry>> {
        let rows = sqlx::query(
            "SELECT
                id,
                request_id,
                candidate_external_id,
                candidate_source,
                candidate_title,
                candidate_protocol,
                candidate_size_bytes,
                candidate_indexer,
                candidate_download_url,
                score,
                explanation_json,
                created_at
             FROM review_queue
             WHERE request_id = ?
             ORDER BY score DESC, id ASC",
        )
        .bind(request_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(ReviewQueueEntry {
                    id: row.get::<i64, _>("id"),
                    request_id: row.get::<String, _>("request_id"),
                    candidate: ReleaseCandidate {
                        external_id: row.get::<String, _>("candidate_external_id"),
                        source: row.get::<String, _>("candidate_source"),
                        title: row.get::<String, _>("candidate_title"),
                        protocol: row.get::<String, _>("candidate_protocol"),
                        size_bytes: row.get::<i64, _>("candidate_size_bytes"),
                        indexer: row.get::<String, _>("candidate_indexer"),
                        download_url: row.get::<Option<String>, _>("candidate_download_url"),
                    },
                    score: row.get::<f32, _>("score"),
                    explanation: serde_json::from_str(
                        row.get::<String, _>("explanation_json").as_str(),
                    )?,
                    created_at: row.get::<String, _>("created_at"),
                })
            })
            .collect()
    }

    pub async fn review_candidate_for_action(
        &self,
        request_id: &str,
        candidate_id: i64,
    ) -> Result<Option<ReviewQueueEntry>> {
        let row = sqlx::query(
            "SELECT
                id,
                request_id,
                candidate_external_id,
                candidate_source,
                candidate_title,
                candidate_protocol,
                candidate_size_bytes,
                candidate_indexer,
                candidate_download_url,
                score,
                explanation_json,
                created_at
             FROM review_queue
             WHERE request_id = ? AND id = ?",
        )
        .bind(request_id)
        .bind(candidate_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|row| {
            Ok(ReviewQueueEntry {
                id: row.get::<i64, _>("id"),
                request_id: row.get::<String, _>("request_id"),
                candidate: ReleaseCandidate {
                    external_id: row.get::<String, _>("candidate_external_id"),
                    source: row.get::<String, _>("candidate_source"),
                    title: row.get::<String, _>("candidate_title"),
                    protocol: row.get::<String, _>("candidate_protocol"),
                    size_bytes: row.get::<i64, _>("candidate_size_bytes"),
                    indexer: row.get::<String, _>("candidate_indexer"),
                    download_url: row.get::<Option<String>, _>("candidate_download_url"),
                },
                score: row.get::<f32, _>("score"),
                explanation: serde_json::from_str(
                    row.get::<String, _>("explanation_json").as_str(),
                )?,
                created_at: row.get::<String, _>("created_at"),
            })
        })
        .transpose()
    }

    pub async fn remove_review_candidate(&self, request_id: &str, candidate_id: i64) -> Result<()> {
        sqlx::query("DELETE FROM review_queue WHERE request_id = ? AND id = ?")
            .bind(request_id)
            .bind(candidate_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn mark_review_approved(
        &self,
        request_id: &str,
        candidate_id: i64,
        candidate_external_id: &str,
    ) -> Result<()> {
        self.append_event(
            request_id,
            RequestEventKind::ReviewApproved,
            json!({
                "candidate_id": candidate_id,
                "candidate_external_id": candidate_external_id,
            }),
        )
        .await
    }

    pub async fn mark_review_rejected(
        &self,
        request_id: &str,
        candidate_id: i64,
        candidate_external_id: &str,
    ) -> Result<()> {
        self.append_event(
            request_id,
            RequestEventKind::ReviewRejected,
            json!({
                "candidate_id": candidate_id,
                "candidate_external_id": candidate_external_id,
            }),
        )
        .await
    }

    pub async fn queue_download(
        &self,
        request_id: &str,
        candidate: &ReleaseCandidate,
        category: &str,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO downloads (
                request_id,
                candidate_external_id,
                candidate_source,
                candidate_title,
                candidate_protocol,
                candidate_size_bytes,
                candidate_indexer,
                category,
                status
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(request_id)
        .bind(&candidate.external_id)
        .bind(&candidate.source)
        .bind(&candidate.title)
        .bind(&candidate.protocol)
        .bind(candidate.size_bytes)
        .bind(&candidate.indexer)
        .bind(category)
        .bind("queued")
        .execute(&self.pool)
        .await?;

        sqlx::query("UPDATE requests SET state = ? WHERE id = ?")
            .bind("queued")
            .bind(request_id)
            .execute(&self.pool)
            .await?;

        self.append_event(
            request_id,
            RequestEventKind::DownloadQueued,
            json!({
                "candidate_external_id": &candidate.external_id,
                "candidate_source": &candidate.source,
                "candidate_title": &candidate.title,
                "candidate_protocol": &candidate.protocol,
                "candidate_size_bytes": candidate.size_bytes,
                "candidate_indexer": &candidate.indexer,
                "download_url": &candidate.download_url,
                "category": category,
            }),
        )
        .await
    }

    pub async fn queued_downloads(&self) -> Result<Vec<QueuedDownloadRecord>> {
        let rows = sqlx::query(
            "SELECT request_id, category
             FROM downloads
             WHERE status = ?
             ORDER BY id ASC",
        )
        .bind("queued")
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| QueuedDownloadRecord {
                request_id: row.get::<String, _>("request_id"),
                category: row.get::<String, _>("category"),
            })
            .collect())
    }

    pub async fn complete_download(&self, request_id: &str, files: &[String]) -> Result<()> {
        sqlx::query(
            "UPDATE downloads
             SET status = ?, payload_json = ?, completed_at = CURRENT_TIMESTAMP
             WHERE request_id = ? AND status = ?",
        )
        .bind("completed")
        .bind(serde_json::to_string(files)?)
        .bind(request_id)
        .bind("queued")
        .execute(&self.pool)
        .await?;

        sqlx::query("UPDATE requests SET state = ? WHERE id = ?")
            .bind("downloaded")
            .bind(request_id)
            .execute(&self.pool)
            .await?;

        self.append_event(
            request_id,
            RequestEventKind::DownloadCompleted,
            json!({ "files": files }),
        )
        .await
    }

    pub async fn mark_import_succeeded(
        &self,
        request_id: &str,
        destination_path: &std::path::Path,
    ) -> Result<()> {
        sqlx::query("UPDATE requests SET state = ?, imported_path = ? WHERE id = ?")
            .bind("imported")
            .bind(destination_path.display().to_string())
            .bind(request_id)
            .execute(&self.pool)
            .await?;

        sqlx::query("UPDATE downloads SET status = ? WHERE request_id = ? AND status = ?")
            .bind("imported")
            .bind(request_id)
            .bind("completed")
            .execute(&self.pool)
            .await?;

        self.append_event(
            request_id,
            RequestEventKind::ImportSucceeded,
            json!({ "destination_path": destination_path }),
        )
        .await
    }

    pub async fn mark_sync_succeeded(
        &self,
        request_id: &str,
        target_path: &std::path::Path,
    ) -> Result<()> {
        sqlx::query("UPDATE requests SET state = ? WHERE id = ?")
            .bind("synced")
            .bind(request_id)
            .execute(&self.pool)
            .await?;

        self.append_event(
            request_id,
            RequestEventKind::SyncSucceeded,
            json!({ "target_path": target_path }),
        )
        .await
    }

    pub async fn find_request_by_imported_path(
        &self,
        path: &str,
        media_type: &str,
    ) -> Result<Option<RequestRecord>> {
        let row = sqlx::query(
            "SELECT id, external_work_id, title, author, media_type, preferred_language, state, created_at
             FROM requests
             WHERE imported_path = ? AND media_type = ?",
        )
        .bind(path)
        .bind(media_type)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let request_id: String = row.get("id");
        let payload_json = sqlx::query(
            "SELECT payload_json FROM request_events WHERE request_id = ? ORDER BY id ASC LIMIT 1",
        )
        .bind(&request_id)
        .fetch_one(&self.pool)
        .await?
        .get::<String, _>("payload_json");

        let payload: serde_json::Value = serde_json::from_str(&payload_json)?;
        let manifestation = manifestation_from_payload(&payload);

        Ok(Some(RequestRecord {
            id: request_id,
            external_work_id: row.get::<String, _>("external_work_id"),
            title: row.get::<String, _>("title"),
            author: row.get::<String, _>("author"),
            media_type: match crate::domain::requests::MediaType::from_str(
                row.get::<String, _>("media_type").as_str(),
            ) {
                Some(media_type) => media_type,
                None => anyhow::bail!("unknown media type stored in requests"),
            },
            preferred_language: row.get::<Option<String>, _>("preferred_language"),
            manifestation,
            state: row.get::<String, _>("state"),
            created_at: row.get::<String, _>("created_at"),
        }))
    }

    pub async fn find_request_by_title_author(
        &self,
        title: &str,
        author: &str,
        media_type: &str,
    ) -> Result<Option<RequestRecord>> {
        let row = sqlx::query(
            "SELECT id, external_work_id, title, author, media_type, preferred_language, state, created_at
             FROM requests
             WHERE title = ? AND author = ? AND media_type = ?",
        )
        .bind(title)
        .bind(author)
        .bind(media_type)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let request_id: String = row.get("id");
        let payload_json = sqlx::query(
            "SELECT payload_json FROM request_events WHERE request_id = ? ORDER BY id ASC LIMIT 1",
        )
        .bind(&request_id)
        .fetch_one(&self.pool)
        .await?
        .get::<String, _>("payload_json");

        let payload: serde_json::Value = serde_json::from_str(&payload_json)?;
        let manifestation = manifestation_from_payload(&payload);

        Ok(Some(RequestRecord {
            id: request_id,
            external_work_id: row.get::<String, _>("external_work_id"),
            title: row.get::<String, _>("title"),
            author: row.get::<String, _>("author"),
            media_type: match crate::domain::requests::MediaType::from_str(
                row.get::<String, _>("media_type").as_str(),
            ) {
                Some(media_type) => media_type,
                None => anyhow::bail!("unknown media type stored in requests"),
            },
            preferred_language: row.get::<Option<String>, _>("preferred_language"),
            manifestation,
            state: row.get::<String, _>("state"),
            created_at: row.get::<String, _>("created_at"),
        }))
    }

    pub async fn create_library_discovered_request(
        &self,
        item: &crate::domain::library::ScannedItem,
    ) -> Result<RequestRecord> {
        let id = uuid::Uuid::new_v4().to_string();
        let media_type = item.media_type.as_str();

        sqlx::query(
            "INSERT INTO requests (id, external_work_id, title, author, media_type, preferred_language, state, imported_path, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)",
        )
        .bind(&id)
        .bind("")
        .bind(&item.title)
        .bind(&item.author)
        .bind(media_type)
        .bind::<Option<&str>>(None)
        .bind("imported")
        .bind(&item.imported_path)
        .execute(&self.pool)
        .await?;

        let payload_json = json!({
            "request_id": id,
            "external_work_id": "",
            "work": {
                "external_id": "",
                "title": &item.title,
                "author": &item.author,
            },
            "title": &item.title,
            "author": &item.author,
            "media_type": media_type,
            "preferred_language": null,
            "manifestation": {
                "edition_title": null,
                "preferred_narrator": null,
                "preferred_publisher": null,
                "graphic_audio": false,
            },
            "imported_path": &item.imported_path,
        })
        .to_string();

        sqlx::query(
            "INSERT INTO request_events (request_id, kind, payload_json, created_at)
             VALUES (?, ?, ?, CURRENT_TIMESTAMP)",
        )
        .bind(&id)
        .bind(RequestEventKind::LibraryDiscovered.as_str())
        .bind(payload_json)
        .execute(&self.pool)
        .await?;

        self.find_by_id(&id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("freshly created library request disappeared"))
    }

    pub async fn create_scan_job(&self) -> Result<i64> {
        let row = sqlx::query(
            "INSERT INTO library_scan_jobs (started_at)
             VALUES (CURRENT_TIMESTAMP)
             RETURNING id",
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get::<i64, _>("id"))
    }

    pub async fn complete_scan_job(
        &self,
        job_id: i64,
        ebooks_found: i64,
        audiobooks_found: i64,
        duplicates_skipped: i64,
        error_message: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE library_scan_jobs
             SET completed_at = CURRENT_TIMESTAMP,
                 ebooks_found = ?,
                 audiobooks_found = ?,
                 duplicates_skipped = ?,
                 error_message = ?
             WHERE id = ?",
        )
        .bind(ebooks_found)
        .bind(audiobooks_found)
        .bind(duplicates_skipped)
        .bind(error_message)
        .bind(job_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn latest_scan_job(
        &self,
    ) -> Result<Option<crate::domain::library::LibraryScanJobRecord>> {
        let row = sqlx::query(
            "SELECT id, started_at, completed_at, ebooks_found, audiobooks_found, duplicates_skipped, error_message
             FROM library_scan_jobs
             ORDER BY id DESC
             LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| crate::domain::library::LibraryScanJobRecord {
            id: row.get::<i64, _>("id"),
            started_at: row.get::<String, _>("started_at"),
            completed_at: row.get::<Option<String>, _>("completed_at"),
            ebooks_found: row.get::<i64, _>("ebooks_found"),
            audiobooks_found: row.get::<i64, _>("audiobooks_found"),
            duplicates_skipped: row.get::<i64, _>("duplicates_skipped"),
            error_message: row.get::<Option<String>, _>("error_message"),
        }))
    }

    async fn append_event(
        &self,
        request_id: &str,
        kind: RequestEventKind,
        payload: serde_json::Value,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO request_events (request_id, kind, payload_json, created_at)
             VALUES (?, ?, ?, CURRENT_TIMESTAMP)",
        )
        .bind(request_id)
        .bind(kind.as_str())
        .bind(payload.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

impl SqliteSettingsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn ensure_seeded(&self, config: &AppConfig) -> Result<PersistedRuntimeSettings> {
        if let Some(settings) = self.get_persisted_runtime_settings().await? {
            return Ok(settings);
        }

        let settings = PersistedRuntimeSettings::from_config(config);
        self.write_runtime_settings(&settings).await?;
        Ok(settings)
    }

    pub async fn get_runtime_settings(&self) -> Result<RuntimeSettingsRecord> {
        let settings = self
            .get_persisted_runtime_settings()
            .await?
            .context("runtime settings row missing")?;
        Ok(settings.to_record())
    }

    pub async fn get_persisted_runtime_settings(&self) -> Result<Option<PersistedRuntimeSettings>> {
        let row = sqlx::query(
            "SELECT settings_json
             FROM runtime_settings
             WHERE singleton_key = 1",
        )
        .fetch_optional(&self.pool)
        .await?;

        row.map(|row| {
            serde_json::from_str::<PersistedRuntimeSettings>(&row.get::<String, _>("settings_json"))
                .map_err(anyhow::Error::from)
        })
        .transpose()
    }

    pub async fn update_runtime_settings(
        &self,
        update: RuntimeSettingsUpdate,
    ) -> Result<RuntimeSettingsRecord> {
        let mut settings = self
            .get_persisted_runtime_settings()
            .await?
            .context("runtime settings row missing")?;
        settings.apply_update(update);
        settings.validate()?;
        self.write_runtime_settings(&settings).await?;
        Ok(settings.to_record())
    }

    pub async fn list_synced_indexers(&self) -> Result<Vec<SyncedIndexerRecord>> {
        let rows = sqlx::query(
            "SELECT
                id,
                prowlarr_indexer_id,
                name,
                enabled,
                implementation,
                protocol,
                base_url,
                categories_json,
                last_synced_at
             FROM synced_indexers
             ORDER BY name ASC, id ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_synced_indexer_record).collect()
    }

    pub async fn list_search_indexers(&self) -> Result<Vec<SearchIndexerRecord>> {
        let rows = sqlx::query(
            "SELECT
                id,
                implementation,
                protocol,
                base_url,
                api_path,
                api_key,
                enabled
             FROM synced_indexers
             WHERE enabled = 1 AND base_url IS NOT NULL
             ORDER BY id ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| SearchIndexerRecord {
                id: row.get::<i64, _>("id"),
                implementation: row.get::<String, _>("implementation"),
                protocol: row.get::<Option<String>, _>("protocol"),
                base_url: row.get::<String, _>("base_url"),
                api_path: row.get::<Option<String>, _>("api_path"),
                api_key: row.get::<Option<String>, _>("api_key"),
                enabled: row.get::<bool, _>("enabled"),
            })
            .collect())
    }

    pub async fn get_synced_indexer_resource(&self, id: i64) -> Result<Option<Value>> {
        let row = sqlx::query(
            "SELECT id, raw_payload_json, api_key
             FROM synced_indexers
             WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_indexer_resource).transpose()
    }

    pub async fn list_synced_indexer_resources(&self) -> Result<Vec<Value>> {
        let rows = sqlx::query(
            "SELECT id, raw_payload_json, api_key
             FROM synced_indexers
             ORDER BY id ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_indexer_resource).collect()
    }

    pub async fn create_synced_indexer_resource(&self, resource: &Value) -> Result<Value> {
        let indexer = ParsedIndexerPayload::parse(resource)?;

        let row = sqlx::query(
            "INSERT INTO synced_indexers (
                prowlarr_indexer_id,
                name,
                enabled,
                implementation,
                implementation_name,
                config_contract,
                protocol,
                priority,
                base_url,
                api_path,
                categories_json,
                api_key,
                raw_payload_json,
                updated_at,
                last_synced_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            RETURNING id, raw_payload_json, api_key",
        )
        .bind(indexer.prowlarr_indexer_id)
        .bind(&indexer.name)
        .bind(indexer.enabled)
        .bind(&indexer.implementation)
        .bind(&indexer.implementation_name)
        .bind(&indexer.config_contract)
        .bind(indexer.protocol.as_deref())
        .bind(indexer.priority)
        .bind(indexer.base_url.as_deref())
        .bind(indexer.api_path.as_deref())
        .bind(serde_json::to_string(&indexer.categories)?)
        .bind(indexer.api_key.as_deref())
        .bind(indexer.raw_payload.to_string())
        .fetch_one(&self.pool)
        .await?;

        row_to_indexer_resource(row)
    }

    pub async fn update_synced_indexer_resource(
        &self,
        id: i64,
        resource: &Value,
    ) -> Result<Option<Value>> {
        let indexer = ParsedIndexerPayload::parse(resource)?;

        let row = sqlx::query(
            "UPDATE synced_indexers
             SET prowlarr_indexer_id = ?,
                 name = ?,
                 enabled = ?,
                 implementation = ?,
                 implementation_name = ?,
                 config_contract = ?,
                 protocol = ?,
                 priority = ?,
                 base_url = ?,
                 api_path = ?,
                 categories_json = ?,
                 api_key = ?,
                 raw_payload_json = ?,
                 updated_at = CURRENT_TIMESTAMP,
                 last_synced_at = CURRENT_TIMESTAMP
             WHERE id = ?
             RETURNING id, raw_payload_json, api_key",
        )
        .bind(indexer.prowlarr_indexer_id)
        .bind(&indexer.name)
        .bind(indexer.enabled)
        .bind(&indexer.implementation)
        .bind(&indexer.implementation_name)
        .bind(&indexer.config_contract)
        .bind(indexer.protocol.as_deref())
        .bind(indexer.priority)
        .bind(indexer.base_url.as_deref())
        .bind(indexer.api_path.as_deref())
        .bind(serde_json::to_string(&indexer.categories)?)
        .bind(indexer.api_key.as_deref())
        .bind(indexer.raw_payload.to_string())
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_indexer_resource).transpose()
    }

    pub async fn delete_synced_indexer(&self, id: i64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM synced_indexers WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn write_runtime_settings(&self, settings: &PersistedRuntimeSettings) -> Result<()> {
        sqlx::query(
            "INSERT INTO runtime_settings (singleton_key, settings_json)
             VALUES (1, ?)
             ON CONFLICT(singleton_key) DO UPDATE SET
                 settings_json = excluded.settings_json,
                 updated_at = CURRENT_TIMESTAMP",
        )
        .bind(serde_json::to_string(settings)?)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

fn manifestation_from_payload(
    payload: &serde_json::Value,
) -> crate::domain::requests::ManifestationPreference {
    let manifestation = &payload["manifestation"];
    crate::domain::requests::ManifestationPreference {
        edition_title: manifestation
            .get("edition_title")
            .and_then(|value| value.as_str())
            .map(str::to_string),
        preferred_narrator: manifestation
            .get("preferred_narrator")
            .and_then(|value| value.as_str())
            .map(str::to_string),
        preferred_publisher: manifestation
            .get("preferred_publisher")
            .and_then(|value| value.as_str())
            .map(str::to_string),
        graphic_audio: manifestation
            .get("graphic_audio")
            .and_then(|value| value.as_bool())
            .unwrap_or(false),
    }
}

fn row_to_synced_indexer_record(row: sqlx::sqlite::SqliteRow) -> Result<SyncedIndexerRecord> {
    Ok(SyncedIndexerRecord {
        id: row.get::<i64, _>("id"),
        prowlarr_indexer_id: row.get::<Option<i64>, _>("prowlarr_indexer_id"),
        name: row.get::<String, _>("name"),
        enabled: row.get::<bool, _>("enabled"),
        implementation: row.get::<String, _>("implementation"),
        protocol: row.get::<Option<String>, _>("protocol"),
        base_url: row.get::<Option<String>, _>("base_url"),
        categories: serde_json::from_str(row.get::<String, _>("categories_json").as_str())?,
        last_synced_at: row.get::<String, _>("last_synced_at"),
    })
}

fn row_to_indexer_resource(row: sqlx::sqlite::SqliteRow) -> Result<Value> {
    let id = row.get::<i64, _>("id");
    let api_key = row.get::<Option<String>, _>("api_key");
    let mut payload: Value = serde_json::from_str(&row.get::<String, _>("raw_payload_json"))?;

    if let Some(object) = payload.as_object_mut() {
        object.insert("id".to_string(), json!(id));
    }

    mask_indexer_secret_fields(&mut payload, api_key.as_deref());
    Ok(payload)
}

fn mask_indexer_secret_fields(payload: &mut Value, api_key: Option<&str>) {
    let Some(fields) = payload.get_mut("fields").and_then(Value::as_array_mut) else {
        return;
    };

    for field in fields {
        if field.get("name").and_then(Value::as_str) == Some("apiKey")
            && let Some(object) = field.as_object_mut()
        {
            let value = if api_key.unwrap_or_default().is_empty() {
                Value::Null
            } else {
                Value::String("********".to_string())
            };
            object.insert("value".to_string(), value);
        }
    }
}

struct ParsedIndexerPayload {
    prowlarr_indexer_id: Option<i64>,
    name: String,
    enabled: bool,
    implementation: String,
    implementation_name: String,
    config_contract: String,
    protocol: Option<String>,
    priority: i64,
    base_url: Option<String>,
    api_path: Option<String>,
    api_key: Option<String>,
    categories: Vec<i64>,
    raw_payload: Value,
}

impl ParsedIndexerPayload {
    fn parse(payload: &Value) -> Result<Self> {
        let object = payload
            .as_object()
            .context("indexer payload must be a JSON object")?;
        let fields = object
            .get("fields")
            .and_then(Value::as_array)
            .context("indexer payload must include fields")?;

        let base_url = field_as_string(fields, "baseUrl");
        let api_path = field_as_string(fields, "apiPath");
        let api_key = field_as_string(fields, "apiKey");
        let categories = field_as_i64_vec(fields, "categories");
        let implementation = object
            .get("implementation")
            .and_then(Value::as_str)
            .unwrap_or("Torznab")
            .to_string();
        let implementation_name = object
            .get("implementationName")
            .and_then(Value::as_str)
            .unwrap_or(implementation.as_str())
            .to_string();
        let config_contract = object
            .get("configContract")
            .and_then(Value::as_str)
            .unwrap_or_else(|| match implementation.as_str() {
                "Newznab" => "NewznabSettings",
                _ => "TorznabSettings",
            })
            .to_string();

        Ok(Self {
            prowlarr_indexer_id: base_url
                .as_deref()
                .and_then(parse_prowlarr_indexer_id_from_base_url),
            name: object
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("Unnamed Indexer")
                .to_string(),
            enabled: object
                .get("enableRss")
                .and_then(Value::as_bool)
                .unwrap_or(false)
                || object
                    .get("enableAutomaticSearch")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                || object
                    .get("enableInteractiveSearch")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            implementation,
            implementation_name,
            config_contract,
            protocol: object
                .get("protocol")
                .and_then(Value::as_str)
                .map(str::to_string),
            priority: object.get("priority").and_then(Value::as_i64).unwrap_or(25),
            base_url,
            api_path,
            api_key,
            categories,
            raw_payload: payload.clone(),
        })
    }
}

fn field_as_string(fields: &[Value], name: &str) -> Option<String> {
    fields
        .iter()
        .find(|field| field.get("name").and_then(Value::as_str) == Some(name))
        .and_then(|field| field.get("value"))
        .and_then(|value| match value {
            Value::Null => None,
            Value::String(text) => Some(text.clone()),
            other => Some(other.to_string()),
        })
}

fn field_as_i64_vec(fields: &[Value], name: &str) -> Vec<i64> {
    fields
        .iter()
        .find(|field| field.get("name").and_then(Value::as_str) == Some(name))
        .and_then(|field| field.get("value"))
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(Value::as_i64).collect())
        .unwrap_or_default()
}

fn parse_prowlarr_indexer_id_from_base_url(base_url: &str) -> Option<i64> {
    let url = reqwest::Url::parse(base_url).ok()?;
    url.path_segments()?
        .filter(|segment| !segment.is_empty())
        .find_map(|segment| segment.parse::<i64>().ok())
}
