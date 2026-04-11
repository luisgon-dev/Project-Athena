# Phase 2: Frontend Modernization (Svelte SPA) Design

**Date:** 2026-04-10
**Status:** Approved
**Topic:** Transitioning from Askama templates to a SvelteKit SPA with automated type-sharing.

## Summary

This design outlines the second phase of Project Athena's modernization. The goal is to replace the current Server-Side Rendered (SSR) Askama templates with a modern, high-performance Single Page Application (SPA) built with Svelte 5 and SvelteKit. We will implement automated type-sharing between the Rust backend and the TypeScript frontend using the `ts-rs` crate to ensure a robust and fast developer experience.

## Architecture

### 1. Frontend Structure

- **Directory:** All frontend code will reside in a new `/frontend` directory in the project root.
- **Framework:** SvelteKit in **SPA mode** (using `@sveltejs/adapter-static` with `ssr = false`).
- **Styling:** Tailwind CSS v4 with **shadcn-svelte** for a modern, accessible UI component library.
- **Icons:** **Lucide Svelte**.
- **State Management:** Svelte 5 **Runes** for reactive state and Svelte stores where appropriate.

### 2. API & Type Safety

- **API Prefix:** All backend routes will be migrated from `/requests` to `/api/v1/requests`.
- **Type Sharing (`ts-rs`):** Rust structs (e.g., `RequestRecord`, `CreateRequest`) will be annotated with `#[derive(TS)]`. Running `cargo test` will automatically export these as TypeScript interfaces into `frontend/src/lib/types/bindings.ts`.
- **Client:** A lightweight `fetch`-based wrapper in TypeScript will use these generated types for end-to-end type safety.

### 3. Production & Build Process

- **Multi-Stage Docker Build:**
  - **Stage 1 (Frontend):** Build the SvelteKit SPA into static assets in `/frontend/build`.
  - **Stage 2 (Backend):** Build the Rust Axum binary.
  - **Stage 3 (Final):** Copy the binary and the static assets. Axum will serve the assets using `tower_http::services::ServeDir`.
- **Routing:** Axum will handle `/api/*` routes and fallback to `index.html` for all other requests, allowing SvelteKit's client-side router to manage navigation.

## Components & Layout

- **`+layout.svelte`:** Main navigation, dark mode support, and global error boundaries.
- **`+page.svelte` (Index):** Dashboard and request list view using shadcn Data Tables.
- **`+page.svelte` (New Request):** A multi-step metadata-first request wizard.
- **`+page.svelte` (Request Details):** Detailed view of request status, events, and candidate review.

## Data Flow

1. **Development:**
   - Rust server runs on port 3000.
   - Vite dev server runs on port 5173, proxying `/api/*` to port 3000.
   - `ts-rs` ensures that any changes to Rust models are immediately reflected in the frontend's TypeScript environment.
2. **Production:**
   - User requests `app.local/`.
   - Axum serves `index.html`.
   - SvelteKit hydrates and fetches data from `app.local/api/v1/...`.
   - Errors are caught by SvelteKit error boundaries and presented to the user via shadcn Toast notifications.

## Testing Strategy

- **Frontend Unit Tests:** Using Vitest for component logic and state management.
- **E2E Tests:** Playwright to verify the complete flow from search to request creation.
- **Contract Verification:** Automated check to ensure the Rust API and TypeScript types are in sync during CI.

## Phase 2 Implementation Order

1. **Scaffold Frontend:** Initialize SvelteKit in `/frontend` and install dependencies (Tailwind, shadcn, Lucide).
2. **Type Bridge:** Integrate `ts-rs` into the Rust project and export initial models.
3. **API Migration:** Refactor Axum handlers to return JSON and move to `/api/v1/` prefix.
4. **Layout & Dashboard:** Build the base UI and the request list view.
5. **Metadata Wizard:** Port the "New Request" logic to a modern Svelte component.
6. **Dockerization:** Update the `Dockerfile` to handle the new multi-stage build.
7. **Cleanup:** Remove old Askama templates and unused handler logic.
