use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ReleaseCandidate {
    pub external_id: String,
    pub source: String,
    pub title: String,
    pub protocol: String,
    pub size_bytes: i64,
    pub indexer: String,
    pub download_url: Option<String>,
}

impl ReleaseCandidate {
    pub fn for_tests(title: impl Into<String>) -> Self {
        Self {
            external_id: "candidate-1".to_string(),
            source: "test".to_string(),
            title: title.into(),
            protocol: "torrent".to_string(),
            size_bytes: 0,
            indexer: "test-indexer".to_string(),
            download_url: Some(
                "magnet:?xt=urn:btih:0123456789abcdef0123456789abcdef01234567&dn=test".to_string(),
            ),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ScoredCandidate {
    pub score: f32,
    pub explanation: Vec<String>,
    pub auto_acquire: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ReviewQueueEntry {
    pub id: i64,
    pub request_id: String,
    pub candidate: ReleaseCandidate,
    pub score: f32,
    pub explanation: Vec<String>,
    pub created_at: String,
}
