use std::path::{Path, PathBuf};

pub fn build_move_plan(root: &Path, author: &str, work: &str, leaf_name: &str) -> PathBuf {
    let normalized_author = normalize_path_segment(author);
    let normalized_work = normalize_path_segment(work);
    let extension = Path::new(leaf_name)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| format!(".{extension}"))
        .unwrap_or_default();
    let normalized_leaf = format!("{normalized_work}{extension}");

    root.join(normalized_author)
        .join(&normalized_work)
        .join(normalized_leaf)
}

fn normalize_path_segment(value: &str) -> String {
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
