# Book Router V1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first working vertical slice of a Rust/SQLite, Docker-first book and audiobook request router that owns metadata, searches via Prowlarr and direct indexers, dispatches to qBittorrent, imports into `/ebooks` or `/audiobooks`, and syncs outward to Calibre and Audiobookshelf.

**Architecture:** Implement a modular Rust monolith using `axum` for the admin UI/API, `sqlx` with SQLite for persistence, `reqwest` for external adapters, and in-process background workers for search, dispatch, import, and sync. Build the system in thin vertical slices so each task leaves the repo in a runnable, testable state with explicit request events and deterministic file-path handling.

**Tech Stack:** Rust, axum, tokio, sqlx (SQLite), reqwest, serde, askama, quick-xml, lofty, tempfile, tracing, testcontainers or docker-compose for end-to-end verification.

---

## Progress Snapshot As Of 2026-04-10

Merged on `main` today:

- foundation app bootstrap and `/health`
- SQLite migration plus request and event persistence
- Open Library work resolution
- admin request UI/API scaffold

Important current limitation:

- the app still creates an in-memory SQLite database at startup, so requests do not survive process restarts

Approved scope revision:

- the merged request UI is now a transitional scaffold
- request creation must be metadata-first:
  - search metadata first
  - select a canonical work
  - choose `ebook`, `audiobook`, or both
  - optionally refine manifestation preferences
  - create one or two requests only from a metadata-backed match
- there is no free-text fallback request path in v1

## UI & Architecture Modernization Strategy (New Priority)

To address fragility in the current SSR setup and ensure the admin UI feels fast and modern, we are adopting a separated frontend architecture bundled via Docker. This will increase build complexity but vastly improve error handling and developer experience. The implementation must prioritize these two phases for continuity:

- **Phase 1: Stabilize the Backend.** Refactor the Rust API to gracefully handle external API failures (e.g. Open Library), returning clean JSON errors instead of timeouts/panics. Move blocking startup tasks to background workers.
- **Phase 2: Build the Frontend.** Replace the Askama SSR templates with a modern SPA. The SPA will be compiled during a multi-stage Docker build and served statically by the Rust backend in production.

## Revised Immediate Next Milestone

Before continuing with Prowlarr or download work, implement this milestone:

1. persistent SQLite configuration and file-backed startup
2. metadata-first request window
3. canonical work linkage in stored requests
4. dual-media convenience creation from one selected work
5. validation that creates only valid media-type requests when both are selected

This is now the recommended next execution order:

1. Make SQLite persistent across restarts
2. Replace free-text request creation with metadata search then request creation
3. Add Prowlarr search
4. Add direct Torznab search
5. Add candidate scoring and review queue
6. Add qBittorrent dispatch
7. Add import classification and final move planning
8. Add Calibre and Audiobookshelf sync
9. Add Docker packaging and end-to-end harness coverage

## File Structure

Create these files and keep responsibilities narrow:

- `Cargo.toml`
  - crate metadata and dependencies
- `.gitignore`
  - Rust, SQLite, Docker, and local media ignores
- `.env.example`
  - sample config values for local and container runs
- `Dockerfile`
  - multi-stage Rust build for the app container
- `compose.yml`
  - app, qBittorrent, and optional mock adapters for end-to-end tests
- `migrations/0001_initial.sql`
  - core schema for requests, works, manifestations, candidates, downloads, imports, external links, and events
- `migrations/0002_search_and_sync.sql`
  - review queue, sync jobs, and direct indexer tables
- `src/lib.rs`
  - top-level module exports
- `src/main.rs`
  - startup, config load, tracing, router, and worker boot
- `src/config.rs`
  - typed config parsing and path validation
- `src/app.rs`
  - app state wiring and startup helpers
- `src/db/mod.rs`
  - SQLite pool setup and migrations
- `src/db/repositories.rs`
  - repository traits and SQLx implementations
- `src/domain/mod.rs`
  - domain module exports
- `src/domain/catalog.rs`
  - `Author`, `Series`, `Work`, `Manifestation`, and related value types
- `src/domain/requests.rs`
  - request lifecycle models and commands
- `src/domain/search.rs`
  - `ReleaseCandidate`, match explanations, and search query plan
- `src/domain/imports.rs`
  - import classification, move plan, and sync state
- `src/domain/events.rs`
  - append-only request event types
- `src/http/mod.rs`
  - router assembly
- `src/http/handlers/health.rs`
  - health endpoint
- `src/http/handlers/requests.rs`
  - request create, list, detail, approve, and reject handlers
- `src/http/handlers/search.rs`
  - manual search and candidate review handlers
- `src/http/handlers/events.rs`
  - request event history endpoints
- `src/http/views.rs`
  - Askama view models and template rendering helpers
