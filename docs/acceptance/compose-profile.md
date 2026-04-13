# Acceptance Compose Profile

The default `docker compose up` flow still starts only `book-router`.

For a higher-fidelity acceptance environment, start the optional profile:

```bash
docker compose --profile acceptance up --build
```

That adds:

- `qbittorrent` on `http://localhost:8080`
- `prowlarr` on `http://localhost:9696`
- `audiobookshelf` on `http://localhost:13378`
- `torznab-fixture` on `http://localhost:8088`

## Expected usage

1. Start the stack with the acceptance profile.
2. Open Athena at `http://localhost:3001`.
3. Save runtime settings in the UI:
   - qBittorrent base URL: `http://qbittorrent:8080` when running inside Compose, or `http://localhost:8080` when testing from the host.
   - Prowlarr base URL: `http://prowlarr:9696` or `http://localhost:9696`.
   - Audiobookshelf base URL: `http://audiobookshelf` or `http://localhost:13378`.
4. Point any direct Torznab-style test indexer to `http://torznab-fixture:8080` or `http://localhost:8088`.
5. Create an ebook or audiobook request and drive it through the review or auto-acquire flow.

## Notes

- The Torznab fixture is static on purpose. It gives Athena a deterministic search response without relying on a live public indexer.
- qBittorrent, Prowlarr, and Audiobookshelf still need their normal first-run setup inside their own UIs.
- Acceptance data is stored under `./data/acceptance/`.
