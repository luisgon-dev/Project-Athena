use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use book_router::{app::build_app, config::AppConfig};
use tower::util::ServiceExt;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn get_requests_searches_open_library_and_renders_matches() {
    let server = MockServer::start().await;
    let app = build_app(AppConfig::for_tests().with_metadata_base_url(server.uri()))
        .await
        .unwrap();

    Mock::given(method("GET"))
        .and(path("/search.json"))
        .and(query_param("title", "The Hobbit"))
        .and(query_param("author", "Tolkien"))
        .and(query_param("limit", "10"))
        .and(header("user-agent", "book-router/0.1"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{
            "docs": [
                {"key":"OL27448W","title":"The Hobbit","author_name":["J.R.R. Tolkien"]},
                {"key":"OL99999W","title":"The Hobbit (Graphic Novel)","author_name":["David Wenzel"]}
            ]
        }"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/requests?title=The+Hobbit&author=Tolkien")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;

    assert!(body.contains("Search results"));
    assert!(body.contains("The Hobbit"));
    assert!(body.contains("J.R.R. Tolkien"));
    assert!(body.contains("value=\"OL27448W|The Hobbit|J.R.R. Tolkien\""));
    assert!(body.contains("The Hobbit (Graphic Novel)"));
}

#[tokio::test]
async fn post_requests_with_both_media_types_creates_two_requests() {
    let server = MockServer::start().await;
    let app = build_app(AppConfig::for_tests().with_metadata_base_url(server.uri()))
        .await
        .unwrap();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/requests")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(
                    "selected_work=OL27448W%7CThe+Hobbit%7CJ.R.R.+Tolkien&ebook=on&audiobook=on&preferred_language=en&edition_title=&preferred_narrator=Andy+Serkis&preferred_publisher=&graphic_audio=on",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = body_text(response).await;

    assert!(body.contains("Created requests"));
    assert!(body.matches("/requests/").count() >= 2);
    assert!(body.contains("The Hobbit"));
    assert!(body.contains("Ebook"));
    assert!(body.contains("Audiobook"));

    let links = extract_request_links(&body);
    assert_eq!(links.len(), 2);

    for link in links {
        let detail = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(&link)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(detail.status(), StatusCode::OK);
        let detail_body = body_text(detail).await;
        assert!(detail_body.contains("The Hobbit"));
        assert!(detail_body.contains("Andy Serkis"));
        assert!(detail_body.contains("Graphic audio: true"));
    }
}

#[tokio::test]
async fn post_requests_rejects_raw_free_text_creation_without_selected_match() {
    let app = build_app(AppConfig::for_tests()).await.unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/requests")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(
                    "title=The+Hobbit&author=J.R.R.+Tolkien&audiobook=on",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
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
                        "selected_work=OL27448W%7CThe+Hobbit%7CJ.R.R.+Tolkien&audiobook=on&preferred_language=en&edition_title=&preferred_narrator=Andy+Serkis&preferred_publisher=",
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        extract_request_links(&body_text(response).await)
            .into_iter()
            .next()
            .expect("request link")
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

async fn body_text(response: axum::response::Response) -> String {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    String::from_utf8(body.to_vec()).unwrap()
}

fn extract_request_links(body: &str) -> Vec<String> {
    body.split('"')
        .filter(|value| value.starts_with("/requests/"))
        .map(str::to_string)
        .collect()
}
