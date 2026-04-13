use std::{fs, path::Path};

use book_router::{
    config::DatabaseTarget,
    db::{connect_sqlite, repositories::SqliteRequestRepository},
    domain::{
        requests::{CreateRequest, ManifestationPreference, MediaType},
        search::ReleaseCandidate,
    },
    downloads::qbittorrent::QbittorrentClient,
    sync::{audiobookshelf::AudiobookshelfClient, calibre::CalibreHook},
    workers::{
        download_worker::DownloadWorker, import_worker::ImportWorker, sync_worker::SyncWorker,
    },
};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use wiremock::matchers::{body_string_contains, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn request_to_import_flow_records_success_events() {
    let pool = connect_sqlite(&DatabaseTarget::memory()).await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    let repo = SqliteRequestRepository::new(pool);
    let tempdir = tempfile::tempdir().unwrap();
    let downloads_root = tempdir.path().join("downloads");
    let ebooks_root = tempdir.path().join("ebooks");
    fs::create_dir_all(&downloads_root).unwrap();
    fs::create_dir_all(&ebooks_root).unwrap();

    let request = repo
        .create(CreateRequest {
            external_work_id: "OL27448W".into(),
            title: "The Hobbit".into(),
            author: "J.R.R. Tolkien".into(),
            media_type: MediaType::Ebook,
            preferred_language: Some("en".into()),
            manifestation: ManifestationPreference::default(),
        })
        .await
        .unwrap();

    let candidate = ReleaseCandidate::for_tests("The Hobbit EPUB");
    let source_file = downloads_root.join("The Hobbit.epub");
    fs::write(&source_file, b"ebook payload").unwrap();

    DownloadWorker::record_approved_candidate(&repo, &request.id, &candidate)
        .await
        .unwrap();
    ImportWorker::import_completed_ebook(
        &repo,
        &request.id,
        &[source_file.display().to_string()],
        &ebooks_root,
    )
    .await
    .unwrap();

    let updated = repo.find_by_id(&request.id).await.unwrap().unwrap();
    let events = repo.events_for(&request.id).await.unwrap();

    assert_eq!(updated.state, "imported");
    assert!(
        events
            .iter()
            .any(|event| event.kind.as_str() == "download.queued")
    );
    assert!(
        events
            .iter()
            .any(|event| event.kind.as_str() == "download.completed")
    );
    assert!(
        events
            .iter()
            .any(|event| event.kind.as_str() == "import.succeeded")
    );
}

#[tokio::test]
async fn ebook_request_moves_file_and_runs_calibre_hook() {
    let pool = connect_sqlite(&DatabaseTarget::memory()).await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    let repo = SqliteRequestRepository::new(pool);
    let tempdir = tempfile::tempdir().unwrap();
    let downloads_root = tempdir.path().join("downloads");
    let ebooks_root = tempdir.path().join("ebooks");
    fs::create_dir_all(&downloads_root).unwrap();
    fs::create_dir_all(&ebooks_root).unwrap();

    let request = repo
        .create(CreateRequest {
            external_work_id: "OL27448W".into(),
            title: "The Hobbit: There and Back Again".into(),
            author: "J.R.R. Tolkien".into(),
            media_type: MediaType::Ebook,
            preferred_language: Some("en".into()),
            manifestation: ManifestationPreference::default(),
        })
        .await
        .unwrap();

    let candidate = ReleaseCandidate::for_tests("The Hobbit EPUB");
    DownloadWorker::record_approved_candidate(&repo, &request.id, &candidate)
        .await
        .unwrap();

    let source_file = downloads_root.join("The.Hobbit.50th.Anniversary.Release.epub");
    fs::write(&source_file, b"ebook payload").unwrap();

    let moved_path = ImportWorker::import_completed_ebook(
        &repo,
        &request.id,
        &[source_file.display().to_string()],
        &ebooks_root,
    )
    .await
    .unwrap();

    assert_eq!(
        moved_path,
        ebooks_root
            .join("J.R.R. Tolkien")
            .join("The Hobbit There and Back Again")
            .join("The Hobbit There and Back Again.epub")
    );
    assert!(moved_path.exists());
    assert!(!source_file.exists());

    let script_path = tempdir.path().join("fake-calibredb.sh");
    let log_path = tempdir.path().join("calibre.log");
    write_fake_calibre(&script_path, &log_path);

    let hook = CalibreHook::new(&script_path);
    SyncWorker::sync_ebook(&repo, &request.id, &moved_path, &hook)
        .await
        .unwrap();

    let updated = repo.find_by_id(&request.id).await.unwrap().unwrap();
    let events = repo.events_for(&request.id).await.unwrap();
    let log = fs::read_to_string(&log_path).unwrap();

    assert_eq!(updated.state, "synced");
    assert!(
        events
            .iter()
            .any(|event| event.kind.as_str() == "sync.succeeded")
    );
    assert!(log.contains(moved_path.to_string_lossy().as_ref()));
}

fn write_fake_calibre(script_path: &Path, log_path: &Path) {
    let script = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"{}\"\n",
        escape_shell_path(log_path)
    );
    fs::write(script_path, script).unwrap();
    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(script_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(script_path, permissions).unwrap();
    }
}

