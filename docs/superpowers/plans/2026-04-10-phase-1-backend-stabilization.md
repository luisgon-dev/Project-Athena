# Phase 1: Backend Stabilization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stabilize the Rust backend by refactoring external API calls for resilience, decoupling startup from network tasks, and implementing structured error handling.

**Architecture:** Refactor `OpenLibraryClient` with structured errors and timeouts; implement a centralized `AppError` with Axum `IntoResponse`; move data backfilling into a background `tokio` task.

**Tech Stack:** Rust, Axum, Reqwest, Sqlx, Serde, Thiserror, Wiremock.

---

## Task 1: Refactor `OpenLibraryClient` for Resilience

**Files:**
- Modify: `src/metadata/openlibrary.rs`
- Test: `tests/openlibrary_client_resilience.rs`

- [ ] **Step 1: Define `OpenLibraryError` and update client constructor**

```rust
// src/metadata/openlibrary.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OpenLibraryError {
    #[error("Open Library request timed out")]
    Timeout(#[source] reqwest::Error),
    #[error("Network error communicating with Open Library")]
    Network(#[source] reqwest::Error),
    #[error("Open Library returned an API error: {0} - {1}")]
    ApiError(reqwest::StatusCode, String),
    #[error("Failed to parse Open Library response")]
    Serialization(#[source] serde_json::Error),
    #[error("No work match found in Open Library")]
    NoMatch,
    #[error("Work not found in Open Library")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, OpenLibraryError>;

impl OpenLibraryClient {
    pub fn new(base_url: impl Into<String>, covers_base_url: impl Into<String>) -> Self {
        let http = reqwest::Client::builder()
            .user_agent("book-router/0.1")
            .timeout(std::time::Duration::from_secs(10)) // Add 10s timeout
            .build()
            .expect("reqwest client");

        Self {
            base_url: base_url.into(),
            covers_base_url: covers_base_url.into(),
            http,
        }
    }
}
```

- [ ] **Step 2: Update all client methods to return `Result<T>`**

Update `search_works`, `resolve_work`, `resolve_work_by_id`, `fetch_cover`, and all private helper methods to use the new `OpenLibraryError` enum. Map `reqwest::Error` to `Timeout` or `Network` based on `is_timeout()`.

- [ ] **Step 3: Write resilience tests with Wiremock**

```rust
// tests/openlibrary_client_resilience.rs
use book_router::metadata::openlibrary::{OpenLibraryClient, OpenLibraryError};
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use std::time::Duration;

#[tokio::test]
async fn client_handles_timeout() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(11)))
        .mount(&server)
        .await;

    let client = OpenLibraryClient::new(server.uri(), server.uri());
    let result = client.search_works("title", "author").await;

    match result {
        Err(OpenLibraryError::Timeout(_)) => (),
        other => panic!("Expected Timeout, got {:?}", other),
    }
}
```

- [ ] **Step 4: Run tests and verify they pass**

Run: `cargo test --test openlibrary_client_resilience`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/metadata/openlibrary.rs tests/openlibrary_client_resilience.rs
git commit -m "refactor: add timeouts and structured errors to OpenLibraryClient"
```

## Task 2: Implement `AppError` and Handler Normalization

**Files:**
- Create: `src/http/error.rs`
- Modify: `src/http/mod.rs`
- Modify: `src/http/handlers/requests.rs`
- Modify: `src/app.rs`

- [ ] **Step 1: Create the `AppError` type**

```rust
// src/http/error.rs
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;
use crate::metadata::openlibrary::OpenLibraryError;

pub enum AppError {
    Metadata(OpenLibraryError),
    Database(sqlx::Error),
    NotFound(String),
    BadRequest(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::Metadata(OpenLibraryError::Timeout(_)) => (StatusCode::GATEWAY_TIMEOUT, "Metadata service timed out"),
            Self::Metadata(OpenLibraryError::NoMatch) => (StatusCode::NOT_FOUND, "No matching work found"),
            Self::Metadata(_) => (StatusCode::BAD_GATEWAY, "Error from metadata service"),
            Self::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
        };

