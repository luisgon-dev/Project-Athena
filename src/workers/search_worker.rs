use std::collections::HashSet;
use std::time::Duration;

use anyhow::{Context, Result};
use tracing::{error, warn};

use crate::{
    db::repositories::{SearchIndexerRecord, SqliteRequestRepository, SqliteSettingsRepository},
    domain::{
        requests::{MediaType, RequestRecord},
        search::{ReleaseCandidate, ScoredCandidate},
        settings::PersistedRuntimeSettings,
    },
    downloads::qbittorrent::QbittorrentClient,
    matcher::score_candidate,
    search::{prowlarr::ProwlarrClient, torznab::TorznabClient},
};

pub struct SearchWorker;

impl SearchWorker {
    pub fn spawn(pool: sqlx::SqlitePool, settings: SqliteSettingsRepository, interval: Duration) {
        tokio::spawn(async move {
            loop {
                if let Err(error) = Self::run_once(pool.clone(), settings.clone()).await {
                    error!(error = %error, "search worker iteration failed");
                }
                tokio::time::sleep(interval).await;
            }
        });
    }

    pub async fn run_once(
        pool: sqlx::SqlitePool,
        settings: SqliteSettingsRepository,
    ) -> Result<usize> {
        let repo = SqliteRequestRepository::new(pool);
        let pending = repo.list_pending_search_requests().await?;
        let mut processed = 0usize;

        for request in pending {
            Self::process_request(&repo, &settings, &request).await?;
            processed += 1;
        }

        Ok(processed)
    }

    pub async fn process_request_by_id(
        pool: sqlx::SqlitePool,
        settings: SqliteSettingsRepository,
        request_id: &str,
    ) -> Result<()> {
        let repo = SqliteRequestRepository::new(pool);
        let request = repo
            .find_by_id(request_id)
            .await?
            .with_context(|| format!("request {request_id} not found"))?;
        Self::process_request(&repo, &settings, &request).await
    }

    async fn process_request(
        repo: &SqliteRequestRepository,
        settings_repo: &SqliteSettingsRepository,
        request: &RequestRecord,
    ) -> Result<()> {
        let settings = settings_repo
            .get_persisted_runtime_settings()
            .await?
            .context("runtime settings row missing")?;

        let mut candidates = Self::search_candidates(settings_repo, request, &settings).await;
        if candidates.is_empty() {
            repo.clear_review_queue(&request.id).await?;
            repo.mark_search_completed(&request.id, "no_match", 0, 0, None)
                .await?;
            return Ok(());
        }

        let rejected = repo
            .rejected_candidate_ids(&request.id)
            .await?
            .into_iter()
            .collect::<HashSet<_>>();
        let candidates_seen = candidates.len();
        candidates.retain(|candidate| !rejected.contains(&candidate.external_id));

        let qualified = Self::qualify_candidates(request, &settings, candidates);
        let qualified_candidates = qualified.len();
        let top_score = qualified.first().map(|entry| entry.1.score);

        if let Some((candidate, scored)) = qualified.first() {
            if scored.auto_acquire
                && !request.requires_admin_approval
                && candidate.download_url.is_some()
                && settings.download_clients.qbittorrent.enabled
            {
                let qb = &settings.download_clients.qbittorrent;
                let client = QbittorrentClient::new(
                    &qb.base_url,
                    &qb.username,
                    qb.password.clone().unwrap_or_default(),
                );
                match crate::workers::download_worker::DownloadWorker::dispatch_approved_candidate(
                    repo,
                    &client,
                    &request.id,
                    candidate,
                )
                .await
                {
                    Ok(()) => {
                        repo.clear_review_queue(&request.id).await?;
                        repo.mark_search_completed(
                            &request.id,
                            "auto_acquire",
                            candidates_seen,
                            qualified_candidates,
                            top_score,
                        )
                        .await?;
                        return Ok(());
                    }
                    Err(error) => {
                        warn!(error = %error, request_id = %request.id, "automatic dispatch failed; falling back to review queue");
                    }
                }
            }
        }

        repo.clear_review_queue(&request.id).await?;
        if qualified.is_empty() {
            repo.mark_search_completed(&request.id, "no_match", candidates_seen, 0, None)
                .await?;
            return Ok(());
        }

        for (candidate, scored) in qualified.iter().take(10) {
            repo.enqueue_review_candidate(&request.id, candidate, scored)
                .await?;
        }
        repo.mark_review_queued(&request.id, qualified.len().min(10), top_score)
            .await?;

        Ok(())
    }

    async fn search_candidates(
        settings_repo: &SqliteSettingsRepository,
        request: &RequestRecord,
        settings: &PersistedRuntimeSettings,
    ) -> Vec<ReleaseCandidate> {
        let query = build_query(request);
        let media_type = match request.media_type {
            MediaType::Ebook => "book",
            MediaType::Audiobook => "audio",
        };
        let mut results = Vec::new();
        let selected_indexer_ids = settings.integrations.prowlarr.selected_indexer_ids.clone();

        if settings.integrations.prowlarr.enabled {
            let prowlarr = &settings.integrations.prowlarr;
            if let Some(api_key) = &prowlarr.api_key {
                let client = ProwlarrClient::new(&prowlarr.base_url, api_key);
                match client
                    .search(&query, media_type, &selected_indexer_ids)
                    .await
                {
                    Ok(candidates) => results.extend(candidates),
                    Err(error) => warn!(
                        error = %error,
                        request_id = %request.id,
                        title = %request.title,
                        author = %request.author,
                        query = %query,
                        "prowlarr search failed"
                    ),
                }
            }
        }

        match settings_repo.list_search_indexers().await {
            Ok(indexers) => {
                for indexer in indexers {
                    if !selected_indexer_ids.is_empty()
                        && indexer
                            .prowlarr_indexer_id
                            .and_then(|id| i32::try_from(id).ok())
                            .is_none_or(|id| !selected_indexer_ids.contains(&id))
                    {
                        continue;
                    }
                    if let Err(error) =
                        Self::search_direct_indexer(&mut results, request, &query, indexer).await
                    {
                        warn!(error = %error, request_id = %request.id, "direct indexer search failed");
                    }
                }
            }
            Err(error) => {
                warn!(error = %error, request_id = %request.id, "failed to load synced indexers")
            }
        }

        dedupe_candidates(results)
    }

