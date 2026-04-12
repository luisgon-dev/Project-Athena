use book_router::importer::classify::classify_payload;
use book_router::importer::move_plan::build_move_plan;
use std::path::{Path, PathBuf};

#[test]
fn m4b_folder_is_classified_as_audiobook() {
    let classification = classify_payload(&[
        "The Hobbit/part01.m4b".into(),
        "The Hobbit/cover.jpg".into(),
    ]);

    assert_eq!(classification.media_type.as_str(), "audiobook");
}

#[test]
fn epub_file_is_classified_as_ebook() {
    let classification = classify_payload(&["The Hobbit.epub".into()]);

    assert_eq!(classification.media_type.as_str(), "ebook");
}

#[test]
fn ebook_move_plan_normalizes_leaf_name_to_work_title() {
    let destination = build_move_plan(
        Path::new("/ebooks"),
        "J.R.R. Tolkien",
        "The Hobbit: There and Back Again",
        "The.Hobbit.50th.Anniversary.Release.epub",
    );

    assert_eq!(
        destination,
        PathBuf::from(
            "/ebooks/J.R.R. Tolkien/The Hobbit There and Back Again/The Hobbit There and Back Again.epub",
        ),
    );
}
