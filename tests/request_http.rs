use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header as http_header},
};
use book_router::{app::build_app, config::AppConfig, db::connect_sqlite};
use serde_json::json;
use tower::util::ServiceExt;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn get_requests_lists_active_requests_and_statuses() {
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
                .uri("/requests")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(body.contains("Active requests"));
    assert!(body.contains("Newer Book"));
    assert!(body.contains("Older Book"));
    assert!(body.contains("imported"));
    assert!(body.contains("requested"));
    assert!(body.contains("/requests/new"));
    assert!(body.contains("/requests/newer-request"));
    assert!(body.contains("/requests/older-request"));
    assert!(body.find("Newer Book").unwrap() < body.find("Older Book").unwrap());
}

#[tokio::test]
async fn get_requests_new_searches_open_library_and_renders_enriched_matches() {
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
                .uri("/requests/new?title=The+Hobbit&author=Tolkien")
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
    assert!(body.contains("1937"));
    assert!(body.contains("Bilbo Baggins leaves the Shire"));
    assert!(body.contains("Fantasy"));
    assert!(body.contains("42 editions"));
    assert!(body.contains("Canonical work ID: OL27448W"));
    assert!(body.contains("/covers/openlibrary/2468"));
    assert!(body.contains("name=\"selected_work_id\""));
    assert!(body.contains("value=\"OL27448W\""));
    assert!(body.contains("The Hobbit (Graphic Novel)"));
}

#[tokio::test]
async fn get_requests_new_accepts_title_only_searches_and_normalizes_work_keys() {
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
                .uri("/requests/new?title=Dune")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;

    assert!(body.contains("value=\"OL123W\""));
    assert!(!body.contains("value=\"/works/OL123W\""));
}

#[tokio::test]
async fn get_requests_new_accepts_author_only_searches() {
    let server = MockServer::start().await;
    let app = build_app(AppConfig::for_tests().with_metadata_base_url(server.uri()))
        .await
        .unwrap();

    Mock::given(method("GET"))
        .and(path("/search.json"))
        .and(query_param("title", ""))
        .and(query_param("author", "Tolkien"))
        .and(query_param("limit", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{
                "docs":[
                    {"key":"OL27448W","title":"The Hobbit","author_name":["J.R.R. Tolkien"]}
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
                "description":"A hobbit goes on an adventure."
            }"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/requests/new?author=Tolkien")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(body.contains("The Hobbit"));
    assert!(body.contains("J.R.R. Tolkien"));
}

#[tokio::test]
async fn get_requests_new_renders_clean_no_match_state() {
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
                .uri("/requests/new?title=No+Match")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(body.contains("No works matched that search."));
    assert!(!body.contains("Create request"));
}

#[tokio::test]
async fn search_then_request_flow_handles_missing_author_metadata() {
    let server = MockServer::start().await;
    let app = build_app(AppConfig::for_tests().with_metadata_base_url(server.uri()))
        .await
        .unwrap();

    Mock::given(method("GET"))
        .and(path("/search.json"))
        .and(query_param("title", "Mystery Book"))
        .and(query_param("author", ""))
        .and(query_param("limit", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{
            "docs": [
                {"key":"/works/OL999W","title":"Mystery Book"}
            ]
        }"#,
            "application/json",
        ))
        .mount(&server)
        .await;

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

    let search_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/requests/new?title=Mystery+Book")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(search_response.status(), StatusCode::OK);
    let search_body = body_text(search_response).await;
    assert!(search_body.contains("Mystery Book"));
    assert!(search_body.contains("Unknown author"));

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/requests")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("selected_work_id=OL999W&ebook=on"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::CREATED);
    let create_body = body_text(create_response).await;
    assert!(create_body.contains("Unknown author"));

    let location = extract_request_links(&create_body)
        .into_iter()
        .next()
        .expect("created request link");
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
    let detail_body = body_text(detail).await;
    assert!(detail_body.contains("Unknown author"));
}

#[tokio::test]
async fn post_requests_still_works_after_title_only_search() {
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
                "docs":[{"key":"OL123W","title":"Dune","author_name":["Frank Herbert"]}]
            }"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    stub_work_lookup(&server, "OL123W", "Dune", "/authors/OL1A", "Frank Herbert").await;

    let search = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/requests/new?title=Dune")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(search.status(), StatusCode::OK);
    let search_body = body_text(search).await;
    assert!(search_body.contains("value=\"OL123W\""));

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/requests")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("selected_work_id=OL123W&ebook=on"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create.status(), StatusCode::CREATED);
    let create_body = body_text(create).await;
    assert!(create_body.contains("Dune"));
    assert!(create_body.contains("Frank Herbert"));
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
                .uri("/covers/openlibrary/2468")
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
                .uri("/covers/openlibrary/999")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
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
async fn get_requests_new_returns_json_error_on_metadata_timeout() {
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
                .uri("/requests/new?title=Timeout")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::GATEWAY_TIMEOUT);
    let body = body_text(response).await;
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["error"], "Metadata service timed out");
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

    let mut body = String::new();
    for _ in 0..10 {
        let response = app
            .clone()
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
        body = body_text(response).await;
        if body.contains("OL27448W") {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    assert!(body.contains("OL27448W"));
    assert!(body.contains("The Hobbit"));
    assert!(!body.contains("The Hob-bit!"));
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
                .uri("/requests/legacy-unresolved")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(body.contains("Missing Book"));
    assert!(body.contains("Unresolved"));
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
