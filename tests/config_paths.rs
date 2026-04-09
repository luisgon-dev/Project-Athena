use book_router::config::AppConfig;

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
