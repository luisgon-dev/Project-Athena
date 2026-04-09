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
        let id = Uuid::new_v4().to_string();
        let media_type = request.media_type.as_str();
        let preferred_language = request.preferred_language.clone();
        let manifestation = request.manifestation.clone();
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            "INSERT INTO requests (id, title, author, media_type, preferred_language, state, created_at)
             VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)",
        )
        .bind(&id)
        .bind(&request.title)
        .bind(&request.author)
        .bind(media_type)
        .bind(preferred_language.as_deref())
        .bind("requested")
        .execute(&mut *tx)
        .await?;

        let payload_json = json!({
            "request_id": id,
            "title": request.title,
            "author": request.author,
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
            "SELECT id, title, author, media_type, preferred_language, state, created_at
             FROM requests
             WHERE id = ?",
        )
        .bind(&id)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(RequestRecord {
            id: row.get::<String, _>("id"),
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
        })
    }

    pub async fn find_by_id(&self, request_id: impl AsRef<str>) -> Result<Option<RequestRecord>> {
        let row = sqlx::query(
            "SELECT id, title, author, media_type, preferred_language, state, created_at
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
