ALTER TABLE review_queue
    ADD COLUMN candidate_download_url TEXT;

CREATE TABLE rejected_candidates (
    request_id TEXT NOT NULL,
    candidate_external_id TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (request_id, candidate_external_id),
    FOREIGN KEY (request_id) REFERENCES requests(id) ON DELETE CASCADE
);
