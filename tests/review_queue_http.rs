use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header as http_header},
};
use book_router::{
    app::build_app,
    config::AppConfig,
    db::{
        connect_sqlite,
        repositories::{SqliteRequestRepository, SqliteSettingsRepository},
    },
    domain::{
        requests::{CreateRequest, ManifestationPreference, MediaType},
        search::{ReleaseCandidate, ScoredCandidate},
        settings::{
            DownloadClientSettingsUpdate, QbittorrentSettingsUpdate, RuntimeSettingsUpdate,
        },
    },
};
use serde_json::{Value, json};
use tower::util::ServiceExt;
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const API_PREFIX: &str = "/api/v1";

#[tokio::test]
async fn request_detail_includes_review_queue_and_reject_moves_to_no_match() {
    let tempdir = tempfile::tempdir().unwrap();
    let config = AppConfig::for_tests_with_database_path(tempdir.path().join("book-router.sqlite"));
    let pool = connect_sqlite(&config.database).await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    let repo = SqliteRequestRepository::new(pool.clone());
    let settings = SqliteSettingsRepository::new(pool);
    settings.ensure_seeded(&config).await.unwrap();

    let request = repo
        .create(CreateRequest {
            external_work_id: "OL27448W".into(),
            title: "The Hobbit".into(),
            author: "J.R.R. Tolkien".into(),
            media_type: MediaType::Ebook,
            preferred_language: Some("en".into()),
            manifestation: ManifestationPreference::default(),
        })
        .await
        .unwrap();
    repo.enqueue_review_candidate(
        &request.id,
        &ReleaseCandidate {
            external_id: "candidate-review".into(),
            source: "prowlarr".into(),
            title: "The Hobbit J.R.R. Tolkien".into(),
            protocol: "torrent".into(),
            size_bytes: 1234,
            indexer: "Books".into(),
            download_url: Some("magnet:?xt=urn:btih:cccccccccccccccccccccccccccccccccccccccc".into()),
        },
        &ScoredCandidate {
            score: 0.75,
            explanation: vec!["title matched".into()],
            auto_acquire: false,
        },
    )
    .await
    .unwrap();
    repo.update_state(&request.id, "review").await.unwrap();

    let app = build_app(config).await.unwrap();
    let detail = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("{API_PREFIX}/requests/{}", request.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(detail.status(), StatusCode::OK);
    let body = json_body(detail).await;
    assert_eq!(body["review_queue"].as_array().unwrap().len(), 1);

    let reject = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "{API_PREFIX}/requests/{}/review-queue/{}/reject",
                    request.id, 1
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reject.status(), StatusCode::OK);
    let rejected = json_body(reject).await;
    assert_eq!(rejected["request"]["state"], "no_match");
    assert_eq!(rejected["review_queue"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn approving_review_candidate_dispatches_to_qbittorrent_and_clears_queue() {
    let tempdir = tempfile::tempdir().unwrap();
    let qb_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v2/auth/login"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("set-cookie", "SID=test-session; HttpOnly; Path=/"),
        )
        .mount(&qb_server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v2/torrents/add"))
        .and(body_string_contains("category=athena-ebooks"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&qb_server)
        .await;

    let config = AppConfig::for_tests_with_database_path(tempdir.path().join("book-router.sqlite"));
    let pool = connect_sqlite(&config.database).await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    let repo = SqliteRequestRepository::new(pool.clone());
    let settings = SqliteSettingsRepository::new(pool);
    settings.ensure_seeded(&config).await.unwrap();
    settings
        .update_runtime_settings(RuntimeSettingsUpdate {
            download_clients: Some(DownloadClientSettingsUpdate {
                qbittorrent: Some(QbittorrentSettingsUpdate {
                    enabled: Some(true),
                    base_url: Some(qb_server.uri()),
                    username: Some("admin".to_string()),
                    password: Some("adminadmin".to_string()),
                    category_ebook: Some("athena-ebooks".to_string()),
                    category_audiobook: Some("athena-audiobooks".to_string()),
                    ..Default::default()
                }),
            }),
            ..Default::default()
        })
        .await
        .unwrap();

    let request = repo
        .create(CreateRequest {
            external_work_id: "OL27448W".into(),
            title: "The Hobbit".into(),
            author: "J.R.R. Tolkien".into(),
            media_type: MediaType::Ebook,
            preferred_language: Some("en".into()),
            manifestation: ManifestationPreference::default(),
        })
        .await
        .unwrap();
    repo.enqueue_review_candidate(
        &request.id,
        &ReleaseCandidate {
            external_id: "candidate-approve".into(),
            source: "prowlarr".into(),
            title: "The Hobbit J.R.R. Tolkien".into(),
            protocol: "torrent".into(),
            size_bytes: 1234,
            indexer: "Books".into(),
            download_url: Some("magnet:?xt=urn:btih:dddddddddddddddddddddddddddddddddddddddd".into()),
        },
        &ScoredCandidate {
            score: 0.75,
            explanation: vec!["title matched".into()],
            auto_acquire: false,
        },
    )
    .await
    .unwrap();
    repo.update_state(&request.id, "review").await.unwrap();

    let app = build_app(config).await.unwrap();
    let approve = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "{API_PREFIX}/requests/{}/review-queue/{}/approve",
                    request.id, 1
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(approve.status(), StatusCode::OK);
    let body = json_body(approve).await;
    assert_eq!(body["request"]["state"], "queued");
    assert_eq!(body["review_queue"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn audiobookshelf_settings_can_be_saved_and_tested() {
    let abs_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/libraries/library-1/scan"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&abs_server)
        .await;

    let app = build_app(AppConfig::for_tests()).await.unwrap();
    let save = app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("{API_PREFIX}/settings/integrations/audiobookshelf"),
            json!({
                "enabled": true,
                "base_url": abs_server.uri(),
                "library_id": "library-1",
                "api_key": "abs-secret"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(save.status(), StatusCode::OK);
    let body = json_body(save).await;
    assert_eq!(body["enabled"], true);
    assert_eq!(body["has_api_key"], true);

    let test = app
        .oneshot(json_request(
            "POST",
            &format!("{API_PREFIX}/settings/integrations/audiobookshelf/test"),
            json!({
                "enabled": true,
                "base_url": abs_server.uri(),
                "library_id": "library-1",
                "api_key": "abs-secret"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(test.status(), StatusCode::OK);
    assert_eq!(
        json_body(test).await["message"],
        "Audiobookshelf connection succeeded"
    );
}

fn json_request(method: &str, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(http_header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}
