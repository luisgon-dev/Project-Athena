use anyhow::Result;
use serde_json::json;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::{
    events::{RequestEventKind, RequestEventRecord},
    requests::{CreateRequest, RequestListRecord, RequestRecord},
    search::{ReleaseCandidate, ReviewQueueEntry, ScoredCandidate},
};

pub struct SqliteRequestRepository {
    pool: SqlitePool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueuedDownloadRecord {
    pub request_id: String,
    pub category: String,
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
                score,
                explanation_json
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(request_id)
        .bind(&candidate.external_id)
        .bind(&candidate.source)
        .bind(&candidate.title)
        .bind(&candidate.protocol)
        .bind(candidate.size_bytes)
        .bind(&candidate.indexer)
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
                        download_url: None,
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
        sqlx::query("UPDATE requests SET state = ? WHERE id = ?")
            .bind("imported")
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
