#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
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

pub struct CreateRequest {
    pub external_work_id: String,
    pub title: String,
    pub author: String,
    pub media_type: MediaType,
    pub preferred_language: Option<String>,
    pub manifestation: ManifestationPreference,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestRecord {
    pub id: String,
    pub external_work_id: String,
    pub title: String,
    pub author: String,
    pub media_type: MediaType,
    pub preferred_language: Option<String>,
    pub manifestation: ManifestationPreference,
    pub state: String,
    pub created_at: String,
}
