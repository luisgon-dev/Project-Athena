# Project Athena

Project Athena is a request-driven acquisition console for ebooks and audiobooks. It resolves works against Open Library, tracks requests in SQLite, searches configured acquisition sources, dispatches approved candidates to qBittorrent, imports completed payloads into managed media roots, and triggers downstream sync hooks for Calibre or Audiobookshelf.

The backend is a Rust Axum service. The frontend is a SvelteKit SPA that is built into `frontend/build` and served by the Rust app in production-style runs.

## Core flow

1. Search Open Library metadata from the request wizard.
2. Create one or more requests from a canonical work record.
3. Let Athena search Prowlarr and synced Torznab/Newznab indexers.
4. Auto-acquire high-confidence matches or review candidates manually.
5. Dispatch approved downloads to qBittorrent.
6. Import completed files into the configured ebook or audiobook roots.
7. Trigger Calibre or Audiobookshelf sync once import succeeds.

## Local development

### Prerequisites

- Rust toolchain with `cargo`
- Node.js 22+ and `npm`
- Playwright browsers for browser tests
- Optional: Docker/Compose for higher-fidelity acceptance runs
- Optional: `calibredb` if you want to exercise the real ebook sync hook outside automated tests

### Install frontend dependencies

```bash
cd frontend
npm ci
```

### Run the frontend only

This is useful for UI work against an existing backend or for mocked browser tests.

```bash
cd frontend
npm run dev
```

The Vite dev server proxies `/api` requests to `http://127.0.0.1:3000`.

### Run the backend only

From the repo root:

```bash
cargo run
```

Default boot values:

- `BIND_ADDR=127.0.0.1:3000`
- `EBOOKS_ROOT=/ebooks`
- `AUDIOBOOKS_ROOT=/audiobooks`
- `DATABASE_PATH=/data/book-router/book-router.sqlite`
- `METADATA_BASE_URL=https://openlibrary.org`
- `COVER_BASE_URL=https://covers.openlibrary.org`
- `ENABLE_FULFILLMENT_WORKERS=true`

### Run the production-style full stack locally

This builds the SPA and lets the Rust backend serve it directly.

```bash
cd frontend
npm run build
cd ..
cargo run
```

## Container image

Athena builds into a single image that contains:

- the compiled Rust backend
- the built Svelte frontend
- an entrypoint that supports Unraid-style `PUID` / `PGID` execution

### Build locally

```bash
docker build -t project-athena:local .
```

### Run locally

```bash
docker run --rm -p 3001:3000 \
  -e TZ=America/Los_Angeles \
  -e PUID=1000 \
  -e PGID=1000 \
  -v "$(pwd)/data:/config" \
  -v "$(pwd)/media/ebooks:/ebooks" \
  -v "$(pwd)/media/audiobooks:/audiobooks" \
  project-athena:local
```

Container defaults:

- `BIND_ADDR=0.0.0.0:3000`
- `DATABASE_PATH=/config/book-router.sqlite`
- `EBOOKS_ROOT=/ebooks`
- `AUDIOBOOKS_ROOT=/audiobooks`
- `UMASK=002`

### GHCR publishing

The workflow in [.github/workflows/publish-image.yml](.github/workflows/publish-image.yml) publishes the image to GitHub Container Registry on pushes to `main`, matching `v*` tags, and manual dispatches.

Published image path:

- `ghcr.io/<github-owner>/project-athena:latest`

For Unraid-specific setup, see [docs/unraid.md](docs/unraid.md).

## Runtime configuration model

Athena has two layers of configuration:

- Bootstrap env vars in `AppConfig`
- Persisted runtime settings stored in SQLite

Bootstrap env vars seed the runtime settings on first boot. After that, the UI writes to the persisted settings records, and request/search/import/sync flows read those persisted values at runtime.

That means:

- changing env vars is most relevant on first boot or when pointing Athena at a new database
- day-to-day operational changes should be made through the settings UI

The main operational surfaces are:

- `/settings/runtime`
- `/settings/storage`
- `/settings/import`
- `/settings/acquisition`
- `/settings/download-clients/qbittorrent`
- `/settings/integrations/prowlarr`
- `/settings/integrations/audiobookshelf`

## Validation commands

From the repo root:

```bash
cargo test
```

From `frontend/`:

```bash
npm run check
npm run test:unit -- --run
npm run test:e2e:mocked
npm run test:e2e:fullstack
```

Or run the combined frontend browser coverage:

```bash
cd frontend
npm run test:e2e
```

### Browser test split

- `test:e2e:mocked` runs fast, deterministic Playwright coverage against mocked `/api/v1` responses.
- `test:e2e:fullstack` starts a local Open Library fixture and the real Rust backend, then exercises the built frontend through the live app.

The full-stack suite disables fulfillment workers with `ENABLE_FULFILLMENT_WORKERS=false` so request-state assertions stay deterministic.

## Acceptance environment

For the optional higher-fidelity stack:

```bash
docker compose --profile acceptance up --build
```

See [docs/acceptance/compose-profile.md](docs/acceptance/compose-profile.md) for setup details and first-run expectations.

## Troubleshooting

### Browser tests cannot start their web server

If Playwright reports that its web server failed to start, check:

- whether another local process is already bound to the configured port
- whether your environment allows local port binding
- whether Playwright browsers are installed

Install browsers if needed:

```bash
cd frontend
npx playwright install chromium
```

### The Rust app serves a minimal shell instead of the real SPA

If the backend serves a bare fallback shell, `frontend/build/index.html` is missing. Rebuild the frontend first:

```bash
cd frontend
npm run build
```

### Request behavior does not match env vars after the first boot

The persisted runtime settings may already be seeded in SQLite. Either update settings through the UI or start with a fresh database if you want bootstrap env vars to be applied again.
