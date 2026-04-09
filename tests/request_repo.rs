use book_router::{
    db::{connect_sqlite, repositories::SqliteRequestRepository},
    domain::requests::{CreateRequest, MediaType},
};

#[tokio::test]
async fn create_request_persists_initial_event() {
    let pool = connect_sqlite("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    let repo = SqliteRequestRepository::new(pool);

    let request = repo
        .create(CreateRequest {
            title: "The Hobbit".into(),
            author: "J.R.R. Tolkien".into(),
            media_type: MediaType::Audiobook,
            preferred_language: Some("en".into()),
        })
        .await
        .unwrap();

    let events = repo.events_for(request.id).await.unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind.as_str(), "request.created");
}