- `src/metadata/mod.rs`
  - metadata adapter trait
- `src/metadata/openlibrary.rs`
  - Open Library primary provider client
- `src/metadata/google_books.rs`
  - optional enrichment client
- `src/search/mod.rs`
  - search adapter trait and orchestration
- `src/search/prowlarr.rs`
  - Prowlarr search adapter
- `src/search/torznab.rs`
  - direct Torznab adapter
- `src/matcher.rs`
  - weighted confidence scoring and hard filters
- `src/downloads/mod.rs`
  - download client trait and worker interface
- `src/downloads/qbittorrent.rs`
  - qBittorrent adapter
- `src/importer/mod.rs`
  - import orchestration
- `src/importer/classify.rs`
  - ebook/audiobook/mixed/invalid payload classification
- `src/importer/move_plan.rs`
  - deterministic rename and move planning
- `src/sync/mod.rs`
  - sync adapter trait and job orchestration
- `src/sync/audiobookshelf.rs`
  - Audiobookshelf API adapter
- `src/sync/calibre.rs`
  - Calibre command-hook adapter
- `src/workers/mod.rs`
  - worker bootstrapping
- `src/workers/search_worker.rs`
  - automatic candidate search and queueing
- `src/workers/download_worker.rs`
  - qBittorrent dispatch and completion polling
- `src/workers/import_worker.rs`
  - import worker and sync enqueue
- `src/workers/sync_worker.rs`
  - Calibre and Audiobookshelf sync execution
- `templates/layout.html`
  - base admin page
- `templates/requests/index.html`
  - request list and create form
- `templates/requests/show.html`
  - request detail, candidate review, and event trace
- `tests/health_smoke.rs`
  - router smoke test
- `tests/config_paths.rs`
  - config and filesystem-root validation
- `tests/request_repo.rs`
  - repository persistence tests
- `tests/openlibrary_client.rs`
  - metadata adapter tests
- `tests/search_prowlarr.rs`
  - Prowlarr normalization tests
- `tests/search_torznab.rs`
  - direct indexer normalization tests
- `tests/matcher_scores.rs`
  - scoring and hard-filter tests
- `tests/qbittorrent_client.rs`
  - qBittorrent adapter tests
- `tests/import_classifier.rs`
  - import classification and move plan tests
- `tests/sync_adapters.rs`
  - Audiobookshelf and Calibre adapter tests
- `tests/e2e_request_flow.rs`
  - end-to-end request -> search -> dispatch -> import -> sync flow

## Task 1: Bootstrap The Repository And Rust App Skeleton

**Files:**
- Create: `.gitignore`
- Create: `.env.example`
- Create: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `src/main.rs`
- Create: `src/app.rs`
- Create: `src/config.rs`
- Create: `src/http/mod.rs`
- Create: `src/http/handlers/health.rs`
- Test: `tests/health_smoke.rs`
- Test: `tests/config_paths.rs`

- [ ] **Step 1: Write the failing health and config tests**

```rust
// tests/health_smoke.rs
use axum::{body::Body, http::{Request, StatusCode}};
use book_router::{app::build_app, config::AppConfig};
use tower::util::ServiceExt;

#[tokio::test]
async fn health_returns_ok() {
    let config = AppConfig::for_tests();
    let app = build_app(config).await.unwrap();

    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
```

```rust
// tests/config_paths.rs
use book_router::config::AppConfig;

#[test]
fn config_rejects_relative_media_roots() {
    let config = AppConfig {
        ebooks_root: "ebooks".into(),
        audiobooks_root: "/audiobooks".into(),
        ..AppConfig::for_tests()
    };

    let error = config.validate().unwrap_err();
    assert!(error.to_string().contains("absolute path"));
}
```

- [ ] **Step 2: Run tests to verify the crate does not build yet**

Run: `cargo test health_returns_ok config_rejects_relative_media_roots -- --nocapture`

Expected: FAIL with missing crate files and unresolved imports such as `use of unresolved module or unlinked crate 'book_router'`.

- [ ] **Step 3: Create the minimal app skeleton and config validation**

```toml
# Cargo.toml
[package]
name = "book_router"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
axum = "0.8"
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
tower = "0.5"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

```rust
// src/config.rs
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub bind_addr: String,
    pub ebooks_root: PathBuf,
    pub audiobooks_root: PathBuf,
}

impl AppConfig {
    pub fn for_tests() -> Self {
        Self {
            bind_addr: "127.0.0.1:0".into(),
            ebooks_root: "/ebooks".into(),
            audiobooks_root: "/audiobooks".into(),
        }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        for path in [&self.ebooks_root, &self.audiobooks_root] {
            if !path.is_absolute() {
                anyhow::bail!("media roots must be absolute paths");
            }
        }
        Ok(())
    }
}
```

```rust
// src/http/handlers/health.rs
use axum::http::StatusCode;

