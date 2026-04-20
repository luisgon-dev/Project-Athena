# Acceptance Compose Profile

The default `docker compose up` flow starts only `book-router`. Use the optional acceptance profile when you want Athena plus its neighboring services available together.

## Start the stack

```bash
docker compose --profile acceptance up --build
```

This adds:

- `book-router` at `http://localhost:3001`
- `qbittorrent` at `http://localhost:8080`
- `prowlarr` at `http://localhost:9696`
- `audiobookshelf` at `http://localhost:13378`
- `torznab-fixture` at `http://localhost:8088`

Acceptance data is stored under `./data/acceptance/`.

## First-run setup

### Athena

Open `http://localhost:3001` and save the runtime settings through the UI.

Recommended values from the host:

- qBittorrent base URL: `http://localhost:8080`
- Prowlarr base URL: `http://localhost:9696`
- Audiobookshelf base URL: `http://localhost:13378`
- Torznab-style direct indexer base URL: `http://localhost:8088`

If you are configuring Athena from inside the Compose network instead of from the host browser, use:

- `http://qbittorrent:8080`
- `http://prowlarr:9696`
- `http://audiobookshelf`
- `http://torznab-fixture:8080`

### qBittorrent

- Complete the normal WebUI first-run flow.
- Ensure the WebUI is reachable before testing Athena’s qBittorrent connection.

### Prowlarr

- Complete any initial setup required by Prowlarr.
- Generate an API key and save it into Athena’s Prowlarr settings page.
- If you want synced indexers to appear in Athena, enable Athena’s application-sync-compatible settings and push indexers from Prowlarr.

### Audiobookshelf

- Complete first-run setup and create or identify the target library.
- Save the base URL, API key, and library ID into Athena’s Audiobookshelf settings page.

## Recommended acceptance flow

1. Start the acceptance stack.
2. Save Athena runtime settings for qBittorrent, Prowlarr, and Audiobookshelf.
3. Verify each service connection from Athena’s settings pages.
4. Create a request from the metadata wizard.
5. Drive the request through review or auto-acquire.
6. Verify qBittorrent dispatch, import completion, and downstream sync behavior.

## Deterministic fixture usage

The Torznab fixture is intentionally static. Use it when you want deterministic search responses without depending on a live public indexer.

## Known limitations

- qBittorrent, Prowlarr, and Audiobookshelf still require their own normal first-run setup.
- Compose acceptance is a manual or higher-fidelity workflow; it is not the fast default verification path.
- The frontend Playwright full-stack suite uses a lightweight local Open Library fixture, not this Compose profile.
