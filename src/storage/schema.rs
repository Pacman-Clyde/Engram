pub const CREATE_TABLES: &str = "
CREATE TABLE IF NOT EXISTS project_meta (
    name        TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    stack       TEXT NOT NULL DEFAULT '[]',
    conventions TEXT NOT NULL DEFAULT '[]',
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS decisions (
    id           TEXT PRIMARY KEY,
    title        TEXT NOT NULL,
    context      TEXT NOT NULL,
    decision     TEXT NOT NULL,
    alternatives TEXT NOT NULL DEFAULT '[]',
    tags         TEXT NOT NULL DEFAULT '[]',
    status       TEXT NOT NULL DEFAULT 'active',
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tasks (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    status      TEXT NOT NULL DEFAULT 'todo',
    priority    TEXT NOT NULL DEFAULT 'medium',
    phase       TEXT,
    tags        TEXT NOT NULL DEFAULT '[]',
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS file_summaries (
    id           TEXT PRIMARY KEY,
    path         TEXT NOT NULL UNIQUE,
    summary      TEXT NOT NULL,
    key_types    TEXT NOT NULL DEFAULT '[]',
    dependencies TEXT NOT NULL DEFAULT '[]',
    tags         TEXT NOT NULL DEFAULT '[]',
    content_hash TEXT NOT NULL,
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS sessions (
    id         TEXT PRIMARY KEY,
    agent      TEXT NOT NULL,
    goal       TEXT NOT NULL,
    handoff    TEXT,
    tags       TEXT NOT NULL DEFAULT '[]',
    started_at TEXT NOT NULL,
    ended_at   TEXT
);

CREATE VIRTUAL TABLE IF NOT EXISTS search_index USING fts5(
    entity_id,
    entity_type,
    body
);
";