    async fn search_direct_indexer(
        results: &mut Vec<ReleaseCandidate>,
        request: &RequestRecord,
        query: &str,
        indexer: SearchIndexerRecord,
    ) -> Result<()> {
        let client = TorznabClient::new_with_api_path(
            &indexer.base_url,
            indexer.api_key.clone(),
            indexer.api_path.clone(),
        );
        let mut candidates = client.search(query).await?;
        for candidate in &mut candidates {
            candidate.indexer = format!("indexer-{}", indexer.id);
        }
        results.extend(candidates);
        let _ = request;
        Ok(())
    }

    fn qualify_candidates(
        request: &RequestRecord,
        settings: &PersistedRuntimeSettings,
        candidates: Vec<ReleaseCandidate>,
    ) -> Vec<(ReleaseCandidate, ScoredCandidate)> {
        let preferred_language = request
            .preferred_language
            .as_deref()
            .or(settings.acquisition.preferred_language.as_deref())
            .and_then(normalize_language_preference);
        let blocked_terms = settings
            .acquisition
            .blocked_terms
            .iter()
            .map(|term| term.to_lowercase())
            .collect::<Vec<_>>();

        let mut qualified = candidates
            .into_iter()
            .filter(|candidate| !contains_blocked_term(candidate, &blocked_terms))
            .filter(|candidate| language_matches(candidate, preferred_language.as_deref()))
            .map(|candidate| {
                let mut scored = score_candidate(request, &candidate);
                if preferred_language
                    .as_deref()
                    .zip(candidate.detected_language.as_deref())
                    .is_some_and(|(preferred, detected)| preferred == detected)
                {
                    scored.score += 0.05;
                    scored
                        .explanation
                        .push("preferred language matched".to_string());
                }
                (candidate, scored)
            })
            .filter(|(_, scored)| scored.score >= settings.acquisition.minimum_score)
            .collect::<Vec<_>>();

        qualified.sort_by(|left, right| {
            right
                .1
                .score
                .partial_cmp(&left.1.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        qualified
    }
}

fn build_query(request: &RequestRecord) -> String {
    [request.title.trim(), request.author.trim()]
        .into_iter()
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn contains_blocked_term(candidate: &ReleaseCandidate, blocked_terms: &[String]) -> bool {
    let title = candidate.title.to_lowercase();
    blocked_terms
        .iter()
        .any(|term| !term.is_empty() && title.contains(term))
}

fn language_matches(candidate: &ReleaseCandidate, preferred_language: Option<&str>) -> bool {
    let Some(preferred_language) = preferred_language else {
        return true;
    };

    candidate
        .detected_language
        .as_deref()
        .is_none_or(|detected| detected == preferred_language)
}

fn dedupe_candidates(candidates: Vec<ReleaseCandidate>) -> Vec<ReleaseCandidate> {
    let mut seen = HashSet::new();
    let mut unique = Vec::new();

    for candidate in candidates {
        let key = format!("{}::{}", candidate.source, candidate.external_id);
        if seen.insert(key) {
            unique.push(candidate);
        }
    }

    unique
}

fn normalize_language_preference(value: &str) -> Option<String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "en" | "eng" | "english" => Some("en".to_string()),
        "es" | "spa" | "spanish" => Some("es".to_string()),
        "" => None,
        other => Some(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::requests::{ManifestationPreference, MediaType, RequestRecord};

    use super::{build_query, language_matches};
    use crate::domain::search::ReleaseCandidate;

    #[test]
    fn build_query_does_not_append_language() {
        let request = RequestRecord::for_tests("The Hobbit", "J.R.R. Tolkien", MediaType::Ebook)
            .with_preferences(ManifestationPreference::default());

        let query = build_query(&request);

        assert_eq!(query, "The Hobbit J.R.R. Tolkien");
    }

    #[test]
    fn build_query_omits_audiobook_tokens_and_narrator() {
        let request =
            RequestRecord::for_tests("The Hobbit", "J.R.R. Tolkien", MediaType::Audiobook)
                .with_preferences(ManifestationPreference {
                    edition_title: None,
                    preferred_narrator: Some("Andy Serkis".to_string()),
                    preferred_publisher: None,
                    graphic_audio: false,
                });

        let query = build_query(&request);

        assert_eq!(query, "The Hobbit J.R.R. Tolkien");
    }

    #[test]
    fn language_filter_keeps_untagged_and_matching_candidates() {
        let untagged = ReleaseCandidate::for_tests("The Hobbit EPUB");
        let matching = ReleaseCandidate::for_tests("The Hobbit [ENG] EPUB");
        let non_matching = ReleaseCandidate::for_tests("The Hobbit [SPA] EPUB");

        assert!(language_matches(&untagged, Some("en")));
        assert!(language_matches(&matching, Some("en")));
        assert!(!language_matches(&non_matching, Some("en")));
    }
}
