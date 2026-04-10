use book_router::metadata::openlibrary::{OpenLibraryClient, OpenLibraryError};
use std::time::Duration;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn client_handles_timeout() {
    let server = MockServer::start().await;

    // Simulate a delay of 11 seconds, which is more than the 10s timeout
    Mock::given(method("GET"))
        .and(path("/search.json"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(11)))
        .mount(&server)
        .await;

    let client = OpenLibraryClient::new(server.uri(), server.uri());
    let result = client.resolve_work("The Hobbit", "J.R.R. Tolkien").await;

    match result {
        Err(OpenLibraryError::Timeout(_)) => {}
        other => panic!("Expected timeout error, got {:?}", other),
    }
}