pub async fn health() -> StatusCode {
    StatusCode::OK
}
```

- [ ] **Step 4: Run tests to verify the app skeleton passes**

Run: `cargo test health_returns_ok config_rejects_relative_media_roots -- --nocapture`

Expected: PASS for both tests.

- [ ] **Step 5: Commit the scaffold**

```bash
git add .gitignore .env.example Cargo.toml src tests
git commit -m "chore: bootstrap rust app skeleton"
```

## Task 2: Add SQLite Schema, Migrations, And Request Event Persistence

**Files:**
- Create: `migrations/0001_initial.sql`
- Create: `src/db/mod.rs`
- Create: `src/db/repositories.rs`
- Create: `src/domain/mod.rs`
- Create: `src/domain/catalog.rs`
- Create: `src/domain/requests.rs`
- Create: `src/domain/events.rs`
- Modify: `src/app.rs`
- Test: `tests/request_repo.rs`

- [ ] **Step 1: Write the failing repository test for request creation and event append**

```rust
// tests/request_repo.rs
use book_router::{
    db::{connect_sqlite, repositories::SqliteRequestRepository},
    domain::requests::{CreateRequest, MediaType},
};

#[tokio::test]
async fn create_request_persists_initial_event() {
    let pool = connect_sqlite("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    let repo = SqliteRequestRepository::new(pool);

    let request = repo.create(CreateRequest {
        title: "The Hobbit".into(),
        author: "J.R.R. Tolkien".into(),
        media_type: MediaType::Audiobook,
        preferred_language: Some("en".into()),
    }).await.unwrap();

    let events = repo.events_for(request.id).await.unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind.as_str(), "request.created");
}
```

- [ ] **Step 2: Run the repository test and confirm the schema is missing**

Run: `cargo test create_request_persists_initial_event -- --nocapture`

Expected: FAIL with missing `db` module, missing `CreateRequest`, or missing migration files.

- [ ] **Step 3: Implement the initial schema and repository**

```sql
-- migrations/0001_initial.sql
CREATE TABLE requests (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    media_type TEXT NOT NULL,
    preferred_language TEXT,
    state TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE request_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    request_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (request_id) REFERENCES requests(id) ON DELETE CASCADE
);
```

```rust
// src/domain/requests.rs
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MediaType {
    Ebook,
    Audiobook,
}

pub struct CreateRequest {
    pub title: String,
    pub author: String,
    pub media_type: MediaType,
    pub preferred_language: Option<String>,
}
```

```rust
// src/db/repositories.rs
pub struct SqliteRequestRepository {
    pool: sqlx::SqlitePool,
}

impl SqliteRequestRepository {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        Self { pool }
    }
}
```

- [ ] **Step 4: Run the repository test to verify request creation and event append**

Run: `cargo test create_request_persists_initial_event -- --nocapture`

Expected: PASS with one persisted `request.created` event.

- [ ] **Step 5: Commit the persistence slice**

```bash
git add migrations src/db src/domain tests/request_repo.rs Cargo.toml
git commit -m "feat: persist requests and request events"
```

## Task 3: Implement Open Library Metadata Resolution For Work-Level Requests

**Files:**
- Create: `src/metadata/mod.rs`
- Create: `src/metadata/openlibrary.rs`
- Modify: `src/domain/catalog.rs`
- Modify: `src/domain/requests.rs`
- Modify: `src/app.rs`
- Test: `tests/openlibrary_client.rs`

- [ ] **Step 1: Write the failing metadata-resolution test**

```rust
// tests/openlibrary_client.rs
use book_router::metadata::openlibrary::OpenLibraryClient;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, query_param};

