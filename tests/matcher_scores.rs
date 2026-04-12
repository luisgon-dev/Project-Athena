use book_router::{
    domain::{
        requests::{ManifestationPreference, MediaType, RequestRecord},
        search::ReleaseCandidate,
    },
    matcher::score_candidate,
};

#[test]
fn graphic_audio_preference_penalizes_plain_audio_release() {
    let request = RequestRecord::for_tests("The Sandman", "Neil Gaiman", MediaType::Audiobook)
        .with_preferences(ManifestationPreference {
            edition_title: None,
            preferred_narrator: None,
            preferred_publisher: None,
            graphic_audio: true,
        });

    let candidate = ReleaseCandidate::for_tests("The Sandman Unabridged M4B");
    let scored = score_candidate(&request, &candidate);

    assert!(scored.score < 0.80);
    assert!(scored.explanation.join(" ").contains("graphic audio"));
}
