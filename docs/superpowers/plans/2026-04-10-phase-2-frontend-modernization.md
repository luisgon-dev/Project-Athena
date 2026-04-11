# Phase 2: Frontend Modernization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace Askama templates with a SvelteKit SPA using shadcn-svelte, Lucide icons, and automated type-sharing via `ts-rs`.

**Architecture:** SvelteKit in SPA mode, `ts-rs` for type-safety, Axum serving static assets and JSON API.

**Tech Stack:** Svelte 5, TypeScript, Tailwind CSS v4, shadcn-svelte, Lucide Svelte, Rust, Axum, ts-rs.

---

## Task 1: Integrate `ts-rs` for Type-Sharing

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/domain/catalog.rs`
- Modify: `src/domain/requests.rs`
- Modify: `src/domain/events.rs`
- Modify: `src/http/handlers/requests.rs`
- Create: `tests/export_types.rs`

- [ ] **Step 1: Add `ts-rs` to `Cargo.toml`**

Add `ts-rs = "10"` to dependencies.

- [ ] **Step 2: Annotate domain models with `#[derive(TS)]`**

Annotate all public structs that are returned by or sent to the API (e.g., `WorkRecord`, `RequestRecord`, `MediaType`, `ManifestationPreference`).

- [ ] **Step 3: Create the type export test**

```rust
// tests/export_types.rs
use book_router::domain::requests::{RequestRecord, MediaType}; // and others
use ts_rs::TS;

#[test]
fn export_types() {
    RequestRecord::export_all_to("frontend/src/lib/types/bindings.ts").unwrap();
    // Repeat for all key types
}
```

- [ ] **Step 4: Run the test to generate initial types**

Run: `cargo test --test export_types`
Expected: PASS, and file created.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml src/domain tests/export_types.rs
git commit -m "feat: integrate ts-rs for automated type sharing"
```

## Task 2: Scaffold SvelteKit Frontend

**Files:**
- Create: `frontend/` (full project)

- [ ] **Step 1: Initialize SvelteKit project**

Run: `npm create svelte@latest frontend` (Select: Skeleton project, TypeScript, Svelte 5, Tailwind CSS).

- [ ] **Step 2: Install dependencies**

Run: `npm i -D @sveltejs/adapter-static lucide-svelte shadcn-svelte tailwind-merge clsx`

- [ ] **Step 3: Configure SPA mode**

Update `frontend/svelte.config.js` to use `adapter-static` with `fallback: 'index.html'`. Create `frontend/src/routes/+layout.ts` with `export const ssr = false; export const prerender = true;`.

- [ ] **Step 4: Commit scaffold**

```bash
git add frontend
git commit -m "chore: scaffold sveltekit spa frontend"
```

## Task 3: Migrate Backend to JSON API (/api/v1/)

**Files:**
- Modify: `src/app.rs`
- Modify: `src/http/handlers/requests.rs`
- Modify: `src/http/handlers/covers.rs`
- Modify: `src/http/error.rs`

- [ ] **Step 1: Refactor handlers to return `Json<T>`**

Replace `Html` with `Json` in all handlers. Update handlers to return domain models directly (ensure they derive `Serialize`).

- [ ] **Step 2: Update Axum Router with `/api/v1/` prefix and fallback**

```rust
// src/app.rs
let api_router = Router::new()
    .route("/health", get(health))
    .route("/requests", get(requests_index).post(create_request))
    // ... other routes ...
    .with_state(state.clone());

let frontend_service = ServeDir::new("frontend/build").fallback(ServeFile::new("frontend/build/index.html"));

Router::new()
    .nest("/api/v1", api_router)
    .fallback_service(frontend_service)
```

- [ ] **Step 3: Update `AppError` to return JSON**

Ensure `AppError` always returns a JSON payload even for HTML-targeted requests (as we are now an SPA).

- [ ] **Step 4: Verify with tests**

Update `tests/request_http.rs` to expect JSON responses instead of HTML.

- [ ] **Step 5: Commit**

```bash
git add src/app.rs src/http
git commit -m "feat: migrate to JSON API with /api/v1 prefix"
```

## Task 4: Build Basic UI (Layout & Dashboard)

**Files:**
- Create: `frontend/src/routes/+layout.svelte`
- Create: `frontend/src/routes/+page.svelte`

- [ ] **Step 1: Implement global layout with Lucide icons**
- [ ] **Step 2: Implement request list dashboard using shadcn components**
- [ ] **Step 3: Commit**

## Task 5: Implement Metadata-First Request Wizard

**Files:**
- Create: `frontend/src/routes/requests/new/+page.svelte`

- [ ] **Step 1: Port the search and work selection logic to Svelte components**
- [ ] **Step 2: Implement the creation form with manifestation preferences**
- [ ] **Step 3: Commit**

## Task 6: Implement Request Detail Page

**Files:**
- Create: `frontend/src/routes/requests/[id]/+page.svelte`

- [ ] **Step 1: Display request status and event history**
- [ ] **Step 2: Commit**

## Task 7: Update Dockerfile for Multi-Stage Build

**Files:**
- Modify: `Dockerfile`

- [ ] **Step 1: Refactor Dockerfile to build frontend, then backend, then bundle both**
- [ ] **Step 2: Verify build**

Run: `docker build -t project-athena .`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add Dockerfile
git commit -m "chore: update Dockerfile for multi-stage frontend/backend build"
```

## Task 8: Final Cleanup

**Files:**
- Delete: `templates/`
- Delete: `src/http/views.rs`

- [ ] **Step 1: Remove old Askama dependencies and unused code**
- [ ] **Step 2: Verify everything still works**

Run: `cargo test`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git rm -r templates
git add src/http/mod.rs Cargo.toml
git commit -m "chore: cleanup old Askama templates and views"
```
