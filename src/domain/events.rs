use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum RequestEventKind {
    Created,
    DownloadQueued,
    DownloadCompleted,
    ImportSucceeded,
    SyncSucceeded,
}

impl RequestEventKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "request.created",
            Self::DownloadQueued => "download.queued",
            Self::DownloadCompleted => "download.completed",
            Self::ImportSucceeded => "import.succeeded",
            Self::SyncSucceeded => "sync.succeeded",
        }
    }

    pub fn from_db(value: String) -> Result<Self> {
        match value.as_str() {
            "request.created" => Ok(Self::Created),
            "download.queued" => Ok(Self::DownloadQueued),
            "download.completed" => Ok(Self::DownloadCompleted),
            "import.succeeded" => Ok(Self::ImportSucceeded),
            "sync.succeeded" => Ok(Self::SyncSucceeded),
            other => Err(anyhow!("unknown request event kind: {other}")),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RequestEventRecord {
    pub id: i64,
    pub request_id: String,
    pub kind: RequestEventKind,
    pub payload_json: String,
    pub created_at: String,
}
