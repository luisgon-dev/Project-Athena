# Book Router Design

Date: 2026-04-09
Status: Approved and revised on 2026-04-10

## Summary

This document defines the first production version of a lightweight, Docker-first Rust application that replaces the request-routing and import portions of Readarr for books and audiobooks.

The system owns canonical identity and metadata from day one. It accepts metadata-backed requests, searches indexers through Prowlarr and/or direct indexer integrations, ranks release candidates, auto-sends only high-confidence matches to qBittorrent, imports completed downloads into `/ebooks` or `/audiobooks`, and then syncs outward to Calibre and Audiobookshelf.

The application is not a full library manager in v1. It is a request, acquisition, metadata, and import engine with admin-first workflows and a future path toward external request integrations such as Discord.

## Product Goals

1. Replace the request-to-download-to-import workflow that Readarr used to provide.
2. Support both ebooks and audiobooks in the first release.
3. Keep deployment simple enough for a home media stack.
4. Make metadata decisions explicit, auditable, and correctable.
5. Stay open source, configurable, and adaptable to different downstream stacks.
6. Treat Calibre and Audiobookshelf as integrations, not authoritative sources.

## Non-Goals For v1

1. Full Calibre replacement
2. Public multi-user request portal
3. Deep author monitoring and future-release following
4. Rich recommendation or discovery features
5. Perfect automatic edition disambiguation for all edge cases
6. Multi-instance or horizontally scaled deployment
7. First-party NZB client support

## Key Decisions

- Architecture: modular Rust monolith
- Database: SQLite as the default and only required v1 database
- Media support: ebooks and audiobooks from day one
- Ownership: the new app is the source of truth for request, identity, import, and metadata state
- Request model: metadata-first, work-backed requests with optional manifestation preferences or later pinning to a specific edition or recording
- Acquisition policy: auto-send to qBittorrent only above a strict confidence threshold
- Search strategy: support both Prowlarr and direct indexers from the same ranking pipeline
- Download support: design for torrent and NZB clients, implement qBittorrent first
- Import ownership: full app-owned import pipeline
- Metadata strategy: Open Library primary, Google Books enrichment only, local tags as supporting signals

## Why Readarr Failed Matters Here

Metadata is the highest-risk subsystem in this product area. Readarr was archived on June 27, 2025, and its archival note explicitly cites broken metadata foundations, especially the loss of Goodreads viability and incomplete migration to Open Library. This design assumes that no single provider is complete enough to be blindly trusted and that every automated metadata decision must be explainable, reversible, and bounded by confidence gates.

