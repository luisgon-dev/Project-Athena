use book_router::{
    config::DatabaseTarget,
    db::{
        connect_sqlite,
        repositories::{SqliteRequestRepository, SqliteSettingsRepository},
    },
    domain::{
        requests::{CreateRequest, ManifestationPreference, MediaType},
        settings::{
            DownloadClientSettingsUpdate, IntegrationSettingsUpdate, ProwlarrIntegrationUpdate,
            QbittorrentSettingsUpdate, RuntimeSettingsUpdate,
        },
    },
    workers::search_worker::SearchWorker,
};
use wiremock::matchers::{body_string_contains, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn search_worker_auto_acquires_high_confidence_candidates() {
    let pool = connect_sqlite(&DatabaseTarget::memory()).await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    let settings = SqliteSettingsRepository::new(pool.clone());
    settings
        .ensure_seeded(&book_router::config::AppConfig::for_tests())
        .await
        .unwrap();

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

    let prowlarr_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/search"))
        .and(header("X-Api-Key", "prowlarr-key"))
        .and(query_param("query", "The Hobbit J.R.R. Tolkien"))
        .and(query_param("type", "book"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"[
                {
                    "guid":"candidate-1",
                    "title":"The Hobbit J.R.R. Tolkien EPUB",
                    "size":1234,
                    "protocol":"torrent",
                    "indexer":"Books",
                    "downloadUrl":"magnet:?xt=urn:btih:0123456789abcdef0123456789abcdef01234567&dn=The+Hobbit"
                }
            ]"#,
            "application/json",
        ))
        .mount(&prowlarr_server)
        .await;

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
            integrations: Some(IntegrationSettingsUpdate {
                prowlarr: Some(ProwlarrIntegrationUpdate {
                    enabled: Some(true),
                    base_url: Some(prowlarr_server.uri()),
                    api_key: Some("prowlarr-key".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        })
        .await
        .unwrap();

    let repo = SqliteRequestRepository::new(pool.clone());
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

    SearchWorker::process_request_by_id(pool.clone(), settings.clone(), &request.id)
        .await
        .unwrap();

    let updated = repo.find_by_id(&request.id).await.unwrap().unwrap();
    assert_eq!(updated.state, "queued");
    assert!(repo.review_queue_for(&request.id).await.unwrap().is_empty());
}

#[tokio::test]
async fn search_worker_queues_review_candidates_below_auto_threshold() {
    let pool = connect_sqlite(&DatabaseTarget::memory()).await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    let settings = SqliteSettingsRepository::new(pool.clone());
    settings
        .ensure_seeded(&book_router::config::AppConfig::for_tests())
        .await
        .unwrap();

    let prowlarr_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/search"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"[
                {
                    "guid":"candidate-review",
                    "title":"The Hobbit J.R.R. Tolkien narrated by Andy Serkis [ENG]",
                    "size":1234,
                    "protocol":"torrent",
                    "indexer":"Books",
                    "downloadUrl":"magnet:?xt=urn:btih:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb&dn=The+Hobbit"
                }
            ]"#,
            "application/json",
        ))
        .mount(&prowlarr_server)
        .await;

    settings
        .update_runtime_settings(RuntimeSettingsUpdate {
            integrations: Some(IntegrationSettingsUpdate {
                prowlarr: Some(ProwlarrIntegrationUpdate {
                    enabled: Some(true),
                    base_url: Some(prowlarr_server.uri()),
                    api_key: Some("prowlarr-key".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        })
        .await
        .unwrap();

    let repo = SqliteRequestRepository::new(pool.clone());
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

    SearchWorker::process_request_by_id(pool.clone(), settings.clone(), &request.id)
        .await
        .unwrap();

    let updated = repo.find_by_id(&request.id).await.unwrap().unwrap();
    let review_queue = repo.review_queue_for(&request.id).await.unwrap();

    assert_eq!(updated.state, "review");
    assert_eq!(review_queue.len(), 1);
    assert_eq!(
        review_queue[0].candidate.download_url.as_deref(),
        Some("magnet:?xt=urn:btih:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb&dn=The+Hobbit")
    );
    assert_eq!(
        review_queue[0].candidate.narrator.as_deref(),
        Some("Andy Serkis")
    );
    assert_eq!(
        review_queue[0].candidate.detected_language.as_deref(),
        Some("en")
    );
}
