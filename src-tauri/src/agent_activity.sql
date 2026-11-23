CREATE TABLE IF NOT EXISTS agent_controls (
  client_id TEXT PRIMARY KEY REFERENCES agent_clients(id) ON DELETE CASCADE,
  paused INTEGER NOT NULL DEFAULT 0 CHECK(paused IN (0,1)),
  last_request_at REAL
);
CREATE TABLE IF NOT EXISTS agent_activity (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  client_id TEXT REFERENCES agent_clients(id) ON DELETE SET NULL,
  client_name TEXT NOT NULL,
  occurred_at REAL NOT NULL,
  operation TEXT NOT NULL CHECK(operation IN (
    'connection_status','get_facts','get_decisions','get_projects','get_context_pack',
    'search_conversations','mark_private','curate_memory','unknown_tool',
    'client_created','client_paused','client_resumed','client_revoked','revoke_all')),
  outcome TEXT NOT NULL CHECK(outcome IN ('started','completed','failed','paused'))
);
CREATE INDEX IF NOT EXISTS agent_activity_client ON agent_activity(client_id,id);

CREATE TABLE IF NOT EXISTS agent_activity_totals (
  singleton INTEGER PRIMARY KEY CHECK(singleton=1),
  denied INTEGER NOT NULL DEFAULT 0 CHECK(denied BETWEEN 0 AND 9007199254740991),
  trimmed INTEGER NOT NULL DEFAULT 0 CHECK(trimmed BETWEEN 0 AND 9007199254740991)
);
INSERT OR IGNORE INTO agent_activity_totals(singleton) VALUES(1);
