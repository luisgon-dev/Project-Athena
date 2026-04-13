use crate::domain::{
    requests::RequestRecord,
    search::{ReleaseCandidate, ScoredCandidate},
};

pub fn score_candidate(request: &RequestRecord, candidate: &ReleaseCandidate) -> ScoredCandidate {
    let mut score = 0.0;
    let mut explanation = Vec::new();
    let candidate_title = candidate.title.to_lowercase();
    let normalized_candidate = normalize(&candidate.title);
    let normalized_request_title = normalize(&request.title);

    if normalized_candidate.contains(&normalized_request_title) {
        score += 0.50;
        explanation.push("title matched".to_string());
    }

    if candidate_title.contains(&request.author.to_lowercase()) {
        score += 0.25;
        explanation.push("author matched".to_string());
    }

    if matches!(request.media_type, crate::domain::requests::MediaType::Ebook)
        && (candidate_title.contains("epub")
            || candidate_title.contains("pdf")
            || candidate_title.contains("azw3"))
    {
        score += 0.20;
        explanation.push("ebook format matched".to_string());
    }

    if matches!(
        request.media_type,
        crate::domain::requests::MediaType::Audiobook
    ) && (candidate_title.contains("m4b")
        || candidate_title.contains("mp3")
        || candidate_title.contains("audiobook"))
    {
        score += 0.20;
        explanation.push("audio format matched".to_string());
    }

    if request.manifestation.graphic_audio && !candidate_title.contains("graphicaudio") {
        score -= 0.30;
        explanation.push("graphic audio requested but candidate does not advertise it".to_string());
    }

    ScoredCandidate {
        score,
        explanation,
        auto_acquire: score >= 0.90,
    }
}

fn normalize(value: &str) -> String {
    value
        .chars()
        .map(|ch| ch.to_ascii_lowercase())
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect()
}
