DROP VIEW IF EXISTS curation_blocked;
CREATE TABLE IF NOT EXISTS agent_scopes_v5 (
  client_id TEXT PRIMARY KEY REFERENCES agent_clients(id) ON DELETE CASCADE,
  provider TEXT NOT NULL CHECK(provider IN ('chatgpt','grok','claude')),
  can_curate INTEGER NOT NULL DEFAULT 0 CHECK(can_curate IN (0,1))
);
INSERT INTO agent_scopes_v5 SELECT * FROM agent_scopes;
DROP TABLE agent_scopes;
ALTER TABLE agent_scopes_v5 RENAME TO agent_scopes;
CREATE TABLE IF NOT EXISTS curation_policy_v5 (
  provider TEXT PRIMARY KEY CHECK(provider IN ('chatgpt','grok','claude')),
  publish_archive INTEGER NOT NULL DEFAULT 0,
  publish_session INTEGER NOT NULL DEFAULT 0
);
INSERT INTO curation_policy_v5 SELECT * FROM curation_policy;
DROP TABLE curation_policy;
ALTER TABLE curation_policy_v5 RENAME TO curation_policy;
CREATE TABLE IF NOT EXISTS curation_notes_v5 (
  id TEXT PRIMARY KEY,
  provider TEXT NOT NULL CHECK(provider IN ('chatgpt','grok','claude')),
  kind TEXT NOT NULL CHECK(kind IN ('fact','decision','project')),
  content TEXT NOT NULL,
  origin TEXT NOT NULL CHECK(origin IN ('archive','session')),
  session_label TEXT NOT NULL,
  revision INTEGER NOT NULL,
  conflict INTEGER NOT NULL,
  sensitive INTEGER NOT NULL,
  withdrawn INTEGER NOT NULL DEFAULT 0,
  updated_at REAL NOT NULL
);
INSERT INTO curation_notes_v5 SELECT * FROM curation_notes;
DROP TABLE curation_notes;
ALTER TABLE curation_notes_v5 RENAME TO curation_notes;
