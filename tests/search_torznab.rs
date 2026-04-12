use book_router::search::torznab::TorznabClient;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn parses_torznab_rss_items_into_release_candidates() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"
                <rss><channel><item>
                    <guid>torznab-1</guid>
                    <title>The Hobbit EPUB</title>
                    <size>2048</size>
                </item></channel></rss>
            "#,
            "application/xml",
        ))
        .mount(&server)
        .await;

    let client = TorznabClient::new(server.uri(), None);
    let items = client.search("The Hobbit").await.unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].source, "torznab");
    assert_eq!(items[0].title, "The Hobbit EPUB");
}
