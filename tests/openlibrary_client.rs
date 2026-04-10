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

    let client = OpenLibraryClient::new(server.uri(), server.uri());
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

    let client = OpenLibraryClient::new(server.uri(), server.uri());
    let result = client
        .resolve_work("The Hobbit", "J.R.R. Tolkien")
        .await
        .unwrap();

    assert_eq!(result.work.external_id, "OL27448W");
    assert_eq!(result.work.title, "The Hobbit");
    assert_eq!(result.work.primary_author, "J.R.R. Tolkien");
}

#[tokio::test]
async fn accepts_a_match_anywhere_in_author_name_list() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/search.json"))
        .and(query_param("title", "The Hobbit"))
        .and(query_param("author", "J.R.R. Tolkien"))
        .and(query_param("limit", "5"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{
            "docs":[{"key":"OL27448W","title":"The Hobbit","author_name":["Someone Else","J.R.R. Tolkien"]}]
        }"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let client = OpenLibraryClient::new(server.uri(), server.uri());
    let result = client
        .resolve_work("The Hobbit", "J.R.R. Tolkien")
        .await
        .unwrap();

    assert_eq!(result.work.external_id, "OL27448W");
    assert_eq!(result.work.title, "The Hobbit");
    assert_eq!(result.work.primary_author, "Someone Else");
}

#[tokio::test]
async fn resolves_work_by_canonical_id() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/works/OL27448W.json"))
        .and(header("user-agent", "book-router/0.1"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{
            "key":"/works/OL27448W",
            "title":"The Hobbit",
            "authors":[{"author":{"key":"/authors/OL26320A"}}]
        }"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/authors/OL26320A.json"))
        .and(header("user-agent", "book-router/0.1"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{
            "key":"/authors/OL26320A",
            "name":"J.R.R. Tolkien"
        }"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let client = OpenLibraryClient::new(server.uri(), server.uri());
    let result = client
        .resolve_work_by_id("OL27448W")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(result.work.external_id, "OL27448W");
    assert_eq!(result.work.title, "The Hobbit");
    assert_eq!(result.work.primary_author, "J.R.R. Tolkien");
}

#[tokio::test]
async fn search_works_hydrates_top_matches_with_detail_metadata() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/search.json"))
        .and(query_param("title", "Dune"))
        .and(query_param("author", "Frank Herbert"))
        .and(query_param("limit", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{
                "docs":[
                    {
                        "key":"/works/OL123W",
                        "title":"Dune",
                        "author_name":["Frank Herbert"],
                        "first_publish_year":1965,
                        "cover_i":9981,
                        "subject":["Science fiction","Arrakis","Epic"],
                        "edition_count":87
                    }
                ]
            }"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/works/OL123W.json"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{
                "key":"/works/OL123W",
                "title":"Dune",
                "description":{"value":"Paul Atreides must survive the desert planet."},
                "subjects":["Science fiction","Politics","Arrakis"],
                "covers":[9981]
            }"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let client = OpenLibraryClient::new(server.uri(), server.uri());
    let result = client.search_works("Dune", "Frank Herbert").await.unwrap();

    assert_eq!(result.works.len(), 1);
    let work = &result.works[0];
    assert_eq!(work.external_id, "OL123W");
    assert_eq!(work.title, "Dune");
    assert_eq!(work.primary_author, "Frank Herbert");
    assert_eq!(work.first_publish_year, Some(1965));
    assert_eq!(
        work.description.as_deref(),
        Some("Paul Atreides must survive the desert planet.")
    );
    assert_eq!(work.cover_id, Some(9981));
    assert_eq!(
        work.subjects,
        vec![
            "Science fiction".to_string(),
            "Arrakis".to_string(),
            "Epic".to_string()
        ]
    );
    assert_eq!(work.edition_count, Some(87));
}

#[tokio::test]
async fn search_works_keeps_search_results_when_detail_hydration_fails() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/search.json"))
        .and(query_param("title", "Dune"))
        .and(query_param("author", ""))
        .and(query_param("limit", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{
                "docs":[
                    {
                        "key":"/works/OL123W",
                        "title":"Dune",
                        "author_name":["Frank Herbert"],
                        "first_publish_year":1965,
                        "cover_i":9981,
                        "subject":["Science fiction","Arrakis"],
                        "edition_count":87
                    }
                ]
            }"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/works/OL123W.json"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let client = OpenLibraryClient::new(server.uri(), server.uri());
    let result = client.search_works("Dune", "").await.unwrap();

    assert_eq!(result.works.len(), 1);
    let work = &result.works[0];
    assert_eq!(work.external_id, "OL123W");
    assert_eq!(work.title, "Dune");
    assert_eq!(work.primary_author, "Frank Herbert");
    assert_eq!(work.first_publish_year, Some(1965));
    assert_eq!(work.description, None);
    assert_eq!(work.cover_id, Some(9981));
    assert_eq!(
        work.subjects,
        vec!["Science fiction".to_string(), "Arrakis".to_string()]
    );
    assert_eq!(work.edition_count, Some(87));
}
