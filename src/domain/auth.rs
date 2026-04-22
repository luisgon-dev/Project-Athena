use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum UserRole {
    Admin,
    Trusted,
    #[default]
    Requester,
}

impl UserRole {
    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "admin" => Some(Self::Admin),
            "trusted" => Some(Self::Trusted),
            "requester" => Some(Self::Requester),
            _ => None,
        }
    }

    pub fn as_db(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Trusted => "trusted",
            Self::Requester => "requester",
        }
    }

    pub fn can_auto_acquire(&self) -> bool {
        matches!(self, Self::Admin | Self::Trusted)
    }

    pub fn is_admin(&self) -> bool {
        matches!(self, Self::Admin)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AuthUserRecord {
    pub id: String,
    pub username: String,
    pub role: UserRole,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UserRecord {
    pub id: String,
    pub username: String,
    pub role: UserRole,
    pub disabled: bool,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SessionRecord {
    pub id: String,
    pub user_id: String,
    pub expires_at: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetupRequest {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub role: UserRole,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UpdateUserRequest {
    pub role: Option<UserRole>,
    pub disabled: Option<bool>,
    pub password: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AuthBootstrapStatus {
    pub setup_required: bool,
    pub authenticated_user: Option<AuthUserRecord>,
}
