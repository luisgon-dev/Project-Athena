use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{Response, StatusCode, header},
};
use serde::Deserialize;
use tracing::warn;

use crate::{app::AppState, metadata::openlibrary::CoverSize};

#[derive(Deserialize)]
pub struct CoverQuery {
    pub size: Option<String>,
}

pub async fn openlibrary_cover(
    State(state): State<AppState>,
    Path(cover_id): Path<i64>,
    Query(query): Query<CoverQuery>,
) -> Result<Response<Body>, StatusCode> {
    let size = CoverSize::from_query_value(query.size.as_deref());
    let open_library = state.open_library_client().await.map_err(|error| {
        warn!(cover_id, size = %size.as_str(), error = %error, "cover proxy settings load failed");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let Some(image) = open_library
        .fetch_cover(cover_id, size)
        .await
        .map_err(|error| {
            warn!(cover_id, size = %size.as_str(), error = %error, "cover proxy request failed");
            StatusCode::BAD_GATEWAY
        })?
    else {
        return Err(StatusCode::NOT_FOUND);
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, image.content_type)
        .body(Body::from(image.bytes))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
