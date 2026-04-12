use crate::domain::imports::Classification;

pub fn classify_payload(files: &[String]) -> Classification {
    let has_audio = files
        .iter()
        .any(|file| file.ends_with(".m4b") || file.ends_with(".mp3") || file.ends_with(".flac"));
    let has_ebook = files
        .iter()
        .any(|file| file.ends_with(".epub") || file.ends_with(".pdf") || file.ends_with(".azw3"));

    match (has_audio, has_ebook) {
        (true, false) => Classification::audiobook(),
        (false, true) => Classification::ebook(),
        (true, true) => Classification::mixed(),
        (false, false) => Classification::invalid("no supported media files"),
    }
}
