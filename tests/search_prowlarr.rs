use book_router::search::prowlarr::ProwlarrClient;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn normalizes_prowlarr_results_into_release_candidates() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/search"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"[
                {"guid":"abc","title":"The Hobbit Andy Serkis M4B","size":1234,"protocol":"torrent","indexer":"Books"}
            ]"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let client = ProwlarrClient::new(server.uri(), "test-api-key");
    let results = client.search("The Hobbit", "audio", &[]).await.unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].source, "prowlarr");
    assert_eq!(results[0].title, "The Hobbit Andy Serkis M4B");
    assert_eq!(results[0].narrator, None);
}

#[tokio::test]
async fn tolerates_partial_or_loose_prowlarr_items() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/search"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"[
                {"guid":"abc","title":"Red Rising by Pierce Brown [ENG / EPUB]","size":"1234","protocol":"torrent","indexer":"Books","downloadUrl":false},
                {"infoUrl":"https://example.invalid/fallback","title":"A Court of Thorns and Roses","size":5678,"protocol":null,"indexer":null}
            ]"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let client = ProwlarrClient::new(server.uri(), "test-api-key");
    let results = client.search("test", "book", &[]).await.unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].external_id, "abc");
    assert_eq!(results[0].size_bytes, 1234);
    assert_eq!(results[0].download_url, None);
    assert_eq!(results[1].external_id, "https://example.invalid/fallback");
    assert_eq!(results[1].protocol, "torrent");
    assert_eq!(results[1].indexer, "Prowlarr");
}

#[tokio::test]
async fn parses_candidate_metadata_from_release_titles() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/search"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"[
                {"guid":"abc","title":"The Sandman narrated by Neil Gaiman [ENG] GraphicAudio","size":1234,"protocol":"torrent","indexer":"Books"}
            ]"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let client = ProwlarrClient::new(server.uri(), "test-api-key");
    let results = client.search("The Sandman", "audio", &[]).await.unwrap();

    assert_eq!(results[0].narrator.as_deref(), Some("Neil Gaiman"));
    assert!(results[0].graphic_audio);
    assert_eq!(results[0].detected_language.as_deref(), Some("en"));
}

#[tokio::test]
async fn passes_selected_indexer_ids_to_prowlarr_search() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/search"))
        .and(query_param("indexerIds", "12"))
        .and(query_param("indexerIds", "42"))
        .respond_with(ResponseTemplate::new(200).set_body_raw("[]", "application/json"))
        .mount(&server)
        .await;

    let client = ProwlarrClient::new(server.uri(), "test-api-key");
    let results = client
        .search("The Hobbit", "book", &[12, 42])
        .await
        .unwrap();

    assert!(results.is_empty());
}
