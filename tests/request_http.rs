use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header as http_header},
};
use book_router::{app::build_app, config::AppConfig, db::connect_sqlite};
use serde_json::{Value, json};
use tower::util::ServiceExt;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const API_PREFIX: &str = "/api/v1";

#[tokio::test]
async fn get_requests_lists_active_requests_and_statuses_as_json() {
    let tempdir = tempfile::tempdir().unwrap();
    let config = AppConfig::for_tests_with_database_path(tempdir.path().join("book-router.sqlite"));
    let pool = connect_sqlite(&config.database).await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    sqlx::query(
        "INSERT INTO requests (id, external_work_id, title, author, media_type, preferred_language, state, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("older-request")
    .bind("OL100W")
    .bind("Older Book")
    .bind("Old Author")
    .bind("ebook")
    .bind(Option::<String>::None)
    .bind("requested")
    .bind("2026-04-10 00:00:00")
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO requests (id, external_work_id, title, author, media_type, preferred_language, state, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("newer-request")
    .bind("OL200W")
    .bind("Newer Book")
    .bind("New Author")
    .bind("audiobook")
    .bind(Option::<String>::None)
    .bind("imported")
    .bind("2026-04-10 00:00:01")
    .execute(&pool)
    .await
    .unwrap();

    let app = build_app(config).await.unwrap();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("{API_PREFIX}/requests"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_json_content_type(&response);
    let body = json_body(response).await;
    let requests = body.as_array().unwrap();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0]["id"], "newer-request");
    assert_eq!(requests[0]["title"], "Newer Book");
    assert_eq!(requests[0]["media_type"], "Audiobook");
    assert_eq!(requests[0]["state"], "imported");
    assert_eq!(requests[1]["id"], "older-request");
    assert_eq!(requests[1]["media_type"], "Ebook");
}

#[tokio::test]
async fn get_requests_searches_open_library_and_returns_enriched_matches_as_json() {
    let server = MockServer::start().await;
    let app = build_app(
        AppConfig::for_tests()
            .with_metadata_base_url(server.uri())
            .with_cover_base_url(server.uri()),
    )
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
                {
                    "key":"OL27448W",
                    "title":"The Hobbit",
                    "author_name":["J.R.R. Tolkien"],
                    "first_publish_year":1937,
                    "cover_i":2468,
                    "subject":["Fantasy","Middle Earth","Adventure"],
                    "edition_count":42
                },
                {
                    "key":"OL99999W",
                    "title":"The Hobbit (Graphic Novel)",
                    "author_name":["David Wenzel"],
                    "first_publish_year":1989,
                    "cover_i":8642,
                    "subject":["Graphic novels","Fantasy"],
                    "edition_count":9
                }
            ]
        }"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/works/OL27448W.json"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{
                "key":"/works/OL27448W",
                "title":"The Hobbit",
                "description":{"value":"Bilbo Baggins leaves the Shire for an unexpected journey."},
                "subjects":["Fantasy","Adventure","Dragons"],
                "covers":[2468]
            }"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/works/OL99999W.json"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{
                "key":"/works/OL99999W",
                "title":"The Hobbit (Graphic Novel)",
                "description":"Illustrated adaptation of the classic fantasy.",
                "subjects":["Graphic novels","Fantasy"],
                "covers":[8642]
            }"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("{API_PREFIX}/requests/search?title=The+Hobbit&author=Tolkien"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_json_content_type(&response);
    let body = json_body(response).await;
    let works = body["works"].as_array().unwrap();
    assert_eq!(works.len(), 2);
    assert_eq!(works[0]["external_id"], "OL27448W");
    assert_eq!(works[0]["title"], "The Hobbit");
    assert_eq!(works[0]["primary_author"], "J.R.R. Tolkien");
    assert_eq!(works[0]["first_publish_year"], 1937);
    assert_eq!(
        works[0]["description"],
        "Bilbo Baggins leaves the Shire for an unexpected journey."
    );
    assert_eq!(works[0]["cover_id"], 2468);
    assert_eq!(works[0]["edition_count"], 42);
}

