use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use book_router::{app::build_app, config::AppConfig};
use tower::util::ServiceExt;

#[tokio::test]
async fn health_returns_ok() {
    let config = AppConfig::for_tests();
    let app = build_app(config).await.unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
