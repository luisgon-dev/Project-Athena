CREATE TABLE requests (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    media_type TEXT NOT NULL,
    preferred_language TEXT,
    state TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE request_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    request_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (request_id) REFERENCES requests(id) ON DELETE CASCADE
);
