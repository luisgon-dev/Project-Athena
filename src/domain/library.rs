use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::requests::MediaType;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScannedItem {
    pub author: String,
    pub title: String,
    pub media_type: MediaType,
    pub imported_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LibraryScanJobRecord {
    pub id: i64,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub ebooks_found: i64,
    pub audiobooks_found: i64,
    pub duplicates_skipped: i64,
    pub error_message: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LibraryScanResponse {
    pub job_id: i64,
}
