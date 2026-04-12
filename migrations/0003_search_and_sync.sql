CREATE TABLE review_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    request_id TEXT NOT NULL,
    candidate_external_id TEXT NOT NULL,
    candidate_source TEXT NOT NULL,
    candidate_title TEXT NOT NULL,
    candidate_protocol TEXT NOT NULL,
    candidate_size_bytes INTEGER NOT NULL,
    candidate_indexer TEXT NOT NULL,
    score REAL NOT NULL,
    explanation_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (request_id) REFERENCES requests(id) ON DELETE CASCADE
);
