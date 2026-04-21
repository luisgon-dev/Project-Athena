# Unraid Deployment

Project Athena ships as a single container image and is intended to run cleanly on Unraid through GitHub Container Registry.

## Published image

The publish workflow pushes:

- `ghcr.io/<github-owner>/project-athena:latest` from `main`
- `ghcr.io/<github-owner>/project-athena:vX.Y.Z` from matching tags
- `ghcr.io/<github-owner>/project-athena:sha-...` for immutable references

After the first push, make the GHCR package public if you want Unraid to pull it without GitHub credentials.

## Recommended Unraid mappings

- Port: `3000` container port to your preferred host port
- Path: `/config`
- Path: `/ebooks`
- Path: `/audiobooks`

Recommended environment values:

- `TZ=America/Los_Angeles`
- `PUID=99` or your preferred media user id
- `PGID=100` or your preferred media group id
- `UMASK=002`

Optional overrides:

- `DATABASE_PATH=/config/book-router.sqlite`
- `BIND_ADDR=0.0.0.0:3000`
- `METADATA_BASE_URL=https://openlibrary.org`
- `COVER_BASE_URL=https://covers.openlibrary.org`
- `ENABLE_FULFILLMENT_WORKERS=true`

## Runtime behavior

- If `PUID` and `PGID` are set together, the container entrypoint creates or reuses that uid/gid inside the container and starts Athena under that identity.
- The container only adjusts ownership for the database directory. Media roots are left alone on purpose so Unraid host ownership stays under your control.
- The image stores the SQLite database under `/config` by default, which matches the usual Unraid appdata pattern.

## Example Unraid-style docker run

```bash
docker run -d \
  --name=project-athena \
  -p 3001:3000 \
  -e TZ=America/Los_Angeles \
  -e PUID=99 \
  -e PGID=100 \
  -e UMASK=002 \
  -v /mnt/user/appdata/project-athena:/config \
  -v /mnt/user/media/ebooks:/ebooks \
  -v /mnt/user/media/audiobooks:/audiobooks \
  ghcr.io/<github-owner>/project-athena:latest
```
