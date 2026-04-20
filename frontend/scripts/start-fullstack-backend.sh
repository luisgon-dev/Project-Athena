#!/usr/bin/env sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/../.." && pwd)
TMP_BASE=$(mktemp -d "${TMPDIR:-/tmp}/athena-fullstack-e2e.XXXXXX")

mkdir -p "$TMP_BASE/ebooks" "$TMP_BASE/audiobooks"

export BIND_ADDR="127.0.0.1:4174"
export DATABASE_PATH="$TMP_BASE/book-router.sqlite"
export EBOOKS_ROOT="$TMP_BASE/ebooks"
export AUDIOBOOKS_ROOT="$TMP_BASE/audiobooks"
export METADATA_BASE_URL="http://127.0.0.1:5001"
export COVER_BASE_URL="http://127.0.0.1:5001"
export ENABLE_FULFILLMENT_WORKERS="false"

cd "$ROOT_DIR"
exec cargo run --quiet --manifest-path "$ROOT_DIR/Cargo.toml" --bin book_router