#[tokio::test]
async fn resolves_work_and_primary_manifestation_from_open_library() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/search.json"))
        .and(query_param("title", "The Hobbit"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(r#"{
            "docs":[{"key":"OL27448W","title":"The Hobbit","author_name":["J.R.R. Tolkien"],"language":["eng"]}]
        }"#, "application/json"))
        .mount(&server)
        .await;

    let client = OpenLibraryClient::new(server.uri());
    let result = client.resolve_work("The Hobbit", "J.R.R. Tolkien").await.unwrap();

    assert_eq!(result.work.title, "The Hobbit");
    assert_eq!(result.work.primary_author, "J.R.R. Tolkien");
}
```

- [ ] **Step 2: Run the test and verify the metadata adapter is missing**

Run: `cargo test resolves_work_and_primary_manifestation_from_open_library -- --nocapture`

Expected: FAIL with missing `metadata` module or missing `OpenLibraryClient`.

- [ ] **Step 3: Implement the Open Library client and work-resolution mapping**

```rust
// src/metadata/openlibrary.rs
#[derive(Clone)]
pub struct OpenLibraryClient {
    base_url: String,
    http: reqwest::Client,
}

impl OpenLibraryClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self { base_url: base_url.into(), http: reqwest::Client::new() }
    }

    pub async fn resolve_work(&self, title: &str, author: &str) -> anyhow::Result<ResolvedWork> {
        let response = self.http
            .get(format!("{}/search.json", self.base_url))
            .query(&[("title", title), ("author", author), ("limit", "5")])
            .send()
            .await?
            .error_for_status()?;

        let payload: SearchResponse = response.json().await?;
        let first = payload.docs.into_iter().next().ok_or_else(|| anyhow::anyhow!("no work match"))?;

        Ok(ResolvedWork {
            work: WorkRecord {
                external_id: first.key,
                title: first.title,
                primary_author: first.author_name.into_iter().next().unwrap_or_default(),
            },
        })
    }
}
```

- [ ] **Step 4: Run the metadata test**

Run: `cargo test resolves_work_and_primary_manifestation_from_open_library -- --nocapture`

Expected: PASS with the work title and primary author populated from the mock response.

- [ ] **Step 5: Commit metadata resolution**

```bash
git add src/metadata src/domain tests/openlibrary_client.rs Cargo.toml
git commit -m "feat: resolve works from open library"
```

## Task 4: Build The Admin Request UI And API With Optional Manifestation Preferences

Status note:
This task has been merged as an initial scaffold, but its original free-text request entry path is now superseded by the approved metadata-first request contract. Treat the current implementation as temporary and use Task 4A to replace it.

**Files:**
- Create: `src/http/handlers/requests.rs`
- Create: `src/http/views.rs`
- Create: `templates/layout.html`
- Create: `templates/requests/index.html`
- Create: `templates/requests/show.html`
- Modify: `src/http/mod.rs`
- Modify: `src/domain/requests.rs`
- Test: `tests/request_http.rs`

- [ ] **Step 1: Write the failing request-creation HTTP test**

```rust
// tests/request_http.rs
use axum::{body::Body, http::{Request, StatusCode}};
use book_router::{app::build_app, config::AppConfig};
use tower::util::ServiceExt;

#[tokio::test]
async fn post_requests_creates_a_work_level_request() {
    let app = build_app(AppConfig::for_tests()).await.unwrap();

    let response = app.oneshot(
        Request::builder()
            .method("POST")
            .uri("/requests")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from("title=The+Hobbit&author=J.R.R.+Tolkien&media_type=audiobook&preferred_language=en&preferred_narrator=Andy+Serkis"))
            .unwrap()
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
}
```

- [ ] **Step 2: Run the request HTTP test**

Run: `cargo test post_requests_creates_a_work_level_request -- --nocapture`

Expected: FAIL with missing `/requests` route or missing form types.

- [ ] **Step 3: Implement request handlers, views, and manifestation preference fields**

```rust
// src/domain/requests.rs
pub struct ManifestationPreference {
    pub edition_title: Option<String>,
    pub preferred_narrator: Option<String>,
    pub preferred_publisher: Option<String>,
    pub graphic_audio: bool,
}

