use book_router::{
    config::DatabaseTarget,
    db::{connect_sqlite, repositories::SqliteRequestRepository},
    domain::requests::{CreateRequest, ManifestationPreference, MediaType},
};
use serde_json::Value;

#[tokio::test]
async fn create_request_persists_initial_event() {
    let pool = connect_sqlite(&DatabaseTarget::memory()).await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    let repo = SqliteRequestRepository::new(pool);

    let request = repo
        .create(CreateRequest {
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
    assert_eq!(request.media_type, MediaType::Audiobook);
    assert_eq!(request.preferred_language.as_deref(), Some("en"));
    assert_eq!(request.state, "requested");
    assert!(!request.created_at.is_empty());
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind.as_str(), "request.created");
    assert_eq!(events[0].request_id, request_id);
    assert_eq!(event_payload["request_id"], request_id);
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
