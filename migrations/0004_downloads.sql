CREATE TABLE downloads (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    request_id TEXT NOT NULL,
    candidate_external_id TEXT NOT NULL,
    candidate_source TEXT NOT NULL,
    candidate_title TEXT NOT NULL,
    candidate_protocol TEXT NOT NULL,
    candidate_size_bytes INTEGER NOT NULL,
    candidate_indexer TEXT NOT NULL,
    category TEXT NOT NULL,
    status TEXT NOT NULL,
    payload_json TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (request_id) REFERENCES requests(id) ON DELETE CASCADE
);
