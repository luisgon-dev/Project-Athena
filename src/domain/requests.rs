use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::events::RequestEventRecord;
use super::search::ReviewQueueEntry;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum MediaType {
    Ebook,
    Audiobook,
}

impl MediaType {
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "ebook" => Some(Self::Ebook),
            "audiobook" => Some(Self::Audiobook),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ebook => "ebook",
            Self::Audiobook => "audiobook",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ManifestationPreference {
    pub edition_title: Option<String>,
    pub preferred_narrator: Option<String>,
    pub preferred_publisher: Option<String>,
    pub graphic_audio: bool,
}

impl ManifestationPreference {
    pub fn new(
        edition_title: Option<String>,
        preferred_narrator: Option<String>,
        preferred_publisher: Option<String>,
        graphic_audio: bool,
    ) -> Self {
        Self {
            edition_title,
            preferred_narrator,
            preferred_publisher,
            graphic_audio,
        }
    }
}

impl Default for MediaType {
    fn default() -> Self {
        Self::Ebook
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CreateRequest {
    pub external_work_id: String,
    pub title: String,
    pub author: String,
    pub media_type: MediaType,
    pub preferred_language: Option<String>,
    pub manifestation: ManifestationPreference,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RequestRecord {
    pub id: String,
    pub external_work_id: String,
    pub title: String,
    pub author: String,
    pub media_type: MediaType,
    pub preferred_language: Option<String>,
    pub manifestation: ManifestationPreference,
    pub submission_id: Option<String>,
    pub requested_by_user_id: Option<String>,
    pub requires_admin_approval: bool,
    pub state: String,
    pub created_at: String,
}

impl RequestRecord {
    pub fn for_tests(
        title: impl Into<String>,
        author: impl Into<String>,
        media_type: MediaType,
    ) -> Self {
        Self {
            id: "request-1".to_string(),
            external_work_id: "work-1".to_string(),
            title: title.into(),
            author: author.into(),
            media_type,
            preferred_language: None,
            manifestation: ManifestationPreference::default(),
            submission_id: None,
            requested_by_user_id: None,
            requires_admin_approval: false,
            state: "requested".to_string(),
            created_at: "1970-01-01T00:00:00Z".to_string(),
        }
    }

    pub fn with_preferences(mut self, manifestation: ManifestationPreference) -> Self {
        self.manifestation = manifestation;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RequestListRecord {
    pub id: String,
    pub title: String,
    pub author: String,
    pub media_type: MediaType,
    pub state: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CreateRequestSelection {
    pub selected_work_id: Option<String>,
    #[serde(default)]
    pub media_types: Vec<MediaType>,
    pub preferred_language: Option<String>,
    #[serde(default)]
    pub manifestation: ManifestationPreference,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RequestDetailRecord {
    pub request: RequestRecord,
    pub events: Vec<RequestEventRecord>,
    pub review_queue: Vec<ReviewQueueEntry>,
}
