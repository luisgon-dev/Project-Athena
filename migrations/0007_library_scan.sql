ALTER TABLE requests ADD COLUMN imported_path TEXT;

CREATE TABLE library_scan_jobs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    started_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TEXT,
    ebooks_found INTEGER NOT NULL DEFAULT 0,
    audiobooks_found INTEGER NOT NULL DEFAULT 0,
    duplicates_skipped INTEGER NOT NULL DEFAULT 0,
    error_message TEXT
);
