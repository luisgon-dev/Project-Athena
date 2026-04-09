#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MediaType {
    Ebook,
    Audiobook,
}

impl MediaType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ebook => "ebook",
            Self::Audiobook => "audiobook",
        }
    }
}

pub struct CreateRequest {
    pub title: String,
    pub author: String,
    pub media_type: MediaType,
    pub preferred_language: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestRecord {
    pub id: String,
    pub title: String,
    pub author: String,
    pub media_type: MediaType,
    pub preferred_language: Option<String>,
    pub state: String,
    pub created_at: String,
}
