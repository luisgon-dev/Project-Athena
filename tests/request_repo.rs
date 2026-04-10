use book_router::{
    config::DatabaseTarget,
    db::{connect_sqlite, repositories::SqliteRequestRepository},
    domain::requests::{CreateRequest, ManifestationPreference, MediaType},
};
use serde_json::Value;
use sqlx::Row;

#[tokio::test]
async fn create_request_persists_canonical_work_identity_and_initial_event() {
    let pool = connect_sqlite(&DatabaseTarget::memory()).await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    let repo = SqliteRequestRepository::new(pool.clone());

    let request = repo
        .create(CreateRequest {
            external_work_id: "OL27448W".into(),
            title: "The Hobbit".into(),
            author: "J.R.R. Tolkien".into(),
            media_type: MediaType::Audiobook,
            preferred_language: Some("en".into()),
            manifestation: ManifestationPreference::default(),
        })
        .await
        .unwrap();

    let request_id = request.id.clone();
    let events = repo.events_for(&request_id).await.unwrap();
    let event_payload: Value = serde_json::from_str(&events[0].payload_json).unwrap();

    assert_eq!(request.title, "The Hobbit");
    assert_eq!(request.author, "J.R.R. Tolkien");
    assert_eq!(request.external_work_id, "OL27448W");
    assert_eq!(request.media_type, MediaType::Audiobook);
    assert_eq!(request.preferred_language.as_deref(), Some("en"));
    assert_eq!(request.state, "requested");
    assert!(!request.created_at.is_empty());
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind.as_str(), "request.created");
    assert_eq!(events[0].request_id, request_id);
    assert_eq!(event_payload["request_id"], request_id);
    assert_eq!(event_payload["external_work_id"], "OL27448W");
}

#[tokio::test]
async fn create_batch_rolls_back_all_requests_if_any_request_is_invalid() {
    let pool = connect_sqlite(&DatabaseTarget::memory()).await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    let repo = SqliteRequestRepository::new(pool.clone());

    let error = repo
        .create_batch(vec![
            CreateRequest {
                external_work_id: "OL27448W".into(),
                title: "The Hobbit".into(),
                author: "J.R.R. Tolkien".into(),
                media_type: MediaType::Ebook,
                preferred_language: Some("en".into()),
                manifestation: ManifestationPreference::default(),
            },
            CreateRequest {
                external_work_id: String::new(),
                title: "The Hobbit".into(),
                author: "J.R.R. Tolkien".into(),
                media_type: MediaType::Audiobook,
                preferred_language: Some("en".into()),
                manifestation: ManifestationPreference::default(),
            },
        ])
        .await
        .unwrap_err();

    assert!(error.to_string().contains("external work id"));

    let stored_request_count = sqlx::query("SELECT COUNT(*) AS count FROM requests")
        .fetch_one(&pool)
        .await
        .unwrap()
        .get::<i64, _>("count");
    let stored_event_count = sqlx::query("SELECT COUNT(*) AS count FROM request_events")
        .fetch_one(&pool)
        .await
        .unwrap()
        .get::<i64, _>("count");

    assert_eq!(stored_request_count, 0);
    assert_eq!(stored_event_count, 0);
}

#[tokio::test]
async fn connect_sqlite_creates_parent_directory_for_file_databases() {
    let tempdir = tempfile::tempdir().unwrap();
    let database_path = tempdir.path().join("nested/state/book-router.sqlite");

    let pool = connect_sqlite(&DatabaseTarget::file(&database_path))
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    assert!(database_path.exists());
    assert!(database_path.parent().unwrap().exists());
}
