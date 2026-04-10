use anyhow::Result;
use serde_json::json;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::{
    events::{RequestEventKind, RequestEventRecord},
    requests::{CreateRequest, RequestRecord},
};

pub struct SqliteRequestRepository {
    pool: SqlitePool,
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