fn escape_shell_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
}

#[tokio::test]
async fn ebook_request_dispatches_to_qbittorrent_and_completes_via_polling() {
    let pool = connect_sqlite(&DatabaseTarget::memory()).await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    let repo = SqliteRequestRepository::new(pool);
    let tempdir = tempfile::tempdir().unwrap();
    let downloads_root = tempdir.path().join("downloads");
    let ebooks_root = tempdir.path().join("ebooks");
    fs::create_dir_all(&downloads_root).unwrap();
    fs::create_dir_all(&ebooks_root).unwrap();

    let request = repo
        .create(CreateRequest {
            external_work_id: "OL27448W".into(),
            title: "The Hobbit".into(),
            author: "J.R.R. Tolkien".into(),
            media_type: MediaType::Ebook,
            preferred_language: Some("en".into()),
            manifestation: ManifestationPreference::default(),
        })
        .await
        .unwrap();

    let source_file = downloads_root.join("The Hobbit.epub");
    fs::write(&source_file, b"ebook payload").unwrap();

    let server = MockServer::start().await;
    let expected_tag = format!("tags={}", request.id);
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
        .and(body_string_contains(
            "urls=magnet%3A%3Fxt%3Durn%3Abtih%3A0123456789abcdef0123456789abcdef01234567",
        ))
        .and(body_string_contains("category=athena-ebooks"))
        .and(body_string_contains(expected_tag))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v2/torrents/info"))
        .and(query_param("category", "athena-ebooks"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            format!(
                r#"[
                    {{
                        "hash":"0123456789abcdef0123456789abcdef01234567",
                        "name":"The Hobbit",
                        "category":"athena-ebooks",
                        "tags":"{request_id}",
                        "progress":1.0,
                        "state":"uploading",
                        "content_path":"{content_path}"
                    }}
                ]"#,
                request_id = request.id,
                content_path = source_file.display()
            ),
            "application/json",
        ))
        .mount(&server)
        .await;

    let candidate = ReleaseCandidate {
        external_id: "candidate-1".into(),
        source: "prowlarr".into(),
        title: "The Hobbit EPUB".into(),
        protocol: "torrent".into(),
        size_bytes: 1234,
        indexer: "Books".into(),
        download_url: Some(
            "magnet:?xt=urn:btih:0123456789abcdef0123456789abcdef01234567&dn=The+Hobbit".into(),
        ),
    };

    let client = QbittorrentClient::new(server.uri(), "admin", "adminadmin");
    DownloadWorker::dispatch_approved_candidate(&repo, &client, &request.id, &candidate)
        .await
        .unwrap();

    let script_path = tempdir.path().join("fake-calibredb.sh");
    let log_path = tempdir.path().join("calibre.log");
    write_fake_calibre(&script_path, &log_path);
    let hook = CalibreHook::new(&script_path);

    let processed = DownloadWorker::poll_qbittorrent_once(
        &repo,
        &client,
        &ebooks_root,
        &tempdir.path().join("audiobooks"),
        &hook,
        None,
        None,
    )
    .await
    .unwrap();

    assert_eq!(processed, 1);

    let updated = repo.find_by_id(&request.id).await.unwrap().unwrap();
    let events = repo.events_for(&request.id).await.unwrap();
    let log = fs::read_to_string(&log_path).unwrap();
    let final_path = ebooks_root
        .join("J.R.R. Tolkien")
        .join("The Hobbit")
        .join("The Hobbit.epub");

    assert_eq!(updated.state, "synced");
    assert!(final_path.exists());
    assert!(!source_file.exists());
    assert!(
        events
            .iter()
            .any(|event| event.kind.as_str() == "download.queued")
    );
    assert!(
        events
            .iter()
            .any(|event| event.kind.as_str() == "download.completed")
    );
    assert!(
        events
            .iter()
            .any(|event| event.kind.as_str() == "import.succeeded")
    );
    assert!(
        events
            .iter()
            .any(|event| event.kind.as_str() == "sync.succeeded")
    );
    assert!(log.contains(final_path.to_string_lossy().as_ref()));
}

