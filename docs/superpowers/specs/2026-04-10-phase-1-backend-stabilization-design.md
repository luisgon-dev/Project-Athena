# Phase 1: Backend Stabilization Design

**Date:** 2026-04-10
**Status:** Approved
**Topic:** Stabilizing the Rust backend and improving error handling for external dependencies.

## Summary

This design outlines the first phase of the architecture modernization for Project Athena. The primary goal is to address the application's current fragility by stabilizing external API integrations, decoupling startup from network-dependent tasks, and implementing structured error handling. This foundation is required before moving to a separated frontend in Phase 2.

## Problem Statement

The application currently errors out ungracefully under several conditions:
1. **Startup Blocking:** The app performs synchronous metadata backfilling during startup. If the Open Library API is unreachable, the server fails to boot.
2. **Unstructured Errors:** Failures in external API calls (e.g., searches, work resolution) result in generic 500 or 502 errors without clear feedback to the user or logs.
3. **Missing Timeouts:** Lack of explicit timeouts on `reqwest` calls can lead to hung threads if an external service is slow but responsive.
4. **Tight Coupling:** The UI (Askama templates) is tightly coupled to the result of these fragile calls, making error presentation difficult.

## Architecture & Components

### 1. Resilient `OpenLibraryClient`

The `OpenLibraryClient` will be refactored to be more robust and provide better diagnostic information.

- **Timeouts:** A default 10-second timeout will be applied to all HTTP requests via the `reqwest` client.
- **Structured Error Enum:** A custom `OpenLibraryError` enum will replace `anyhow::Result` for public methods.
  ```rust
  pub enum OpenLibraryError {
      Timeout(reqwest::Error),
      Network(reqwest::Error),
      ApiError(reqwest::StatusCode, String),
      Serialization(serde_json::Error),
      NoMatch,
  }
  ```
- **Logging:** Every failed call will be logged with structured metadata (URL, status code, error type).

### 2. Background Worker System (In-Process)

Startup and periodic tasks will be moved out of the main request path to prevent blocking.

- **Decoupled Startup:** The `backfill_legacy_request_work_ids` function will be moved from `build_app` into a background `tokio::spawn` task.
- **Worker Loop:** A simple worker loop will run in the background, polling the database for "unresolved" requests and attempting to backfill them with a jittered exponential backoff (e.g., 30s, 2m, 5m, 15m).
- **Graceful Boot:** The Axum server will start immediately after database migrations, regardless of whether external APIs are reachable.

### 3. Structured API Error Normalization

Axum handlers will move away from returning generic `StatusCode` or `anyhow::Result`.

- **`AppError` Enum:** A centralized error type that implements `IntoResponse`.
  ```rust
  pub enum AppError {
      MetadataFailure(OpenLibraryError),
      DatabaseFailure(sqlx::Error),
      NotFound(String),
      BadRequest(String),
  }
  ```
- **JSON & HTML Error Support:** `AppError` will detect the `Accept` header (or follow a dedicated API route convention) to return either a JSON error payload or an Askama error template.

## Data Flow

1. **Request Lifecycle:**
   - User submits a search/request.
   - Handler calls `OpenLibraryClient`.
   - If a timeout or API error occurs, `OpenLibraryClient` returns a structured `OpenLibraryError`.
   - Handler maps this to an `AppError`.
   - `AppError` implementation of `IntoResponse` renders the error (JSON or HTML) to the user.

2. **Background Backfill Lifecycle:**
   - App boots and spawns `BackfillWorker`.
   - `BackfillWorker` queries DB for `external_work_id = ''`.
   - For each row, it calls `OpenLibraryClient::resolve_work`.
   - If it fails, the worker logs the error and moves to the next row, scheduling a retry for the failed one.
   - If it succeeds, it updates the DB and records a `backfill.succeeded` event.

## Error Handling Strategy

- **Retry Policy:** Background workers will retry with exponential backoff.
- **User Feedback:** The UI will present specific error messages (e.g., "Metadata service is currently slow, please try again in a few minutes") instead of "Internal Server Error".
- **Validation:** All inputs will be validated before external calls are made to reduce unnecessary API traffic.

## Testing Strategy

- **Integration Tests (Wiremock):**
  - Verify `OpenLibraryClient` handles timeouts correctly.
  - Verify `OpenLibraryClient` handles 5xx and 4xx responses gracefully.
  - Verify `OpenLibraryClient` handles malformed JSON without panicking.
- **Unit Tests:**
  - Verify `AppError` renders correct status codes and payloads.
  - Verify `BackfillWorker` logic correctly identifies and updates pending rows.
- **Smoke Tests:**
  - Verify the app boots without internet access and responds to `/health`.

## Future Considerations (Phase 2)

- This stabilization ensures that when we move to a React/SPA frontend, the API is already robust and returns structured JSON errors that the SPA can handle using standardized "Error Boundary" patterns.
