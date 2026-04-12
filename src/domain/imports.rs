#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImportMediaType {
    Ebook,
    Audiobook,
    Mixed,
    Invalid,
}

impl ImportMediaType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ebook => "ebook",
            Self::Audiobook => "audiobook",
            Self::Mixed => "mixed",
            Self::Invalid => "invalid",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Classification {
    pub media_type: ImportMediaType,
    pub reason: Option<String>,
}

impl Classification {
    pub fn ebook() -> Self {
        Self {
            media_type: ImportMediaType::Ebook,
            reason: None,
        }
    }

    pub fn audiobook() -> Self {
        Self {
            media_type: ImportMediaType::Audiobook,
            reason: None,
        }
    }

    pub fn mixed() -> Self {
        Self {
            media_type: ImportMediaType::Mixed,
            reason: None,
        }
    }

    pub fn invalid(reason: impl Into<String>) -> Self {
        Self {
            media_type: ImportMediaType::Invalid,
            reason: Some(reason.into()),
        }
    }
}
