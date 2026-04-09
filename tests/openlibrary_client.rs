use book_router::metadata::openlibrary::OpenLibraryClient;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn resolves_work_and_primary_manifestation_from_open_library() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/search.json"))
        .and(query_param("title", "The Hobbit"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{
            "docs":[{"key":"OL27448W","title":"The Hobbit","author_name":["J.R.R. Tolkien"],"language":["eng"]}]
        }"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let client = OpenLibraryClient::new(server.uri());
    let result = client
        .resolve_work("The Hobbit", "J.R.R. Tolkien")
        .await
        .unwrap();

    assert_eq!(result.work.title, "The Hobbit");
    assert_eq!(result.work.primary_author, "J.R.R. Tolkien");
}
