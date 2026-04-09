use axum::Router;

use crate::{config::AppConfig, http::handlers::health::health};

pub fn build_app(config: AppConfig) -> anyhow::Result<Router> {
    config.validate()?;

    Ok(Router::new().route("/health", axum::routing::get(health)))
}
