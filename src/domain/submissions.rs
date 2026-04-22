use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::domain::{
    catalog::WorkRecord,
    requests::{ManifestationPreference, MediaType, RequestListRecord},
};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum SubmissionIntakeMode {
    #[default]
    Metadata,
    Manual,
}

impl SubmissionIntakeMode {
    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "metadata" => Some(Self::Metadata),
            "manual" => Some(Self::Manual),
            _ => None,
        }
    }

    pub fn as_db(&self) -> &'static str {
        match self {
            Self::Metadata => "metadata",
            Self::Manual => "manual",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum SubmissionStatus {
    #[default]
    Submitted,
    PendingResolution,
    Resolved,
    Approved,
    Rejected,
}

impl SubmissionStatus {
    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "submitted" => Some(Self::Submitted),
            "pending_resolution" => Some(Self::PendingResolution),
            "resolved" => Some(Self::Resolved),
            "approved" => Some(Self::Approved),
            "rejected" => Some(Self::Rejected),
            _ => None,
        }
    }

    pub fn as_db(&self) -> &'static str {
        match self {
            Self::Submitted => "submitted",
            Self::PendingResolution => "pending_resolution",
            Self::Resolved => "resolved",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum DuplicateSource {
    #[default]
    AthenaRequested,
    AthenaImported,
    Audiobookshelf,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DuplicateHint {
    pub source: DuplicateSource,
    pub label: String,
    pub detail: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SubmissionSearchCandidate {
    pub work: WorkRecord,
    pub duplicate_hints: Vec<DuplicateHint>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SubmissionSearchResult {
    pub works: Vec<SubmissionSearchCandidate>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RequestSubmissionRecord {
    pub id: String,
    pub requested_by_user_id: String,
    pub requested_by_username: String,
    pub intake_mode: SubmissionIntakeMode,
    pub title: String,
    pub author: String,
    pub external_work_id: Option<String>,
    pub notes: Option<String>,
    pub media_types: Vec<MediaType>,
    pub preferred_language: Option<String>,
    pub manifestation: ManifestationPreference,
    pub status: SubmissionStatus,
    pub requires_admin_approval: bool,
    pub allow_duplicate: bool,
    pub linked_requests: Vec<RequestListRecord>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SubmissionEventRecord {
    pub id: i64,
    pub submission_id: String,
    pub kind: String,
    pub payload_json: String,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RequestSubmissionDetailRecord {
    pub submission: RequestSubmissionRecord,
    pub events: Vec<SubmissionEventRecord>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CreateSubmissionRequest {
    pub intake_mode: Option<SubmissionIntakeMode>,
    pub selected_work_id: Option<String>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub notes: Option<String>,
    #[serde(default)]
    pub media_types: Vec<MediaType>,
    pub preferred_language: Option<String>,
    #[serde(default)]
    pub manifestation: ManifestationPreference,
    #[serde(default)]
    pub allow_duplicate: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ResolveManualSubmissionRequest {
    pub selected_work_id: String,
}
