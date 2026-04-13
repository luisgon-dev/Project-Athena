CREATE TABLE runtime_settings (
    singleton_key INTEGER PRIMARY KEY CHECK (singleton_key = 1),
    settings_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE synced_indexers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    prowlarr_indexer_id INTEGER,
    name TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    implementation TEXT NOT NULL,
    implementation_name TEXT,
    config_contract TEXT NOT NULL,
    protocol TEXT,
    priority INTEGER NOT NULL DEFAULT 25,
    base_url TEXT,
    api_path TEXT,
    categories_json TEXT NOT NULL DEFAULT '[]',
    api_key TEXT,
    raw_payload_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_synced_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
