use axum::{
    body::Body,
    http::{Request, StatusCode, header::LOCATION},
};
use book_router::{app::build_app, config::AppConfig};
use tower::util::ServiceExt;

#[tokio::test]
async fn post_requests_creates_a_work_level_request() {
    let app = build_app(AppConfig::for_tests()).await.unwrap();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/requests")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(
                    "title=The+Hobbit&author=J.R.R.+Tolkien&media_type=audiobook&preferred_language=en&edition_title=&preferred_narrator=Andy+Serkis&preferred_publisher=",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let location = response
        .headers()
        .get(LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("redirect location header");
    assert!(location.starts_with("/requests/"));

    let detail = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(location)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(detail.status(), StatusCode::OK);
    let body = axum::body::to_bytes(detail.into_body(), usize::MAX)
        .await
        .unwrap();
    let body = String::from_utf8(body.to_vec()).unwrap();

    assert!(body.contains("The Hobbit"));
    assert!(body.contains("Andy Serkis"));
    assert!(body.contains("Edition title: Any"));
    assert!(body.contains("Publisher: Any"));
}

#[tokio::test]
async fn request_survives_app_rebuild_with_same_config() {
    let tempdir = tempfile::tempdir().unwrap();
    let config = AppConfig::for_tests_with_database_path(tempdir.path().join("book-router.sqlite"));
    let location = {
        let app = build_app(config.clone()).await.unwrap();

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/requests")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from(
                        "title=The+Hobbit&author=J.R.R.+Tolkien&media_type=audiobook&preferred_language=en&edition_title=&preferred_narrator=Andy+Serkis&preferred_publisher=",
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        response
            .headers()
            .get(LOCATION)
            .and_then(|value| value.to_str().ok())
            .expect("redirect location header")
            .to_owned()
    };

    let rebuilt_app = build_app(config).await.unwrap();
    let detail = rebuilt_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(location)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(detail.status(), StatusCode::OK);
}
