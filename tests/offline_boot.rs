use axum::{body::Body, http::{Request, StatusCode}};
use book_router::{app::build_app, config::AppConfig};
use tower::util::ServiceExt;

#[tokio::test]
async fn app_boots_with_invalid_metadata_url() {
    let mut config = AppConfig::for_tests();
    // Use an unreachable local URL to simulate offline/invalid state
    config.metadata_base_url = "http://invalid-metadata-url-that-does-not-exist.local".into();
    
    let app = build_app(config).await.expect("App should boot even if metadata is unreachable");
    
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
