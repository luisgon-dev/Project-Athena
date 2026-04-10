use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use book_router::{
    app::build_app,
    config::AppConfig,
    db::connect_sqlite,
};
use serde_json::json;
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
    assert!(body.contains("name=\"selected_work_id\""));
    assert!(body.contains("value=\"OL27448W\""));
    assert!(body.contains("The Hobbit (Graphic Novel)"));
}

#[tokio::test]
async fn post_requests_with_both_media_types_creates_two_requests_from_provider_canonical_data() {
    let server = MockServer::start().await;
    let app = build_app(AppConfig::for_tests().with_metadata_base_url(server.uri()))
        .await
        .unwrap();
    stub_work_lookup(
        &server,
        "OL27448W",
        "The Hobbit: There and Back Again",
        "/authors/OL26320A",
        "J.R.R. Tolkien",
    )
    .await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/requests")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(
                    "selected_work_id=OL27448W&ebook=on&audiobook=on&preferred_language=en&edition_title=&preferred_narrator=Andy+Serkis&preferred_publisher=&graphic_audio=on",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = body_text(response).await;

    assert!(body.contains("Created requests"));
    assert!(body.matches("/requests/").count() >= 2);
    assert!(body.contains("The Hobbit: There and Back Again"));
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
        assert!(detail_body.contains("The Hobbit: There and Back Again"));
        assert!(detail_body.contains("J.R.R. Tolkien"));
        assert!(detail_body.contains("OL27448W"));
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
async fn post_requests_rejects_forged_selected_work_not_backed_by_metadata() {
    let server = MockServer::start().await;
    let app = build_app(AppConfig::for_tests().with_metadata_base_url(server.uri()))
        .await
        .unwrap();
    Mock::given(method("GET"))
        .and(path("/works/OL27448W.json"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/requests")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("selected_work_id=OL27448W&audiobook=on"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn request_survives_app_rebuild_with_same_config() {
    let tempdir = tempfile::tempdir().unwrap();
    let server = MockServer::start().await;
    let config = AppConfig::for_tests_with_database_path(tempdir.path().join("book-router.sqlite"))
        .with_metadata_base_url(server.uri());
    stub_work_lookup(
        &server,
        "OL27448W",
        "The Hobbit: There and Back Again",
        "/authors/OL26320A",
        "J.R.R. Tolkien",
    )
    .await;

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
                        "selected_work_id=OL27448W&audiobook=on&preferred_language=en&edition_title=&preferred_narrator=Andy+Serkis&preferred_publisher=",
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
    let detail_body = body_text(detail).await;
    assert!(detail_body.contains("OL27448W"));
}

#[tokio::test]
async fn build_app_backfills_missing_canonical_work_identity_for_legacy_requests() {
    let tempdir = tempfile::tempdir().unwrap();
    let server = MockServer::start().await;
    let config = AppConfig::for_tests_with_database_path(tempdir.path().join("book-router.sqlite"))
        .with_metadata_base_url(server.uri());
    let pool = connect_sqlite(&config.database).await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    sqlx::query(
        "INSERT INTO requests (id, external_work_id, title, author, media_type, preferred_language, state, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)",
    )
    .bind("legacy-request")
    .bind("")
    .bind("The Hobbit")
    .bind("J.R.R. Tolkien")
    .bind("audiobook")
    .bind(Option::<String>::None)
    .bind("requested")
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO request_events (request_id, kind, payload_json, created_at)
         VALUES (?, ?, ?, CURRENT_TIMESTAMP)",
    )
    .bind("legacy-request")
    .bind("request.created")
    .bind(
        json!({
            "request_id": "legacy-request",
            "title": "The Hobbit",
            "author": "J.R.R. Tolkien",
            "media_type": "audiobook",
            "preferred_language": null,
            "manifestation": {
                "edition_title": null,
                "preferred_narrator": null,
                "preferred_publisher": null,
                "graphic_audio": false
            }
        })
        .to_string(),
    )
    .execute(&pool)
    .await
    .unwrap();

    stub_search(
        &server,
        "The Hobbit",
        "J.R.R. Tolkien",
        "5",
        r#"{
            "docs":[{"key":"OL27448W","title":"The Hobbit","author_name":["J.R.R. Tolkien"]}]
        }"#,
    )
    .await;

    let app = build_app(config).await.unwrap();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/requests/legacy-request")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(body.contains("OL27448W"));
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

async fn stub_search(
    server: &MockServer,
    title: &str,
    author: &str,
    limit: &str,
    body: &str,
) {
    Mock::given(method("GET"))
        .and(path("/search.json"))
        .and(query_param("title", title))
        .and(query_param("author", author))
        .and(query_param("limit", limit))
        .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
        .mount(server)
        .await;
}

async fn stub_work_lookup(
    server: &MockServer,
    work_id: &str,
    title: &str,
    author_key: &str,
    author_name: &str,
) {
    Mock::given(method("GET"))
        .and(path(format!("/works/{work_id}.json")))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            format!(
                r#"{{
                    "key": "/works/{work_id}",
                    "title": "{title}",
                    "authors": [{{ "author": {{ "key": "{author_key}" }} }}]
                }}"#
            ),
            "application/json",
        ))
        .mount(server)
        .await;

    Mock::given(method("GET"))
        .and(path(format!("{author_key}.json")))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            format!(r#"{{"name":"{author_name}"}}"#),
            "application/json",
        ))
        .mount(server)
        .await;
}