Sources:
- [Readarr repository and archival notice](https://github.com/Readarr/Readarr)
- [Open Library Search API](https://openlibrary.org/dev/docs/api/search)
- [Open Library Books API](https://openlibrary.org/dev/docs/api/books)
- [Google Books API overview](https://developers.google.com/books/docs/v1/getting_started)
- [Audiobookshelf book scanner guide](https://www.audiobookshelf.org/guides/book-scanner/)
- [qBittorrent wiki](https://github.com/qbittorrent/qBittorrent/wiki/)

## Architecture

The system is a modular monolith packaged as a single Docker-first Rust service with SQLite-backed persistence and mounted filesystem volumes for config, database, staging, and media roots.

This structure keeps deployment simple while preserving clear internal boundaries:

- `catalog`: canonical authors, series, works, manifestations, and cross-provider identity links
- `requests`: admin request creation, state transitions, review queue, and request intent
- `metadata`: provider adapters, normalization, enrichment, and reconciliation rules
- `search`: unified search interface for Prowlarr and direct indexers
- `matcher`: candidate scoring, hard filters, and auto-acquire decisions
- `downloads`: qBittorrent client and a generic download-client interface for future NZB support
- `importer`: completion detection, payload inspection, media classification, rename, move, and import recording
- `sync`: outbound integration to Calibre, Audiobookshelf, and future downstream consumers
- `api`: admin API for UI and future external request integrations
- `ui`: lightweight admin web interface
- `scheduler`: background jobs for refresh, retry, reconciliation, and cleanup
- `config`: typed application configuration, path rules, templates, and feature flags

### Deployment Model

Supported v1 deployment is a single app instance.

Required mounts:
- `/config`
- `/data`
- `/downloads` or other watched completion roots
- `/ebooks`
- `/audiobooks`

SQLite runs in WAL mode and all background workers run in-process. Job claims, retries, and import decisions must be transactional and idempotent.

## Domain Model

The application uses a two-layer identity model: conceptual works and concrete manifestations.

### Core Entities

- `Author`
  - Canonical name
  - Normalized aliases
  - External provider IDs

- `Series`
  - Canonical name
  - Provider IDs
  - Optional ordered membership data

- `Work`
  - Title-level concept such as "The Hobbit"
  - Primary authors
  - Optional series relationships
  - Language and subject metadata
  - Provider-backed identity links

- `Manifestation`
  - A concrete acquirable form of a work
  - Media type: `ebook` or `audiobook`
  - Edition or recording title
  - ISBN/ASIN or equivalent identifiers when present
  - Narrator, publisher, imprint, abridgment status, format, language, and release-specific traits

- `Request`
  - Links to a `Work`
  - May optionally pin a target `Manifestation`
  - Carries request preferences such as media type, preferred language, narrator, or edition keywords
  - Stores workflow state and decision history

- `ReleaseCandidate`
  - A normalized search result from Prowlarr or a direct indexer
  - Stores raw release name plus parsed attributes used in ranking

- `LibraryItem`
  - Imported local artifact on disk
  - Linked to a `Work` and best-known `Manifestation`
  - Stores checksums, source release, file inventory, and import location

- `ExternalLink`
  - Calibre, Audiobookshelf, and future system IDs connected to internal records

### Identity Rules

1. Requests must be created from a provider-backed work match rather than free-text alone.
2. Requests default to work-level intent after the admin selects a matched work from the metadata provider.
3. Requests can be pinned to a specific manifestation when the admin wants a precise edition or recording.
4. A manifestation may be partially specified when the provider data is incomplete.
5. The application may still auto-acquire an unpinned request if the result strongly matches the work and declared preferences.
6. A weak single signal must never define canonical identity on its own.

## Metadata Strategy

### Provider Model

- `Open Library` is the primary metadata provider for works, editions, authors, and ISBN-linked identity where available.
- `Google Books` is optional enrichment only and must not overwrite higher-confidence canonical fields without explicit rules.
- `Local file tags` and filename parsing are import-time supporting signals, not primary truth.
- `Manual admin corrections` are first-class and must be retained as higher-priority local overrides.

### Metadata Confidence Principles

The system should not assume provider completeness. Instead it should accumulate evidence and explain how a decision was reached.

Examples of signal strength:
- Strong: exact ISBN match, strong provider ID continuity, exact author and normalized title match
- Medium: edition title similarity, narrator match, publisher or imprint match, language alignment
- Weak: title-only match, series-only hints, generic format tokens
- Negative: wrong language, adaptation keywords, summary keywords, dramatization when unrequested, abridged when unrequested

### Manual Override Model

The admin must be able to:
- confirm or reject a candidate
- pin a request to a manifestation
- correct a work or manifestation link
- adjust preferred manifestation traits
- mark a release as permanently blocked for a request or globally

Overrides must be stored explicitly so future refresh jobs do not silently undo them.

## Request Creation Contract

Request creation is metadata-first.

The request window should not create requests directly from raw title and author text. Instead it should:

1. accept title and author search terms
2. query the metadata provider, currently Open Library
3. present matched works
4. require the admin to select one canonical work before a request can be created
5. allow the admin to choose `ebook`, `audiobook`, or both
6. allow optional manifestation preferences before submission:
   - preferred language
   - edition title
   - preferred narrator
   - preferred publisher
   - graphic audio preference

### No-Match Rule

If no metadata-provider match is found, no request can be created. There is no free-text fallback request path in v1.

### Dual-Media Convenience

If the admin selects both `ebook` and `audiobook`, the system should create two separate requests linked to the same canonical work. This is a convenience action, not a combined polymorphic request.

### Partial Validation Rule

If both media types are selected but only one side is valid, the system should create only the valid request and explain why the other one was skipped.

### Request Window UX Shape

The preferred v1 request UX is a lightweight metadata-first wizard:

1. Search for a work
2. Select a matched work
3. Choose one or both media types
4. Add optional manifestation preferences
5. Create one or two requests

This keeps metadata identity at the front of the workflow and prevents raw, unresolved requests from entering the system.

## Search And Ranking

### Search Inputs

The system supports two acquisition paths from the same internal interface:

1. Prowlarr-backed search
   - support Prowlarr application sync behavior where appropriate
   - route search queries through Prowlarr when configured

2. Direct indexer search
   - query Torznab or compatible sources directly
   - normalize results into the same `ReleaseCandidate` structure

### Ranking Pipeline

1. Create a query plan from the request and target media type.
2. Search enabled sources.
3. Normalize results into a common structure.
4. Apply hard filters:
   - media type mismatch
   - blocked words
   - forbidden language
   - invalid file or archive types
   - missing required edition cues
5. Compute weighted confidence score.
6. Sort remaining candidates.
7. Auto-send only if the top candidate exceeds the configured threshold.
8. Otherwise enqueue for manual review with the score breakdown.

### Candidate Signals

Signals used by the matcher should include:
- title and alternate title normalization
- author alias matching
- work and manifestation identifiers
- language
- series and series index as soft signals
- narrator
- publisher or imprint
- edition or recording keywords
- abridged or unabridged cues
- audio-specific branding such as GraphicAudio
- release group and scene naming heuristics
- file inventory heuristics, such as EPUB/PDF/AZW for ebooks and MP3/M4B/FLAC for audiobooks

### Auto-Acquire Policy

Default v1 behavior:
- high confidence: automatically send to qBittorrent
- medium confidence: queue for review
- low confidence: reject silently from acquisition results, but keep audit trace if debugging is enabled

The UI should always be able to explain why a candidate was auto-approved, queued, or rejected.

## Download Client Integration

### qBittorrent

qBittorrent is the first implemented client and the only download client supported in v1.

Required capabilities:
- authentication
- add torrent or magnet
- category and tag assignment
- request-linked metadata tagging
- progress polling
- completion detection
- file listing
- pause, resume, and remove

Every acquired download should carry a request identifier through category, tag, or an equivalent metadata mechanism so completed downloads can be linked back deterministically.

### Future NZB Path

The internal download interface must be designed so SABnzbd or NZBGet can be added later without changing request, matcher, or import semantics. However, no NZB implementation is required in v1.

## Import Pipeline

The application owns the full import path in v1.

### Completion Handling

1. Detect a completed qBittorrent job.
2. Resolve the corresponding request and intended media type.
3. Inspect payload contents.
4. Optionally unpack archives if enabled.
5. Classify payload as `ebook`, `audiobook`, `mixed`, or `invalid`.
6. Extract supporting metadata from files and names.
7. Confirm or revise the candidate manifestation link.
8. Rename and move files to the final media root.
9. Record the import event and mark request state accordingly.

### Import Rules

- `/ebooks` is the final root for ebooks
- `/audiobooks` is the final root for audiobooks
- mixed payloads must be reviewable and should not be silently split without policy
- invalid payloads must be quarantined or marked failed with a clear reason
- file moves must be idempotent
- rename rules must be configurable
- import history must be preserved even if the downstream sync fails

### Suggested Naming Baseline

V1 should implement configurable naming templates with safe defaults:

- Ebook folder or file naming centered on `Author / Work / Edition`
- Audiobook folder naming centered on `Author / Work / Narrator or Edition`

Exact templates can remain configurable, but the canonical internal link must not depend on the rendered filename.

## Downstream Sync

The app remains authoritative even after import.

### Calibre

Calibre is an outbound integration target for ebook imports. The system should support:
- add imported ebooks
- push sidecar metadata where useful
- record returned external IDs

### Audiobookshelf

Audiobookshelf is an outbound integration target for audiobook imports. The system should support:
- trigger scan or update flow
- record external IDs
- preserve enough local metadata to support later rematch or re-sync

The design intentionally does not rely on Audiobookshelf matching to define canonical identity, because Audiobookshelf's scanner groups local files first and online matching happens later.

### Future Targets

The sync layer should be abstract enough to support:
- Jellyfin-oriented libraries
- alternate ebook managers
- alternate audiobook managers

## UI And API

### Admin UI For v1

The first UI is admin-only and should focus on operational clarity rather than broad product surface.

Core screens:
- metadata search and request creation
- request detail and lifecycle
- search result review
- import queue and failures
- metadata correction and manifestation pinning
- settings and integration health
- event history and decision trace

### External Request Integrations

The API should be designed from day one to allow future integrations such as:
- Discord bot commands
- webhook-driven request sources
- third-party request portals

These are explicitly scoped after v1, but request creation and status endpoints should not require redesign to support them. External request sources should also follow the same metadata-first contract rather than creating unresolved free-text requests.

## Configuration

Configurable areas should include:
- filesystem roots
- qBittorrent connection details
- Prowlarr connection details
- direct indexer configuration
- preferred languages
- blocked terms
- media-type-specific format rules
- naming templates
- metadata provider toggles
- confidence thresholds
- import behavior such as archive extraction and mixed-payload handling

Configuration should support both environment variables and a durable config file mounted under `/config`.

## Persistence

SQLite is the default and only required v1 database.

Requirements:
- WAL mode
- foreign keys enabled
- transactional workflow updates
- migrations managed by the app
- durable event and audit log tables
- indexes supporting search result reconciliation, request lookups, and import history

The persistence layer should remain isolated enough that Postgres could be added later, but the implementation should optimize for SQLite simplicity in v1 rather than premature database portability.

## Reliability And Observability

### Event History

Every request should maintain an append-only event stream such as:
- request created
- metadata resolved
- search executed
- candidate auto-approved
- candidate queued for review
- sent to qBittorrent
- download completed
- import succeeded or failed
- sync succeeded or failed
- metadata corrected manually

### Logs And Diagnostics

The app should emit structured logs with request IDs, download IDs, and candidate IDs. The UI should expose a human-readable explanation of matching decisions.

### Retry Model

- transient search failures should retry with backoff
- qBittorrent connectivity failures should retry
- failed imports should move into a repairable queue
- sync failures should not roll back successful imports

## Security

V1 is admin-first, but basic controls still matter:
- local auth for the admin UI and API
- secret storage for provider and integration credentials
- write-path confinement to configured roots
- file-type allowlists for imports
- explicit archive handling policy

## Testing Strategy

### Unit Tests

- normalization of titles, authors, and aliases
- manifestation preference matching
- weighted confidence scoring
- import classification
- naming template rendering

### Contract Tests

- Prowlarr adapter
- qBittorrent adapter
- Calibre adapter
- Audiobookshelf adapter
- metadata provider adapters

### Fixture Tests

Use a corpus of real-world messy release names and file layouts including:
- alternate titles
- box sets and omnibuses
- graphic audio and dramatizations
- abridged versus unabridged releases
- wrong-language near matches
- mixed ebook and audiobook payloads
- malformed archives

### End-To-End Tests

Run docker-compose based flows for:
- request to successful auto-acquire
- request to manual review
- successful import into each media root
- downstream sync after import
- retry and failure-recovery cases

## Open Questions Deferred Beyond v1

These are intentionally deferred rather than left ambiguous:

- public or household user accounts
- advanced recommendation and discovery workflows
- future-release monitoring by author or series
- first-class Discord bot implementation
- first-class NZB client implementation
- post-import metadata editing beyond focused admin correction
- multi-node deployment

## Recommended Implementation Shape

The implementation should start with the smallest end-to-end slice that proves the hardest system behaviors:

1. persistent SQLite-backed request storage
2. metadata search and work selection from Open Library
3. metadata-backed request creation for one or both media types
4. one search path through Prowlarr
5. candidate scoring and review queue
6. qBittorrent dispatch
7. import into `/ebooks` and `/audiobooks`
8. event history and audit trail

Direct indexers, Calibre sync, Audiobookshelf sync, richer manifestation pinning, and external request integrations should layer onto that first vertical slice.

## Acceptance Criteria For v1

The first release is successful if an admin can:

1. search the metadata provider and select a canonical work
2. create an ebook request, an audiobook request, or both from that selected work
3. specify optional edition or recording preferences before request submission
4. search via Prowlarr and/or direct indexers
5. have the system auto-send only high-confidence candidates to qBittorrent
6. review and approve uncertain candidates manually
7. import completed downloads into the correct media root
8. inspect why a decision was made
9. correct metadata when the automatic path is wrong
10. sync imported items outward to Calibre and Audiobookshelf without surrendering canonical ownership

## Risks And Mitigations

### Metadata Incompleteness

Risk:
Open Library is incomplete and inconsistent for some books and editions.

Mitigation:
Use a scored identity model, optional enrichment, and explicit manual override paths.

### Bad Release Naming

Risk:
Release names are often messy, incomplete, or misleading.

Mitigation:
Use hard filters, weighted scoring, and a review queue instead of aggressive auto-acquire.

### Mixed Or Broken Payloads

Risk:
Completed downloads may contain the wrong media type, mixed media, or junk files.

Mitigation:
Make payload inspection and import classification first-class, and quarantine invalid results.

### Over-Expanding Scope

Risk:
The project drifts into a full library manager too early.

Mitigation:
Keep v1 focused on request, acquisition, metadata, import, and sync.
