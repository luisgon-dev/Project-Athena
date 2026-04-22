use axum::http::HeaderMap;

use crate::{
    app::AppState,
    db::repositories::SqliteUserRepository,
    domain::auth::{AuthUserRecord, UserRole},
    http::error::AppError,
};

pub const SESSION_COOKIE_NAME: &str = "athena_session";
pub const SESSION_TTL_SECONDS: i64 = 60 * 60 * 24 * 30;

pub async fn current_user_from_headers(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Option<AuthUserRecord>, AppError> {
    let Some(session_id) = session_id_from_headers(headers) else {
        return Ok(None);
    };
    SqliteUserRepository::new(state.pool.clone())
        .find_user_by_session(&session_id)
        .await
        .map_err(AppError::from)
}

pub async fn require_user(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AuthUserRecord, AppError> {
    current_user_from_headers(state, headers)
        .await?
        .ok_or_else(|| AppError::Unauthorized("login required".to_string()))
}

pub async fn require_admin(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AuthUserRecord, AppError> {
    let repo = SqliteUserRepository::new(state.pool.clone());
    if repo.setup_required().await.map_err(AppError::from)? {
        return Ok(AuthUserRecord {
            id: "bootstrap-admin".to_string(),
            username: "bootstrap-admin".to_string(),
            role: UserRole::Admin,
        });
    }

    let user = require_user(state, headers).await?;
    if !matches!(user.role, UserRole::Admin) {
        return Err(AppError::Forbidden("admin access required".to_string()));
    }
    Ok(user)
}

pub fn session_id_from_headers(headers: &HeaderMap) -> Option<String> {
    let cookie_header = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
    cookie_header
        .split(';')
        .filter_map(|part| part.trim().split_once('='))
        .find(|(name, _)| *name == SESSION_COOKIE_NAME)
        .map(|(_, value)| value.to_string())
}

pub fn session_cookie_value(session_id: &str) -> String {
    format!(
        "{SESSION_COOKIE_NAME}={session_id}; Path=/; HttpOnly; SameSite=Lax; Max-Age={SESSION_TTL_SECONDS}"
    )
}

pub fn clear_session_cookie_value() -> String {
    format!("{SESSION_COOKIE_NAME}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0")
}
