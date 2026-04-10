use std::path::Path;

use book_router::config::{AppConfig, DatabaseTarget};

#[test]
fn config_defaults_to_file_backed_database() {
    let config = AppConfig::from_env_with(|key| match key {
        "BIND_ADDR" => Some("127.0.0.1:7777".into()),
        "EBOOKS_ROOT" => Some("/var/lib/books".into()),
        "AUDIOBOOKS_ROOT" => Some("/var/lib/audiobooks".into()),
        _ => None,
    })
    .unwrap();

    assert!(matches!(
        config.database,
        DatabaseTarget::File(ref path)
            if path == Path::new("/data/book-router/book-router.sqlite")
    ));
}

#[test]
fn config_rejects_relative_media_roots() {
    let config = AppConfig {
        ebooks_root: "ebooks".into(),
        audiobooks_root: "/audiobooks".into(),
        ..AppConfig::for_tests()
    };

    let error = config.validate().unwrap_err();
    assert!(error.to_string().contains("absolute path"));
}

#[test]
fn config_rejects_relative_database_path() {
    let config = AppConfig {
        database: DatabaseTarget::file("book-router.sqlite"),
        ..AppConfig::for_tests()
    };

    let error = config.validate().unwrap_err();
    assert!(error.to_string().contains("sqlite database path"));
}
