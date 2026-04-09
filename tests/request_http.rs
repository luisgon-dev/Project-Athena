use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use book_router::{app::build_app, config::AppConfig};
use tower::util::ServiceExt;

#[tokio::test]
async fn post_requests_creates_a_work_level_request() {
    let app = build_app(AppConfig::for_tests()).await.unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/requests")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(
                    "title=The+Hobbit&author=J.R.R.+Tolkien&media_type=audiobook&preferred_language=en&preferred_narrator=Andy+Serkis",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
}
