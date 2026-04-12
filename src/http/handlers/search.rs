use axum::Json;

use crate::domain::search::ScoredCandidate;

pub async fn review_queue() -> Json<Vec<ScoredCandidate>> {
    Json(Vec::new())
}