pub struct CreateRequest {
    pub title: String,
    pub author: String,
    pub media_type: MediaType,
    pub preferred_language: Option<String>,
    pub manifestation: ManifestationPreference,
}
```

```rust
// src/http/handlers/requests.rs
#[derive(serde::Deserialize)]
pub struct CreateRequestForm {
    pub title: String,
    pub author: String,
    pub media_type: String,
    pub preferred_language: Option<String>,
    pub edition_title: Option<String>,
    pub preferred_narrator: Option<String>,
    pub preferred_publisher: Option<String>,
    pub graphic_audio: Option<String>,
}
```

- [ ] **Step 4: Run the request HTTP test**

Run: `cargo test post_requests_creates_a_work_level_request -- --nocapture`

Expected: PASS with a redirect to the request detail page.

- [ ] **Step 5: Commit the admin request flow**

```bash
git add src/http templates src/domain/requests.rs tests/request_http.rs Cargo.toml
git commit -m "feat: add admin request ui and api"
```

## Task 4A: Replace Free-Text Request Creation With Metadata-First Request Flow

**Why this task exists:**
The approved design was revised after the initial Task 4 scaffold landed. Requests must now be created only from metadata-provider matches, not from free-text title and author submission.

**Files:**
- Modify: `src/app.rs`
- Modify: `src/config.rs`
- Modify: `src/db/repositories.rs`
- Modify: `src/domain/catalog.rs`
- Modify: `src/domain/requests.rs`
- Modify: `src/http/handlers/requests.rs`
- Modify: `src/http/views.rs`
- Modify: `templates/requests/index.html`
- Modify: `templates/requests/show.html`
- Create: `tests/request_metadata_flow.rs`

### Goals

- add a metadata search step to the request window
- require work selection before request creation
- allow `ebook`, `audiobook`, or both from one selected work
- create separate requests when both media types are chosen
- refuse request creation when metadata search finds no acceptable match
- persist canonical work identity with the request instead of free-text-only intent

### Suggested Verification Targets

- searching metadata returns matched works
- submitting a selected work plus `ebook` creates one request
- submitting a selected work plus `audiobook` creates one request
- submitting both creates two requests linked to the same work
- if one media type is invalid, only the valid request is created and the UI explains the skipped side
- restarting the app preserves requests after persistent SQLite is added

## Task 5: Implement Prowlarr Search And Candidate Normalization

**Files:**
- Create: `src/domain/search.rs`
- Create: `src/search/mod.rs`
- Create: `src/search/prowlarr.rs`
- Create: `src/workers/search_worker.rs`
- Modify: `src/workers/mod.rs`
- Test: `tests/search_prowlarr.rs`

- [ ] **Step 1: Write the failing Prowlarr normalization test**

```rust
// tests/search_prowlarr.rs
use book_router::search::prowlarr::ProwlarrClient;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn normalizes_prowlarr_results_into_release_candidates() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/search"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(r#"[
            {"guid":"abc","title":"The Hobbit Andy Serkis M4B","size":1234,"protocol":"torrent","indexer":"Books"}
        ]"#, "application/json"))
        .mount(&server)
        .await;

    let client = ProwlarrClient::new(server.uri(), "test-api-key");
    let results = client.search("The Hobbit", "audiobook").await.unwrap();

    assert_eq!(results[0].source, "prowlarr");
    assert_eq!(results[0].title, "The Hobbit Andy Serkis M4B");
}
```

- [ ] **Step 2: Run the Prowlarr test**

Run: `cargo test normalizes_prowlarr_results_into_release_candidates -- --nocapture`

Expected: FAIL with missing search client and candidate model.

- [ ] **Step 3: Implement the search trait, Prowlarr adapter, and normalized candidate model**

```rust
// src/domain/search.rs
pub struct ReleaseCandidate {
    pub external_id: String,
    pub source: String,
    pub title: String,
    pub protocol: String,
    pub size_bytes: i64,
    pub indexer: String,
}
```

```rust
// src/search/prowlarr.rs
impl ProwlarrClient {
    pub async fn search(&self, query: &str, media_type: &str) -> anyhow::Result<Vec<ReleaseCandidate>> {
        let response = self.http
            .get(format!("{}/api/v1/search", self.base_url))
            .header("X-Api-Key", &self.api_key)
            .query(&[("query", query), ("type", media_type)])
            .send()
            .await?
            .error_for_status()?;

        let items: Vec<ProwlarrItem> = response.json().await?;
        Ok(items.into_iter().map(ReleaseCandidate::from).collect())
    }
}
```

- [ ] **Step 4: Run the Prowlarr test**

Run: `cargo test normalizes_prowlarr_results_into_release_candidates -- --nocapture`

Expected: PASS with one normalized `ReleaseCandidate`.

- [ ] **Step 5: Commit Prowlarr search support**

```bash
git add src/domain/search.rs src/search src/workers/search_worker.rs tests/search_prowlarr.rs Cargo.toml
git commit -m "feat: search prowlarr and normalize candidates"
```

## Task 6: Add Direct Torznab Search Support

**Files:**
- Create: `src/search/torznab.rs`
- Modify: `src/search/mod.rs`
- Modify: `src/config.rs`
- Test: `tests/search_torznab.rs`

- [ ] **Step 1: Write the failing direct-indexer normalization test**

```rust
// tests/search_torznab.rs
use book_router::search::torznab::TorznabClient;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn parses_torznab_rss_items_into_release_candidates() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(r#"
            <rss><channel><item>
                <guid>torznab-1</guid>
                <title>The Hobbit EPUB</title>
                <size>2048</size>
            </item></channel></rss>
        "#, "application/xml"))
        .mount(&server)
        .await;

    let client = TorznabClient::new(server.uri(), None);
    let items = client.search("The Hobbit").await.unwrap();

    assert_eq!(items[0].source, "torznab");
    assert_eq!(items[0].title, "The Hobbit EPUB");
}
```

- [ ] **Step 2: Run the Torznab test**

Run: `cargo test parses_torznab_rss_items_into_release_candidates -- --nocapture`

Expected: FAIL with missing `TorznabClient` or XML parser.

- [ ] **Step 3: Implement the Torznab adapter behind the same search trait**

```rust
// src/search/torznab.rs
pub async fn search(&self, query: &str) -> anyhow::Result<Vec<ReleaseCandidate>> {
    let response = self.http
        .get(format!("{}/api", self.base_url))
        .query(&[("t", "search"), ("q", query)])
        .send()
        .await?
        .error_for_status()?;

    let xml = response.text().await?;
    let feed: TorznabFeed = quick_xml::de::from_str(&xml)?;
    Ok(feed.channel.items.into_iter().map(ReleaseCandidate::from).collect())
}
```

- [ ] **Step 4: Run the Torznab test**

Run: `cargo test parses_torznab_rss_items_into_release_candidates -- --nocapture`

Expected: PASS with one normalized `ReleaseCandidate`.

- [ ] **Step 5: Commit direct indexer support**

```bash
git add src/search/torznab.rs src/search/mod.rs src/config.rs tests/search_torznab.rs Cargo.toml
git commit -m "feat: add direct torznab search adapter"
```

## Task 7: Implement Confidence Scoring, Review Queue, And Candidate Explanations

**Files:**
- Create: `src/matcher.rs`
- Modify: `src/domain/search.rs`
- Modify: `src/db/repositories.rs`
- Create: `migrations/0002_search_and_sync.sql`
- Modify: `src/http/handlers/search.rs`
- Test: `tests/matcher_scores.rs`

- [ ] **Step 1: Write the failing matcher test for high-confidence and review-queue outcomes**

```rust
// tests/matcher_scores.rs
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
```

- [ ] **Step 2: Run the matcher test**

Run: `cargo test graphic_audio_preference_penalizes_plain_audio_release -- --nocapture`

Expected: FAIL with missing matcher functions or missing test helpers.

- [ ] **Step 3: Implement weighted scoring, hard filters, and explanation strings**

```rust
// src/matcher.rs
pub fn score_candidate(request: &RequestRecord, candidate: &ReleaseCandidate) -> ScoredCandidate {
    let mut score = 0.0;
    let mut explanation = Vec::new();

    if normalized_eq(&request.title, &candidate.title) {
        score += 0.45;
        explanation.push("title matched".into());
    }
    if candidate.title.to_lowercase().contains(&request.author.to_lowercase()) {
        score += 0.25;
        explanation.push("author matched".into());
    }
    if request.preferences.graphic_audio && !candidate.title.to_lowercase().contains("graphicaudio") {
        score -= 0.30;
        explanation.push("graphic audio requested but candidate does not advertise it".into());
    }

    ScoredCandidate { score, explanation, auto_acquire: score >= 0.90 }
}
```

- [ ] **Step 4: Run the matcher test and request-detail handler tests**

Run: `cargo test graphic_audio_preference_penalizes_plain_audio_release -- --nocapture`

Expected: PASS with a sub-threshold score and a readable explanation.

- [ ] **Step 5: Commit the matcher and review queue**

```bash
git add src/matcher.rs src/domain/search.rs src/db/repositories.rs src/http/handlers/search.rs migrations/0002_search_and_sync.sql tests/matcher_scores.rs
git commit -m "feat: score candidates and queue uncertain matches for review"
```

## Task 8: Dispatch Approved Candidates To qBittorrent And Track Download Completion

**Files:**
- Create: `src/downloads/mod.rs`
- Create: `src/downloads/qbittorrent.rs`
- Modify: `src/workers/download_worker.rs`
- Modify: `src/workers/mod.rs`
- Modify: `src/config.rs`
- Test: `tests/qbittorrent_client.rs`

- [ ] **Step 1: Write the failing qBittorrent client test**

```rust
// tests/qbittorrent_client.rs
use book_router::downloads::qbittorrent::QbittorrentClient;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn submits_magnet_with_request_tag_and_category() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v2/torrents/add"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = QbittorrentClient::new(server.uri(), "admin", "adminadmin");
    client.add_magnet("magnet:?xt=urn:btih:test", "request-123", "audiobooks").await.unwrap();
}
```

- [ ] **Step 2: Run the qBittorrent test**

Run: `cargo test submits_magnet_with_request_tag_and_category -- --nocapture`

Expected: FAIL with missing client implementation.

- [ ] **Step 3: Implement qBittorrent add/list/complete operations**

```rust
// src/downloads/qbittorrent.rs
pub async fn add_magnet(&self, magnet: &str, request_id: &str, category: &str) -> anyhow::Result<()> {
    let form = [
        ("urls", magnet),
        ("category", category),
        ("tags", request_id),
    ];

    self.http
        .post(format!("{}/api/v2/torrents/add", self.base_url))
        .form(&form)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}
