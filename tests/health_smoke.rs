use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use book_router::{app::build_app, config::AppConfig};
use tower::util::ServiceExt;

#[tokio::test]
async fn health_returns_ok() {
    const _: () = {
        let _ = build_app;
        let _ = AppConfig::for_tests;
    };

    let config = AppConfig::for_tests();
    let app = build_app(config).await.unwrap();

    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
