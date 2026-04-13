use axum::{
    body::Body,
    http::{Request, StatusCode, header as http_header},
};
use book_router::{app::build_app, config::AppConfig};
use serde_json::{Value, json};
use tower::util::ServiceExt;

const API_PREFIX: &str = "/api/v1";

#[tokio::test]
async fn runtime_settings_are_seeded_from_bootstrap_config() {
    let config = AppConfig::for_tests()
        .with_metadata_base_url("https://metadata.seed.test")
        .with_cover_base_url("https://covers.seed.test");
    let app = build_app(config).await.unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("{API_PREFIX}/settings/runtime"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;

    assert_eq!(body["storage"]["ebooks_root"], "/ebooks");
    assert_eq!(body["storage"]["audiobooks_root"], "/audiobooks");
    assert_eq!(body["metadata"]["base_url"], "https://metadata.seed.test");
    assert_eq!(
        body["metadata"]["cover_base_url"],
        "https://covers.seed.test"
    );
    assert_eq!(body["download_clients"]["qbittorrent"]["enabled"], false);
    assert_eq!(
        body["download_clients"]["qbittorrent"]["has_password"],
        false
    );
}

#[tokio::test]
async fn qbittorrent_settings_mask_password_after_save() {
    let app = build_app(AppConfig::for_tests()).await.unwrap();

    let response = app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("{API_PREFIX}/settings/download-clients/qbittorrent"),
            json!({
                "enabled": true,
                "base_url": "http://localhost:8080",
                "username": "admin",
                "password": "secret",
                "category_ebook": "athena-ebooks",
                "category_audiobook": "athena-audiobooks"
            }),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["enabled"], true);
    assert_eq!(body["has_password"], true);
    assert!(body.get("password").is_none());

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("{API_PREFIX}/settings/runtime"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = json_body(response).await;
    assert_eq!(
        body["download_clients"]["qbittorrent"]["has_password"],
        true
    );
    assert!(
        body["download_clients"]["qbittorrent"]
            .get("password")
            .is_none()
    );
}

#[tokio::test]
async fn readarr_compatible_indexer_routes_store_mask_and_delete_synced_indexers() {
    let app = build_app(AppConfig::for_tests()).await.unwrap();

    let payload = json!({
        "id": 0,
        "name": "Books (Prowlarr)",
        "enableRss": true,
        "enableAutomaticSearch": true,
        "enableInteractiveSearch": true,
        "priority": 25,
        "implementationName": "Torznab",
        "implementation": "Torznab",
        "configContract": "TorznabSettings",
        "fields": [
            {"name":"baseUrl","value":"http://prowlarr.local:9696/12/","type":"textbox","advanced":false,"section":"connectivity","hidden":"never"},
            {"name":"apiPath","value":"/api","type":"textbox","advanced":false,"section":"connectivity","hidden":"never"},
            {"name":"apiKey","value":"super-secret","type":"password","advanced":false,"section":"connectivity","hidden":"never"},
            {"name":"categories","value":[3030, 7000],"type":"tag","advanced":false,"section":"indexer","hidden":"never"},
            {"name":"minimumSeeders","value":1,"type":"number","advanced":false,"section":"torrent","hidden":"never"}
        ]
    });

    let create = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("{API_PREFIX}/indexer"),
            payload.clone(),
        ))
        .await
        .unwrap();

    assert_eq!(create.status(), StatusCode::CREATED);
    let created = json_body(create).await;
    assert_eq!(created["id"], 1);
    assert_eq!(field_value(&created, "apiKey"), Some("********"));

    let list = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("{API_PREFIX}/indexer"))
                .header("X-Api-Key", "ignored")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let listed = json_body(list).await;
    assert_eq!(listed.as_array().unwrap().len(), 1);
    assert_eq!(field_value(&listed[0], "apiKey"), Some("********"));

    let synced = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("{API_PREFIX}/settings/synced-indexers"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let synced_body = json_body(synced).await;
    assert_eq!(synced_body[0]["prowlarr_indexer_id"], 12);
    assert_eq!(synced_body[0]["categories"], json!([3030, 7000]));

    let update = app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("{API_PREFIX}/indexer/1?forceSave=true"),
            json!({
                "id": 1,
                "name": "Books Updated (Prowlarr)",
                "enableRss": false,
                "enableAutomaticSearch": true,
                "enableInteractiveSearch": true,
                "priority": 10,
                "implementationName": "Torznab",
                "implementation": "Torznab",
                "configContract": "TorznabSettings",
                "fields": payload["fields"]
            }),
        ))
        .await
        .unwrap();
    let updated = json_body(update).await;
    assert_eq!(updated["name"], "Books Updated (Prowlarr)");
    assert_eq!(field_value(&updated, "apiKey"), Some("********"));

    let delete = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("{API_PREFIX}/indexer/1"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(delete.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn system_status_reports_a_version_for_prowlarr_handshake() {
    let app = build_app(AppConfig::for_tests()).await.unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("{API_PREFIX}/system/status"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["version"], env!("CARGO_PKG_VERSION"));
}

fn json_request(method: &str, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(http_header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn field_value<'a>(payload: &'a Value, name: &str) -> Option<&'a str> {
    payload
        .get("fields")
        .and_then(Value::as_array)
        .and_then(|fields| {
            fields
                .iter()
                .find(|field| field.get("name").and_then(Value::as_str) == Some(name))
        })
        .and_then(|field| field.get("value"))
        .and_then(Value::as_str)
}

async fn text_body(response: axum::response::Response) -> String {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    String::from_utf8(body.to_vec()).unwrap()
}

async fn json_body(response: axum::response::Response) -> Value {
    serde_json::from_str(&text_body(response).await).unwrap()
}
