-- One owner-controlled import at a time. Never exposed through MCP/search.
CREATE TABLE IF NOT EXISTS import_job (
  id INTEGER PRIMARY KEY CHECK(id=1),
  fingerprint TEXT NOT NULL,
  label TEXT NOT NULL,
  total INTEGER NOT NULL CHECK(total>=0),
  saved INTEGER NOT NULL DEFAULT 0 CHECK(saved>=0 AND saved<=total),
  messages INTEGER NOT NULL DEFAULT 0,
  skipped INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE IF NOT EXISTS import_items (
  job_id INTEGER NOT NULL DEFAULT 1 REFERENCES import_job(id) ON DELETE CASCADE,
  position INTEGER PRIMARY KEY,
  conversation_id TEXT NOT NULL UNIQUE,
  payload TEXT NOT NULL
);