#[tokio::test]
async fn get_requests_search_accepts_title_only_queries_and_normalizes_keys() {
    let server = MockServer::start().await;
    let app = build_app(AppConfig::for_tests().with_metadata_base_url(server.uri()))
        .await
        .unwrap();

    Mock::given(method("GET"))
        .and(path("/search.json"))
        .and(query_param("title", "Dune"))
        .and(query_param("author", ""))
        .and(query_param("limit", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{
            "docs": [
                {"key":"/works/OL123W","title":"Dune","author_name":["Frank Herbert"]}
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
                .uri(format!("{API_PREFIX}/requests/search?title=Dune"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["works"][0]["external_id"], "OL123W");
}

#[tokio::test]
async fn get_requests_search_returns_empty_works_when_there_is_no_match() {
    let server = MockServer::start().await;
    let app = build_app(AppConfig::for_tests().with_metadata_base_url(server.uri()))
        .await
        .unwrap();

    Mock::given(method("GET"))
        .and(path("/search.json"))
        .and(query_param("title", "No Match"))
        .and(query_param("author", ""))
        .and(query_param("limit", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(r#"{"docs":[]}"#, "application/json"))
        .mount(&server)
        .await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("{API_PREFIX}/requests/search?title=No+Match"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["works"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn post_requests_creates_requests_and_detail_includes_event_history() {
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
        .oneshot(json_request(
            "POST",
            &format!("{API_PREFIX}/requests"),
            json!({
                "selected_work_id": "OL27448W",
                "media_types": ["Ebook", "Audiobook"],
                "preferred_language": "en",
                "manifestation": {
                    "edition_title": null,
                    "preferred_narrator": "Andy Serkis",
                    "preferred_publisher": null,
                    "graphic_audio": true
                }
            }),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    assert_json_content_type(&response);
    let body = json_body(response).await;
    let created = body.as_array().unwrap();
    assert_eq!(created.len(), 2);
    assert_eq!(created[0]["title"], "The Hobbit: There and Back Again");
    assert_eq!(created[0]["author"], "J.R.R. Tolkien");

    for created_request in created {
        let request_id = created_request["id"].as_str().unwrap();
        let detail = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("{API_PREFIX}/requests/{request_id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(detail.status(), StatusCode::OK);
        let detail_body = json_body(detail).await;
        assert_eq!(detail_body["request"]["external_work_id"], "OL27448W");
        assert_eq!(detail_body["request"]["title"], "The Hobbit: There and Back Again");
        assert_eq!(detail_body["request"]["manifestation"]["preferred_narrator"], "Andy Serkis");
        assert_eq!(detail_body["events"].as_array().unwrap().len(), 1);
        assert_eq!(detail_body["events"][0]["kind"], "Created");
    }
}

#[tokio::test]
async fn post_requests_handles_missing_author_metadata_in_json_flow() {
    let server = MockServer::start().await;
    let app = build_app(AppConfig::for_tests().with_metadata_base_url(server.uri()))
        .await
        .unwrap();

    Mock::given(method("GET"))
        .and(path("/works/OL999W.json"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{
                "key": "/works/OL999W",
                "title": "Mystery Book",
                "authors": []
            }"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let create_response = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("{API_PREFIX}/requests"),
            json!({
                "selected_work_id": "OL999W",
                "media_types": ["Ebook"],
                "preferred_language": null,
                "manifestation": {
                    "edition_title": null,
                    "preferred_narrator": null,
                    "preferred_publisher": null,
                    "graphic_audio": false
                }
            }),
        ))
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::CREATED);
    let created = json_body(create_response).await;
    assert_eq!(created[0]["author"], "Unknown author");

    let request_id = created[0]["id"].as_str().unwrap();
    let detail = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("{API_PREFIX}/requests/{request_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(detail.status(), StatusCode::OK);
    let detail_body = json_body(detail).await;
    assert_eq!(detail_body["request"]["author"], "Unknown author");
}

#[tokio::test]
async fn post_requests_rejects_invalid_json_payloads() {
    let app = build_app(AppConfig::for_tests()).await.unwrap();

    let missing_selection = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("{API_PREFIX}/requests"),
            json!({
                "media_types": ["Audiobook"],
                "manifestation": {
                    "edition_title": null,
                    "preferred_narrator": null,
                    "preferred_publisher": null,
                    "graphic_audio": false
                }
            }),
        ))
        .await
        .unwrap();

    assert_eq!(missing_selection.status(), StatusCode::BAD_REQUEST);
    assert_eq!(json_body(missing_selection).await["error"], "missing selected_work_id");

    let missing_media_types = app
        .oneshot(json_request(
            "POST",
            &format!("{API_PREFIX}/requests"),
            json!({
                "selected_work_id": "OL27448W",
                "media_types": [],
                "manifestation": {
                    "edition_title": null,
                    "preferred_narrator": null,
                    "preferred_publisher": null,
                    "graphic_audio": false
                }
            }),
        ))
        .await
        .unwrap();

    assert_eq!(missing_media_types.status(), StatusCode::BAD_REQUEST);
    assert_eq!(json_body(missing_media_types).await["error"], "no media types selected");
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
        .oneshot(json_request(
            "POST",
            &format!("{API_PREFIX}/requests"),
            json!({
                "selected_work_id": "OL27448W",
                "media_types": ["Audiobook"],
                "manifestation": {
                    "edition_title": null,
                    "preferred_narrator": null,
                    "preferred_publisher": null,
                    "graphic_audio": false
                }
            }),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(json_body(response).await["error"], "selected work id not found");
}

#[tokio::test]
async fn request_survives_app_rebuild_with_same_config_using_api_routes() {
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

    let request_id = {
        let app = build_app(config.clone()).await.unwrap();
        let response = app
            .oneshot(json_request(
                "POST",
                &format!("{API_PREFIX}/requests"),
                json!({
                    "selected_work_id": "OL27448W",
                    "media_types": ["Audiobook"],
                    "preferred_language": "en",
                    "manifestation": {
                        "edition_title": null,
                        "preferred_narrator": "Andy Serkis",
                        "preferred_publisher": null,
                        "graphic_audio": false
                    }
                }),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        json_body(response).await[0]["id"].as_str().unwrap().to_string()
    };

    let rebuilt_app = build_app(config).await.unwrap();
    let detail = rebuilt_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("{API_PREFIX}/requests/{request_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(detail.status(), StatusCode::OK);
    assert_eq!(json_body(detail).await["request"]["external_work_id"], "OL27448W");
}

#[tokio::test]
async fn get_requests_search_returns_json_error_on_metadata_timeout() {
    let server = MockServer::start().await;
    let app = build_app(
        AppConfig::for_tests()
            .with_metadata_base_url(server.uri())
            .with_cover_base_url(server.uri()),
    )
    .await
    .unwrap();

    Mock::given(method("GET"))
        .and(path("/search.json"))
        .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(11)))
        .mount(&server)
        .await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("{API_PREFIX}/requests/search?title=Timeout"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::GATEWAY_TIMEOUT);
    let body = json_body(response).await;
    assert_eq!(body["error"], "Metadata service timed out");
}

#[tokio::test]
async fn get_openlibrary_cover_proxy_passthroughs_image_and_content_type() {
    let server = MockServer::start().await;
    let app = build_app(AppConfig::for_tests().with_cover_base_url(server.uri()))
        .await
        .unwrap();

    Mock::given(method("GET"))
        .and(path("/b/id/2468-M.jpg"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "image/jpeg")
                .set_body_bytes(vec![1_u8, 2, 3, 4]),
        )
        .mount(&server)
        .await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("{API_PREFIX}/covers/openlibrary/2468"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(http_header::CONTENT_TYPE).unwrap(),
        "image/jpeg"
    );
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(body.as_ref(), &[1, 2, 3, 4]);
}

#[tokio::test]
async fn get_openlibrary_cover_proxy_uses_default_size_and_passes_through_404() {
    let server = MockServer::start().await;
    let app = build_app(AppConfig::for_tests().with_cover_base_url(server.uri()))
        .await
        .unwrap();

    Mock::given(method("GET"))
        .and(path("/b/id/999-M.jpg"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("{API_PREFIX}/covers/openlibrary/999"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
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
    .bind("The Hob-bit!")
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
            "title": "The Hob-bit!",
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
        "The Hob-bit!",
        "J.R.R. Tolkien",
        "5",
        r#"{
            "docs":[{"key":"OL27448W","title":"The Hobbit","author_name":["J.R.R. Tolkien"]}]
        }"#,
    )
    .await;

    let app = build_app(config).await.unwrap();

    let mut body = json!({});
    for _ in 0..10 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("{API_PREFIX}/requests/legacy-request"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        body = json_body(response).await;
        if body["request"]["external_work_id"] == "OL27448W" {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    assert_eq!(body["request"]["external_work_id"], "OL27448W");
    assert_eq!(body["request"]["title"], "The Hobbit");
    assert_ne!(body["request"]["title"], "The Hob-bit!");
}

#[tokio::test]
async fn build_app_tolerates_unresolvable_legacy_requests() {
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
    .bind("legacy-unresolved")
    .bind("")
    .bind("Missing Book")
    .bind("Unknown Author")
    .bind("ebook")
    .bind(Option::<String>::None)
    .bind("requested")
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO request_events (request_id, kind, payload_json, created_at)
         VALUES (?, ?, ?, CURRENT_TIMESTAMP)",
    )
    .bind("legacy-unresolved")
    .bind("request.created")
    .bind(
        json!({
            "request_id": "legacy-unresolved",
            "title": "Missing Book",
            "author": "Unknown Author",
            "media_type": "ebook",
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

    Mock::given(method("GET"))
        .and(path("/search.json"))
        .and(query_param("title", "Missing Book"))
        .and(query_param("author", "Unknown Author"))
        .and(query_param("limit", "5"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let app = build_app(config).await.unwrap();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("{API_PREFIX}/requests/legacy-unresolved"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["request"]["title"], "Missing Book");
    assert_eq!(body["request"]["external_work_id"], "");
}

#[tokio::test]
async fn root_serves_the_frontend_shell() {
    let app = build_app(AppConfig::for_tests()).await.unwrap();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(http_header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap(),
        "text/html"
    );
    let body = text_body(response).await;
    assert!(body.contains("<!doctype html>"));
}

fn json_request(method: &str, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(http_header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn assert_json_content_type(response: &axum::response::Response) {
    assert_eq!(
        response
            .headers()
            .get(http_header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap(),
        "application/json"
    );
}

async fn text_body(response: axum::response::Response) -> String {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    String::from_utf8(body.to_vec()).unwrap()
}

async fn json_body(response: axum::response::Response) -> Value {
    serde_json::from_str(&text_body(response).await).unwrap()
}

async fn stub_search(server: &MockServer, title: &str, author: &str, limit: &str, body: &str) {
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
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(format!(r#"{{"name":"{author_name}"}}"#), "application/json"),
        )
        .mount(server)
        .await;
}
