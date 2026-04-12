use book_router::sync::{audiobookshelf::AudiobookshelfClient, calibre::CalibreHook};

#[tokio::test]
async fn audiobookshelf_scan_request_uses_api_key_auth() {
    let client = AudiobookshelfClient::new("http://localhost:13378", "abs-key");
    let request = client.scan_library("library-1").build().unwrap();

    assert_eq!(request.headers()["Authorization"], "Bearer abs-key");
}

#[test]
fn calibre_hook_builds_import_command() {
    let hook = CalibreHook::new("/usr/bin/calibredb");
    let command = hook.add_book_command("/ebooks/Tolkien/The Hobbit/The Hobbit.epub");

    assert_eq!(
        command.get_program().to_string_lossy(),
        "/usr/bin/calibredb"
    );
}