#[tokio::test]
async fn audiobook_request_dispatches_imports_and_syncs_with_audiobookshelf() {
    let pool = connect_sqlite(&DatabaseTarget::memory()).await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    let repo = SqliteRequestRepository::new(pool);
    let tempdir = tempfile::tempdir().unwrap();
    let downloads_root = tempdir.path().join("downloads");
    let ebooks_root = tempdir.path().join("ebooks");
    let audiobooks_root = tempdir.path().join("audiobooks");
    fs::create_dir_all(&downloads_root).unwrap();
    fs::create_dir_all(&ebooks_root).unwrap();
    fs::create_dir_all(&audiobooks_root).unwrap();

    let request = repo
        .create(CreateRequest {
            external_work_id: "OL27448W".into(),
            title: "The Hobbit".into(),
            author: "J.R.R. Tolkien".into(),
            media_type: MediaType::Audiobook,
            preferred_language: Some("en".into()),
            manifestation: ManifestationPreference::default(),
        })
        .await
        .unwrap();

    let source_file = downloads_root.join("The Hobbit.m4b");
    fs::write(&source_file, b"audiobook payload").unwrap();

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
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v2/torrents/info"))
        .and(query_param("category", "athena-audiobooks"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            format!(
                r#"[
                    {{
                        "hash":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                        "name":"The Hobbit",
                        "category":"athena-audiobooks",
                        "tags":"{request_id}",
                        "progress":1.0,
                        "state":"uploading",
                        "content_path":"{content_path}"
                    }}
                ]"#,
                request_id = request.id,
                content_path = source_file.display()
            ),
            "application/json",
        ))
        .mount(&server)
        .await;

    let abs_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/libraries/library-1/scan"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&abs_server)
        .await;

    let candidate = ReleaseCandidate {
        external_id: "candidate-audio".into(),
        source: "prowlarr".into(),
        title: "The Hobbit M4B".into(),
        protocol: "torrent".into(),
        size_bytes: 1234,
        indexer: "Books".into(),
        download_url: Some(
            "magnet:?xt=urn:btih:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa&dn=The+Hobbit".into(),
        ),
    };
    let client = QbittorrentClient::new(server.uri(), "admin", "adminadmin");
    DownloadWorker::dispatch_approved_candidate(&repo, &client, &request.id, &candidate)
        .await
        .unwrap();

    let processed = DownloadWorker::poll_qbittorrent_once(
        &repo,
        &client,
        &ebooks_root,
        &audiobooks_root,
        &CalibreHook::new("/usr/bin/false"),
        Some(&AudiobookshelfClient::new(abs_server.uri(), "abs-key")),
        Some("library-1"),
    )
    .await
    .unwrap();

    assert_eq!(processed, 1);
    let updated = repo.find_by_id(&request.id).await.unwrap().unwrap();
    let final_dir = audiobooks_root.join("J.R.R. Tolkien").join("The Hobbit");

    assert_eq!(updated.state, "synced");
    assert!(final_dir.exists());
    assert!(!source_file.exists());
}
