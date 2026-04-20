# Athena Frontend

This directory contains the SvelteKit SPA for Project Athena.

## What lives here

- route-level UI for requests, review, settings, and library scan
- the typed API client under `src/lib/api.ts`
- browser component tests with Vitest
- Playwright browser tests in two layers:
  - mocked browser coverage under `e2e/mocked`
  - live full-stack coverage under `e2e/fullstack`

## Key commands

```bash
npm ci
npm run dev
npm run check
npm run test:unit -- --run
npm run test:e2e:mocked
npm run test:e2e:fullstack
```

## Notes

- `npm run dev` uses the Vite proxy in `vite.config.ts` to send `/api` calls to `http://127.0.0.1:3000`.
- `npm run build` generates the SPA into `frontend/build`, which the Rust backend serves in production-style runs.
- The full-stack Playwright suite starts:
  - a local Open Library fixture server
  - the real Rust backend with the built frontend mounted

For overall project setup and operational guidance, use the root `README.md`.