        let body = Json(json!({ "error": message }));
        (status, body).into_response()
    }
}

impl From<OpenLibraryError> for AppError {
    fn from(inner: OpenLibraryError) -> Self { Self::Metadata(inner) }
}
// ... add other From impls
```

- [ ] **Step 2: Update `new_request` and `create_request` handlers**

Update the handlers in `src/http/handlers/requests.rs` to return `Result<_, AppError>` instead of `StatusCode` or generic `anyhow::Result`.

- [ ] **Step 3: Verify error responses with a test**

Modify `tests/request_http.rs` to check for specific JSON error payloads when search fails.

- [ ] **Step 4: Commit**

```bash
git add src/http/error.rs src/http/mod.rs src/http/handlers/requests.rs
git commit -m "feat: implement structured AppError and normalize handlers"
```

## Task 3: Decouple Startup from Backfilling

**Files:**
- Modify: `src/app.rs`
- Create: `src/workers/backfill.rs`
- Modify: `src/workers/mod.rs`

- [ ] **Step 1: Create `BackfillWorker`**

```rust
// src/workers/backfill.rs
pub struct BackfillWorker {
    pool: SqlitePool,
    open_library: OpenLibraryClient,
}

impl BackfillWorker {
    pub fn spawn(pool: SqlitePool, open_library: OpenLibraryClient) {
        let worker = Self { pool, open_library };
        tokio::spawn(async move {
            if let Err(e) = worker.run().await {
                tracing::error!(error = %e, "Backfill worker failed");
            }
        });
    }

    async fn run(self) -> anyhow::Result<()> {
        // Implementation of backfill loop with retry logic
        Ok(())
    }
}
```

- [ ] **Step 2: Update `build_app` to spawn the worker**

Remove the `await` on `backfill_legacy_request_work_ids` and instead call `BackfillWorker::spawn`.

- [ ] **Step 3: Commit**

```bash
git add src/app.rs src/workers/backfill.rs src/workers/mod.rs
git commit -m "refactor: decouple backfill from app startup"
```

## Task 4: Implement Exponential Backoff for Backfilling

**Files:**
- Modify: `src/workers/backfill.rs`

- [ ] **Step 1: Add retry loop with backoff**

```rust
// src/workers/backfill.rs
async fn run(self) -> anyhow::Result<()> {
    loop {
        match self.process_pending_backfills().await {
            Ok(count) if count > 0 => {
                 tracing::info!(count, "Backfilled requests");
                 tokio::time::sleep(Duration::from_secs(60)).await;
            }
            Ok(_) => {
                 tokio::time::sleep(Duration::from_secs(3600)).await; // Idle check
            }
            Err(e) => {
                 tracing::warn!(error = %e, "Backfill iteration failed, retrying in 5m");
                 tokio::time::sleep(Duration::from_secs(300)).await;
            }
        }
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src/workers/backfill.rs
git commit -m "feat: add retry and idle loop to backfill worker"
```

## Task 5: Smoke Test Offline Boot

**Files:**
- Create: `tests/offline_boot.rs`

- [ ] **Step 1: Create the smoke test**

```rust
// tests/offline_boot.rs
#[tokio::test]
async fn app_boots_with_invalid_metadata_url() {
    let mut config = AppConfig::for_tests();
    config.metadata_base_url = "http://invalid-metadata-url-that-does-not-exist.local".into();
    
    let app = build_app(config).await.expect("App should boot even if metadata is unreachable");
    // Verify /health works
}
```

- [ ] **Step 2: Run all tests to verify stability**

Run: `cargo test`
Expected: ALL PASS

- [ ] **Step 3: Commit**

```bash
git add tests/offline_boot.rs
git commit -m "test: verify app boots offline"
```
