use anyhow::{Result, anyhow};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RequestEventKind {
    Created,
}

impl RequestEventKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "request.created",
        }
    }

    pub fn from_db(value: String) -> Result<Self> {
        match value.as_str() {
            "request.created" => Ok(Self::Created),
            other => Err(anyhow!("unknown request event kind: {other}")),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestEventRecord {
    pub id: i64,
    pub request_id: String,
    pub kind: RequestEventKind,
    pub payload_json: String,
    pub created_at: String,
}
