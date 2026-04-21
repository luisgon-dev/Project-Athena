use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ReleaseCandidate {
    pub external_id: String,
    pub source: String,
    pub title: String,
    pub protocol: String,
    pub size_bytes: i64,
    pub indexer: String,
    pub download_url: Option<String>,
    pub narrator: Option<String>,
    #[serde(default)]
    pub graphic_audio: bool,
    pub detected_language: Option<String>,
}

impl ReleaseCandidate {
    pub fn for_tests(title: impl Into<String>) -> Self {
        Self {
            external_id: "candidate-1".to_string(),
            source: "test".to_string(),
            title: title.into(),
            protocol: "torrent".to_string(),
            size_bytes: 0,
            indexer: "test-indexer".to_string(),
            download_url: Some(
                "magnet:?xt=urn:btih:0123456789abcdef0123456789abcdef01234567&dn=test".to_string(),
            ),
            narrator: None,
            graphic_audio: false,
            detected_language: None,
        }
        .with_parsed_metadata()
    }

    pub fn with_parsed_metadata(mut self) -> Self {
        let parsed = ParsedTitleMetadata::from_title(&self.title);
        self.narrator = parsed.narrator;
        self.graphic_audio = parsed.graphic_audio;
        self.detected_language = parsed.detected_language;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ScoredCandidate {
    pub score: f32,
    pub explanation: Vec<String>,
    pub auto_acquire: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ReviewQueueEntry {
    pub id: i64,
    pub request_id: String,
    pub candidate: ReleaseCandidate,
    pub score: f32,
    pub explanation: Vec<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedTitleMetadata {
    pub narrator: Option<String>,
    pub graphic_audio: bool,
    pub detected_language: Option<String>,
}

impl ParsedTitleMetadata {
    pub fn from_title(title: &str) -> Self {
        Self {
            narrator: parse_narrator(title),
            graphic_audio: is_graphic_audio(title),
            detected_language: detect_language(title),
        }
    }
}

fn parse_narrator(title: &str) -> Option<String> {
    let lowercase = title.to_lowercase();
    for marker in ["narrated by ", "read by "] {
        if let Some(start) = lowercase.find(marker) {
            let suffix = &title[start + marker.len()..];
            let narrator = truncate_metadata_segment(suffix);
            if !narrator.is_empty() {
                return Some(narrator);
            }
        }
    }

    None
}

fn truncate_metadata_segment(value: &str) -> String {
    let mut result = value.trim();

    for separator in [" - ", " | ", " [", " (", " / "] {
        if let Some(index) = result.find(separator) {
            result = &result[..index];
        }
    }

    result
        .trim_matches(|character: char| {
            character.is_whitespace() || matches!(character, '-' | '|' | '[' | ']' | '(' | ')')
        })
        .to_string()
}

fn is_graphic_audio(title: &str) -> bool {
    let lowercase = title.to_lowercase();
    lowercase.contains("graphic audio")
        || lowercase.contains("graphicaudio")
        || lowercase.contains("dramatized adaptation")
        || lowercase.contains("dramatised adaptation")
}

fn detect_language(title: &str) -> Option<String> {
    tokenize(title)
        .into_iter()
        .find_map(|token| canonical_language_token(&token).map(str::to_string))
}

fn tokenize(value: &str) -> Vec<String> {
    value
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.to_ascii_lowercase())
        .collect()
}

fn canonical_language_token(token: &str) -> Option<&'static str> {
    match token {
        "en" | "eng" | "english" => Some("en"),
        "es" | "spa" | "spanish" => Some("es"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::ParsedTitleMetadata;

    #[test]
    fn parses_narrator_and_graphic_audio_from_title() {
        let parsed = ParsedTitleMetadata::from_title(
            "The Sandman - Narrated by Neil Gaiman [ENG] GraphicAudio M4B",
        );

        assert_eq!(parsed.narrator.as_deref(), Some("Neil Gaiman"));
        assert!(parsed.graphic_audio);
        assert_eq!(parsed.detected_language.as_deref(), Some("en"));
    }

    #[test]
    fn leaves_unstructured_titles_without_narrator() {
        let parsed = ParsedTitleMetadata::from_title("The Hobbit J.R.R. Tolkien M4B");

        assert_eq!(parsed.narrator, None);
        assert!(!parsed.graphic_audio);
        assert_eq!(parsed.detected_language, None);
    }
}
