use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum RequestEventKind {
    Created,
    SearchCompleted,
    ReviewQueued,
    ReviewApproved,
    ReviewRejected,
    DownloadQueued,
    DownloadCompleted,
    ImportSucceeded,
    SyncSucceeded,
    LibraryDiscovered,
}

impl RequestEventKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "request.created",
            Self::SearchCompleted => "search.completed",
            Self::ReviewQueued => "review.queued",
            Self::ReviewApproved => "review.approved",
            Self::ReviewRejected => "review.rejected",
            Self::DownloadQueued => "download.queued",
            Self::DownloadCompleted => "download.completed",
            Self::ImportSucceeded => "import.succeeded",
            Self::SyncSucceeded => "sync.succeeded",
            Self::LibraryDiscovered => "library.discovered",
        }
    }

    pub fn from_db(value: String) -> Result<Self> {
        match value.as_str() {
            "request.created" => Ok(Self::Created),
            "search.completed" => Ok(Self::SearchCompleted),
            "review.queued" => Ok(Self::ReviewQueued),
            "review.approved" => Ok(Self::ReviewApproved),
            "review.rejected" => Ok(Self::ReviewRejected),
            "download.queued" => Ok(Self::DownloadQueued),
            "download.completed" => Ok(Self::DownloadCompleted),
            "import.succeeded" => Ok(Self::ImportSucceeded),
            "sync.succeeded" => Ok(Self::SyncSucceeded),
            "library.discovered" => Ok(Self::LibraryDiscovered),
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
