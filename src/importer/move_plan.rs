use std::path::{Path, PathBuf};

use crate::domain::settings::AudiobookLayoutPreset;

pub fn build_ebook_move_plan(
    root: &Path,
    template: &str,
    author: &str,
    work: &str,
    leaf_name: &str,
) -> PathBuf {
    let mut relative = template
        .split('/')
        .filter(|segment| !segment.trim().is_empty())
        .map(|segment| {
            normalize_path_segment(&segment.replace("{author}", author).replace("{title}", work))
        })
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    let extension = Path::new(leaf_name)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| format!(".{extension}"))
        .unwrap_or_default();

    if let Some(last) = relative.last_mut() {
        if !extension.is_empty() && !last.ends_with(&extension) {
            last.push_str(&extension);
        }
    }

    relative
        .into_iter()
        .fold(root.to_path_buf(), |path, segment| path.join(segment))
}

pub fn build_audiobook_root(
    root: &Path,
    preset: &AudiobookLayoutPreset,
    author: &str,
    work: &str,
) -> PathBuf {
    match preset {
        AudiobookLayoutPreset::AuthorTitle => root
            .join(normalize_path_segment(author))
            .join(normalize_path_segment(work)),
        AudiobookLayoutPreset::Title => root.join(normalize_path_segment(work)),
    }
}

pub fn normalize_path_segment(value: &str) -> String {
    let mut normalized = String::with_capacity(value.len());

    for character in value.chars() {
        if character.is_control()
            || matches!(
                character,
                '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
            )
        {
            normalized.push(' ');
        } else {
            normalized.push(character);
        }
    }

    normalized
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}
