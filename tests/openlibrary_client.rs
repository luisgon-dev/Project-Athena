use book_router::metadata::openlibrary::OpenLibraryClient;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn resolves_work_from_open_library() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/search.json"))
        .and(query_param("title", "The Hobbit"))
        .and(query_param("author", "J.R.R. Tolkien"))
        .and(query_param("limit", "5"))
        .and(header("user-agent", "book-router/0.1"))
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

    assert_eq!(result.work.external_id, "OL27448W");
    assert_eq!(result.work.title, "The Hobbit");
    assert_eq!(result.work.primary_author, "J.R.R. Tolkien");
}

#[tokio::test]
async fn skips_a_mismatched_first_hit() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/search.json"))
        .and(query_param("title", "The Hobbit"))
        .and(query_param("author", "J.R.R. Tolkien"))
        .and(query_param("limit", "5"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{
            "docs":[
                {"key":"OL00000W","title":"Definitely Not The Hobbit","author_name":["Someone Else"]},
                {"key":"OL27448W","title":"The Hobbit","author_name":["J.R.R. Tolkien"]}
            ]
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

    assert_eq!(result.work.external_id, "OL27448W");
    assert_eq!(result.work.title, "The Hobbit");
    assert_eq!(result.work.primary_author, "J.R.R. Tolkien");
}
