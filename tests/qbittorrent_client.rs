use book_router::downloads::qbittorrent::QbittorrentClient;
use wiremock::matchers::{body_string_contains, header_regex, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn authenticates_and_submits_magnet_with_request_tag_and_category() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v2/auth/login"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("set-cookie", "SID=test-session; HttpOnly; Path=/"),
        )
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/v2/torrents/add"))
        .and(header_regex("cookie", "SID=test-session"))
        .and(body_string_contains("urls=magnet%3A%3Fxt%3Durn%3Abtih%3A"))
        .and(body_string_contains("category=audiobooks"))
        .and(body_string_contains("tags=request-123"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = QbittorrentClient::new(server.uri(), "admin", "adminadmin");
    client
        .add_magnet("magnet:?xt=urn:btih:test", "request-123", "audiobooks")
        .await
        .unwrap();
}

#[tokio::test]
async fn filters_completed_torrents_by_category_and_request_tag() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v2/auth/login"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("set-cookie", "SID=test-session; HttpOnly; Path=/"),
        )
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v2/torrents/info"))
        .and(query_param("category", "athena-ebooks"))
        .and(header_regex("cookie", "SID=test-session"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"[
                {
                    "hash":"0123456789abcdef0123456789abcdef01234567",
                    "name":"The Hobbit",
                    "category":"athena-ebooks",
                    "tags":"request-123,ebook",
                    "progress":1.0,
                    "state":"uploading",
                    "content_path":"/downloads/The Hobbit.epub"
                },
                {
                    "hash":"1111111111111111111111111111111111111111",
                    "name":"Other Book",
                    "category":"athena-ebooks",
                    "tags":"request-456",
                    "progress":1.0,
                    "state":"uploading",
                    "content_path":"/downloads/Other Book.epub"
                },
                {
                    "hash":"2222222222222222222222222222222222222222",
                    "name":"The Hobbit Incomplete",
                    "category":"athena-ebooks",
                    "tags":"request-123",
                    "progress":0.45,
                    "state":"downloading",
                    "content_path":"/downloads/The Hobbit Incomplete.epub"
                }
            ]"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let client = QbittorrentClient::new(server.uri(), "admin", "adminadmin");
    let completed = client
        .completed_for_tag("request-123", "athena-ebooks")
        .await
        .unwrap();

    assert_eq!(completed.len(), 1);
    assert_eq!(
        completed[0].hash,
        "0123456789abcdef0123456789abcdef01234567"
    );
}
