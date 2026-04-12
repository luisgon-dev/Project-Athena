use crate::domain::{
    requests::RequestRecord,
    search::{ReleaseCandidate, ScoredCandidate},
};

pub fn score_candidate(request: &RequestRecord, candidate: &ReleaseCandidate) -> ScoredCandidate {
    let mut score = 0.0;
    let mut explanation = Vec::new();
    let candidate_title = candidate.title.to_lowercase();

    if normalized_eq(&request.title, &candidate.title) {
        score += 0.45;
        explanation.push("title matched".to_string());
    }

    if candidate_title.contains(&request.author.to_lowercase()) {
        score += 0.25;
        explanation.push("author matched".to_string());
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

fn normalized_eq(left: &str, right: &str) -> bool {
    normalize(left) == normalize(right)
}

fn normalize(value: &str) -> String {
    value
        .chars()
        .map(|ch| ch.to_ascii_lowercase())
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect()
}