```

- [ ] **Step 4: Run the qBittorrent test**

Run: `cargo test submits_magnet_with_request_tag_and_category -- --nocapture`

Expected: PASS with the mock endpoint receiving the add request.

- [ ] **Step 5: Commit qBittorrent dispatch support**

```bash
git add src/downloads src/workers/download_worker.rs src/workers/mod.rs src/config.rs tests/qbittorrent_client.rs Cargo.toml
git commit -m "feat: dispatch approved candidates to qbittorrent"
```

## Task 9: Build The Import Classifier, Move Planner, And Evented Import Worker

**Files:**
- Create: `src/importer/mod.rs`
- Create: `src/importer/classify.rs`
- Create: `src/importer/move_plan.rs`
- Modify: `src/domain/imports.rs`
- Modify: `src/workers/import_worker.rs`
- Test: `tests/import_classifier.rs`

- [ ] **Step 1: Write the failing import-classification test**

```rust
// tests/import_classifier.rs
use book_router::importer::classify::classify_payload;

#[test]
fn m4b_folder_is_classified_as_audiobook() {
    let classification = classify_payload(&[
        "The Hobbit/part01.m4b".into(),
        "The Hobbit/cover.jpg".into(),
    ]);

    assert_eq!(classification.media_type.as_str(), "audiobook");
}
```

```rust
// tests/import_classifier.rs
#[test]
fn epub_file_is_classified_as_ebook() {
    let classification = classify_payload(&["The Hobbit.epub".into()]);
    assert_eq!(classification.media_type.as_str(), "ebook");
}
```

- [ ] **Step 2: Run the import-classification tests**

Run: `cargo test m4b_folder_is_classified_as_audiobook epub_file_is_classified_as_ebook -- --nocapture`

Expected: FAIL with missing import classifier.

- [ ] **Step 3: Implement payload classification and deterministic move planning**

```rust
// src/importer/classify.rs
pub fn classify_payload(files: &[String]) -> Classification {
    let has_audio = files.iter().any(|file| file.ends_with(".m4b") || file.ends_with(".mp3") || file.ends_with(".flac"));
    let has_ebook = files.iter().any(|file| file.ends_with(".epub") || file.ends_with(".pdf") || file.ends_with(".azw3"));

    match (has_audio, has_ebook) {
        (true, false) => Classification::audiobook(),
        (false, true) => Classification::ebook(),
        (true, true) => Classification::mixed(),
        (false, false) => Classification::invalid("no supported media files"),
    }
}
```

```rust
// src/importer/move_plan.rs
pub fn build_move_plan(root: &std::path::Path, author: &str, work: &str, leaf_name: &str) -> std::path::PathBuf {
    root.join(author).join(work).join(leaf_name)
}
```

- [ ] **Step 4: Run the import-classification tests**

Run: `cargo test m4b_folder_is_classified_as_audiobook epub_file_is_classified_as_ebook -- --nocapture`

Expected: PASS for both ebook and audiobook cases.

- [ ] **Step 5: Commit the import pipeline foundation**

```bash
git add src/importer src/domain/imports.rs src/workers/import_worker.rs tests/import_classifier.rs
git commit -m "feat: classify imports and plan file moves"
```

## Task 10: Add Audiobookshelf And Calibre Sync Adapters

**Files:**
- Create: `src/sync/mod.rs`
- Create: `src/sync/audiobookshelf.rs`
- Create: `src/sync/calibre.rs`
- Modify: `src/workers/sync_worker.rs`
- Modify: `src/workers/mod.rs`
- Test: `tests/sync_adapters.rs`

- [ ] **Step 1: Write the failing sync-adapter tests**

```rust
// tests/sync_adapters.rs
use book_router::sync::{audiobookshelf::AudiobookshelfClient, calibre::CalibreHook};

