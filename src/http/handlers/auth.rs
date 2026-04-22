use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header::SET_COOKIE},
};

use crate::{
    app::AppState,
    db::repositories::SqliteUserRepository,
    domain::auth::{
        AuthBootstrapStatus, AuthUserRecord, CreateUserRequest, LoginRequest, SessionRecord,
        SetupRequest, UpdateUserRequest, UserRecord,
    },
    http::{
        auth::{
            SESSION_TTL_SECONDS, clear_session_cookie_value, require_admin, require_user,
            session_cookie_value, session_id_from_headers,
        },
        error::AppError,
    },
};

pub async fn auth_bootstrap(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AuthBootstrapStatus>, AppError> {
    let session_id = session_id_from_headers(&headers);
    let repo = SqliteUserRepository::new(state.pool);
    Ok(Json(repo.bootstrap_status(session_id.as_deref()).await?))
}

pub async fn auth_setup(
    State(state): State<AppState>,
    Json(payload): Json<SetupRequest>,
) -> Result<(StatusCode, HeaderMap, Json<AuthUserRecord>), AppError> {
    let repo = SqliteUserRepository::new(state.pool.clone());
    let user = repo.create_initial_admin(payload).await?;
    let session = repo.create_session(&user.id, SESSION_TTL_SECONDS).await?;
    Ok(auth_response(StatusCode::CREATED, user_to_auth_user(user), session)?)
}

pub async fn auth_login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<(StatusCode, HeaderMap, Json<AuthUserRecord>), AppError> {
    let repo = SqliteUserRepository::new(state.pool.clone());
    let user = repo
        .verify_login(payload)
        .await?
        .ok_or_else(|| AppError::Unauthorized("invalid username or password".to_string()))?;
    let session = repo.create_session(&user.id, SESSION_TTL_SECONDS).await?;
    Ok(auth_response(StatusCode::OK, user, session)?)
}

pub async fn auth_logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(StatusCode, HeaderMap), AppError> {
    if let Some(session_id) = session_id_from_headers(&headers) {
        SqliteUserRepository::new(state.pool)
            .delete_session(&session_id)
            .await?;
    }

    let mut response_headers = HeaderMap::new();
    response_headers.insert(
        SET_COOKIE,
        HeaderValue::from_str(&clear_session_cookie_value())
            .map_err(|error| AppError::Internal(error.into()))?,
    );
    Ok((StatusCode::NO_CONTENT, response_headers))
}

pub async fn auth_me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AuthUserRecord>, AppError> {
    Ok(Json(require_user(&state, &headers).await?))
}

pub async fn users_index(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<UserRecord>>, AppError> {
    require_admin(&state, &headers).await?;
    Ok(Json(SqliteUserRepository::new(state.pool).list().await?))
}

pub async fn users_create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserRecord>), AppError> {
    require_admin(&state, &headers).await?;
    let user = SqliteUserRepository::new(state.pool)
        .create_user(payload)
        .await?;
    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn users_update(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<UserRecord>, AppError> {
    require_admin(&state, &headers).await?;
    let user = SqliteUserRepository::new(state.pool)
        .update_user(&id, payload)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("user {id} not found")))?;
    Ok(Json(user))
}

fn auth_response(
    status: StatusCode,
    user: AuthUserRecord,
    session: SessionRecord,
) -> Result<(StatusCode, HeaderMap, Json<AuthUserRecord>), AppError> {
    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        HeaderValue::from_str(&session_cookie_value(&session.id))
            .map_err(|error| AppError::Internal(error.into()))?,
    );
    Ok((status, headers, Json(user)))
}

fn user_to_auth_user(user: UserRecord) -> AuthUserRecord {
    AuthUserRecord {
        id: user.id,
        username: user.username,
        role: user.role,
    }
}
