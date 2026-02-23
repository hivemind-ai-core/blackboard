PRAGMA journal_mode = WAL;
PRAGMA busy_timeout = 5000;
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER NOT NULL
);
INSERT OR IGNORE INTO schema_version (version) VALUES (1);

CREATE TABLE IF NOT EXISTS agents (
    id TEXT PRIMARY KEY,
    current_task TEXT NOT NULL DEFAULT '',
    progress INTEGER NOT NULL DEFAULT 0
        CHECK (progress >= 0 AND progress <= 100),
    status TEXT NOT NULL DEFAULT 'idle'
        CHECK (status IN (
            'idle', 'planning', 'coding', 'testing',
            'reviewing', 'blocked', 'offline'
        )),
    blockers TEXT,
    last_seen TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_agent TEXT NOT NULL,
    content TEXT NOT NULL,
    tags TEXT NOT NULL DEFAULT '[]'
        CHECK (json_valid(tags)),
    priority TEXT NOT NULL DEFAULT 'normal'
        CHECK (priority IN ('low', 'normal', 'high', 'critical')),
    in_reply_to INTEGER REFERENCES messages(id),
    refs TEXT NOT NULL DEFAULT '[]'
        CHECK (json_valid(refs)),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS artifacts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE,
    produced_by TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    version TEXT,
    refs TEXT NOT NULL DEFAULT '[]'
        CHECK (json_valid(refs)),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_agents_status ON agents(status);
CREATE INDEX IF NOT EXISTS idx_agents_last_seen ON agents(last_seen);

CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_messages_from_agent ON messages(from_agent);
CREATE INDEX IF NOT EXISTS idx_messages_priority ON messages(priority) WHERE priority IN ('high', 'critical');
CREATE INDEX IF NOT EXISTS idx_messages_in_reply_to ON messages(in_reply_to)
    WHERE in_reply_to IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_artifacts_path ON artifacts(path);
CREATE INDEX IF NOT EXISTS idx_artifacts_produced_by ON artifacts(produced_by);
