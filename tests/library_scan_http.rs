use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use book_router::{app::build_app, config::AppConfig};
use serde_json::{Value, json};
use tower::util::ServiceExt;

const API_PREFIX: &str = "/api/v1";

#[tokio::test]
async fn library_scan_triggers_and_returns_status() {
    let tmp = tempfile::tempdir().unwrap();
    let ebooks = tmp.path().join("ebooks");
    let audiobooks = tmp.path().join("audiobooks");

    std::fs::create_dir_all(ebooks.join("Arthur C. Clarke").join("2001")).unwrap();
    std::fs::write(
        ebooks
            .join("Arthur C. Clarke")
            .join("2001")
            .join("2001.epub"),
        "",
    )
    .unwrap();

    std::fs::create_dir_all(audiobooks.join("Philip K. Dick").join("Do Androids Dream")).unwrap();
    std::fs::write(
        audiobooks
            .join("Philip K. Dick")
            .join("Do Androids Dream")
            .join("part1.m4b"),
        "",
    )
    .unwrap();

    let config = AppConfig::for_tests()
        .with_metadata_base_url("https://metadata.seed.test")
        .with_cover_base_url("https://covers.seed.test");
    // We can't easily override the ebook/audiobook roots in AppConfig for_tests,
    // so we'll update settings via the API first.
    let app = build_app(config).await.unwrap();

    // Update storage settings to point at our temp dirs
    let update = app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("{API_PREFIX}/settings/storage"),
            json!({
                "ebooks_root": ebooks.display().to_string(),
                "audiobooks_root": audiobooks.display().to_string()
            }),
        ))
        .await
        .unwrap();
    assert_eq!(update.status(), StatusCode::OK);

    // Trigger scan
    let scan = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("{API_PREFIX}/library/scan"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(scan.status(), StatusCode::ACCEPTED);
    let scan_body: Value = json_body(scan).await;
    assert!(scan_body.get("job_id").is_some());

    // Poll status until completion (with timeout)
    let mut completed = false;
    for _ in 0..20 {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        let status = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("{API_PREFIX}/library/scan-status"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(status.status(), StatusCode::OK);
        let status_body: Value = json_body(status).await;
        if status_body
            .get("completed_at")
            .and_then(Value::as_str)
            .is_some()
        {
            assert_eq!(status_body["ebooks_found"], 1);
            assert_eq!(status_body["audiobooks_found"], 1);
            assert_eq!(status_body["duplicates_skipped"], 0);
            completed = true;
            break;
        }
    }
    assert!(completed, "scan did not complete in time");

    // Re-scan should report duplicates
    let scan2 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("{API_PREFIX}/library/scan"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(scan2.status(), StatusCode::ACCEPTED);

    let mut duplicates_reported = false;
    for _ in 0..20 {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        let status = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("{API_PREFIX}/library/scan-status"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let status_body: Value = json_body(status).await;
        if status_body
            .get("completed_at")
            .and_then(Value::as_str)
            .is_some()
        {
            if status_body["duplicates_skipped"] == 2 {
                duplicates_reported = true;
                break;
            }
        }
    }
    assert!(duplicates_reported, "second scan did not report duplicates");
}

fn json_request(method: &str, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(axum::http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
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
