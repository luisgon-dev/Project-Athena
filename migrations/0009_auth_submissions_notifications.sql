CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL,
    disabled INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    expires_at INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE request_submissions (
    id TEXT PRIMARY KEY,
    requested_by_user_id TEXT NOT NULL,
    intake_mode TEXT NOT NULL,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    external_work_id TEXT,
    notes TEXT,
    media_types_json TEXT NOT NULL,
    preferred_language TEXT,
    manifestation_json TEXT NOT NULL,
    status TEXT NOT NULL,
    requires_admin_approval INTEGER NOT NULL DEFAULT 0,
    allow_duplicate INTEGER NOT NULL DEFAULT 0,
    resolution_json TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (requested_by_user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE submission_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    submission_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (submission_id) REFERENCES request_submissions(id) ON DELETE CASCADE
);

ALTER TABLE requests ADD COLUMN submission_id TEXT REFERENCES request_submissions(id) ON DELETE SET NULL;
ALTER TABLE requests ADD COLUMN requested_by_user_id TEXT REFERENCES users(id) ON DELETE SET NULL;
ALTER TABLE requests ADD COLUMN requires_admin_approval INTEGER NOT NULL DEFAULT 0;