#[tokio::test]
async fn audiobookshelf_scan_request_uses_api_key_auth() {
    let client = AudiobookshelfClient::new("http://localhost:13378", "abs-key");
    let request = client.scan_library("library-1").build().unwrap();

    assert_eq!(request.headers()["Authorization"], "Bearer abs-key");
}
```

```rust
// tests/sync_adapters.rs
#[test]
fn calibre_hook_builds_import_command() {
    let hook = CalibreHook::new("/usr/bin/calibredb");
    let command = hook.add_book_command("/ebooks/Tolkien/The Hobbit/The Hobbit.epub");

    assert_eq!(command.get_program().to_string_lossy(), "/usr/bin/calibredb");
}
```

- [ ] **Step 2: Run the sync-adapter tests**

Run: `cargo test audiobookshelf_scan_request_uses_api_key_auth calibre_hook_builds_import_command -- --nocapture`

Expected: FAIL with missing sync adapters.

- [ ] **Step 3: Implement the outbound sync adapters**

```rust
// src/sync/audiobookshelf.rs
pub fn scan_library(&self, library_id: &str) -> reqwest::RequestBuilder {
    self.http
        .post(format!("{}/api/libraries/{library_id}/scan", self.base_url))
        .bearer_auth(&self.api_key)
}
```

```rust
// src/sync/calibre.rs
pub fn add_book_command(&self, path: &str) -> std::process::Command {
    let mut command = std::process::Command::new(&self.binary_path);
    command.arg("add").arg(path);
    command
}
```

- [ ] **Step 4: Run the sync-adapter tests**

Run: `cargo test audiobookshelf_scan_request_uses_api_key_auth calibre_hook_builds_import_command -- --nocapture`

Expected: PASS for both adapters.

- [ ] **Step 5: Commit sync adapter support**

```bash
git add src/sync src/workers/sync_worker.rs src/workers/mod.rs tests/sync_adapters.rs
git commit -m "feat: add calibre and audiobookshelf sync adapters"
```

## Task 11: Package The App For Docker, Add End-To-End Flow Coverage, And Commit The Planning Docs

**Files:**
- Create: `Dockerfile`
- Create: `compose.yml`
- Create: `tests/e2e_request_flow.rs`
- Modify: `.env.example`
- Modify: `Cargo.toml`
- Modify: `docs/superpowers/specs/2026-04-09-book-router-design.md`
- Modify: `docs/superpowers/plans/2026-04-09-book-router-v1.md`

- [ ] **Step 1: Write the failing end-to-end flow test**

```rust
// tests/e2e_request_flow.rs
#[tokio::test]
async fn request_to_import_flow_records_success_events() {
    let harness = crate::support::Harness::boot().await;

    let request_id = harness.create_request("The Hobbit", "J.R.R. Tolkien", "ebook").await;
    harness.enqueue_high_confidence_candidate(&request_id, "The Hobbit EPUB").await;
    harness.mark_download_complete(&request_id, vec!["The Hobbit.epub"]).await;

    let events = harness.request_events(&request_id).await;
    assert!(events.iter().any(|event| event.kind == "import.succeeded"));
}
```

- [ ] **Step 2: Run the end-to-end test and confirm the harness is still missing**

Run: `cargo test request_to_import_flow_records_success_events -- --nocapture`

Expected: FAIL with missing e2e harness support.

- [ ] **Step 3: Add Docker packaging, compose wiring, and the e2e harness support**

```dockerfile
# Dockerfile
FROM rust:1.87 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/book_router /usr/local/bin/book_router
CMD ["book_router"]
```

```yaml
# compose.yml
services:
  app:
    build: .
    volumes:
      - ./data:/data
      - ./config:/config
      - ./media/ebooks:/ebooks
      - ./media/audiobooks:/audiobooks
```

- [ ] **Step 4: Run the end-to-end test suite and initialize the repository commit history**

Run: `cargo test --test e2e_request_flow -- --nocapture`

Expected: PASS once the harness boots the app, records events, and sees `import.succeeded`.

Run: `git init`

Expected: PASS with a new `.git/` directory created in the project root.

- [ ] **Step 5: Commit the design and plan documents plus the packaging layer**

```bash
git add Dockerfile compose.yml docs/superpowers/specs/2026-04-09-book-router-design.md docs/superpowers/plans/2026-04-09-book-router-v1.md
git commit -m "docs: add design and implementation plan"
```

## Spec Coverage Check

- Product goals: covered by Tasks 1 through 11, with the end-to-end flow in Task 11 verifying the request-to-import path.
- SQLite default: Task 2 establishes SQLite persistence and migration flow, and the revised immediate milestone adds persistent file-backed startup.
- Open Library primary metadata: Task 3 implements the primary metadata provider.
- Admin-only v1 UI/API: Task 4 creates the initial scaffold, and Task 4A revises it into the approved metadata-first request flow.
- Prowlarr plus direct indexers: Tasks 5 and 6 implement both search paths.
- Confidence scoring and review queue: Task 7 implements weighted scoring and explanation strings.
- qBittorrent first: Task 8 implements the first download client.
- Full import ownership: Task 9 implements classification and file-move planning.
- Calibre and Audiobookshelf integrations: Task 10 implements the outbound adapters.
- Docker-first deployment and verification: Task 11 adds packaging and end-to-end coverage.

## Self-Review

- Placeholder scan: no `TODO`, `TBD`, or deferred implementation steps remain inside the numbered tasks.
- Type consistency: `CreateRequest`, `ManifestationPreference`, `ReleaseCandidate`, and the adapter names are reused consistently across tasks.
- Scope check: the plan remains within the approved `v1` scope and intentionally does not include public accounts, first-class Discord bots, or NZB client implementation.
