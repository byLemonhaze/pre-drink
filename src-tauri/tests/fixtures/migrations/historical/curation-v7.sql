-- Provider grants are owner-created. Legacy clients get no grant automatically.
CREATE TABLE IF NOT EXISTS agent_scopes (
  client_id TEXT PRIMARY KEY REFERENCES agent_clients(id) ON DELETE CASCADE,
  provider TEXT NOT NULL CHECK(provider IN ('chatgpt','grok','claude')),
  can_curate INTEGER NOT NULL DEFAULT 0 CHECK(can_curate IN (0,1))
);
CREATE TABLE IF NOT EXISTS curation_policy (
  provider TEXT PRIMARY KEY CHECK(provider IN ('chatgpt','grok','claude')),
  publish_archive INTEGER NOT NULL DEFAULT 0,
  publish_session INTEGER NOT NULL DEFAULT 0
);
INSERT OR IGNORE INTO curation_policy(provider) VALUES ('chatgpt'),('grok'),('claude');
CREATE TABLE IF NOT EXISTS source_versions (
  message_id TEXT PRIMARY KEY,
  version INTEGER NOT NULL
);
INSERT OR IGNORE INTO source_versions SELECT id,1 FROM messages;
CREATE TRIGGER IF NOT EXISTS curation_source_insert AFTER INSERT ON messages BEGIN
  INSERT INTO source_versions VALUES (NEW.id,1) ON CONFLICT(message_id) DO UPDATE SET version=version+1;
END;
CREATE TRIGGER IF NOT EXISTS curation_source_update AFTER UPDATE OF content,role,parent_id,is_active,timestamp,branch_id ON messages
WHEN OLD.content != NEW.content OR OLD.role != NEW.role OR OLD.parent_id IS NOT NEW.parent_id OR OLD.is_active!=NEW.is_active OR OLD.timestamp IS NOT NEW.timestamp OR OLD.branch_id IS NOT NEW.branch_id
BEGIN
  UPDATE source_versions SET version=version+1 WHERE message_id=NEW.id;
END;
CREATE TABLE IF NOT EXISTS curation_notes (
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
CREATE TABLE IF NOT EXISTS curation_sources (
  note_id TEXT NOT NULL REFERENCES curation_notes(id) ON DELETE CASCADE,
  message_id TEXT NOT NULL,
  version INTEGER NOT NULL,
  PRIMARY KEY(note_id,message_id)
);
CREATE TABLE IF NOT EXISTS curation_revisions (
  note_id TEXT NOT NULL REFERENCES curation_notes(id) ON DELETE CASCADE,
  revision INTEGER NOT NULL,
  author_client_id TEXT NOT NULL,
  snapshot TEXT NOT NULL,
  created_at REAL NOT NULL,
  PRIMARY KEY(note_id,revision)
);
CREATE TABLE IF NOT EXISTS curation_requests (
  client_id TEXT NOT NULL,
  request_key TEXT NOT NULL,
  request_hash TEXT NOT NULL,
  result TEXT NOT NULL,
  PRIMARY KEY(client_id,request_key)
);
CREATE TABLE IF NOT EXISTS curation_jobs (
  id TEXT PRIMARY KEY,
  provider TEXT NOT NULL CHECK(provider IN ('chatgpt','grok','claude')),
  label TEXT NOT NULL,
  revision INTEGER NOT NULL DEFAULT 1,
  checkpoint TEXT NOT NULL DEFAULT '',
  status TEXT NOT NULL DEFAULT 'running' CHECK(status IN ('running','paused','completed','cancelled')),
  updated_at REAL NOT NULL
);
-- Separate owner authority: agent inputs never update this table.
CREATE TABLE IF NOT EXISTS curation_owner_holds (
  note_id TEXT PRIMARY KEY REFERENCES curation_notes(id) ON DELETE CASCADE
);
-- Checked at every delivery, including briefs and owner sharing previews.
DROP VIEW IF EXISTS curation_blocked;
CREATE VIEW curation_blocked AS
SELECT n.id FROM curation_notes n WHERE n.withdrawn=1 OR n.conflict=1 OR n.sensitive=1
OR EXISTS(SELECT 1 FROM curation_owner_holds h WHERE h.note_id=n.id)
OR (n.origin='archive' AND NOT EXISTS(SELECT 1 FROM curation_sources s WHERE s.note_id=n.id))
OR EXISTS(SELECT 1 FROM curation_sources s
 LEFT JOIN messages m ON m.id=s.message_id
 LEFT JOIN conversations c ON c.id=m.conversation_id
 LEFT JOIN source_versions v ON v.message_id=s.message_id
 WHERE s.note_id=n.id AND (m.id IS NULL OR c.id IS NULL OR c.platform!=n.provider
 OR c.private=1 OR c.sensitive=1 OR m.sensitive=1 OR v.version IS NULL OR v.version!=s.version));

CREATE TABLE IF NOT EXISTS curation_job_sources (
  job_id TEXT NOT NULL REFERENCES curation_jobs(id) ON DELETE CASCADE,
  ordinal INTEGER NOT NULL,
  message_id TEXT NOT NULL,
  version INTEGER NOT NULL,
  PRIMARY KEY(job_id,ordinal)
);
CREATE TABLE IF NOT EXISTS curation_processed_sources (
  provider TEXT NOT NULL,
  message_id TEXT NOT NULL,
  version INTEGER NOT NULL,
  PRIMARY KEY(provider,message_id)
);

CREATE TABLE IF NOT EXISTS curation_publications (
  note_id TEXT PRIMARY KEY REFERENCES curation_notes(id) ON DELETE CASCADE,
  revision INTEGER NOT NULL,
  basis TEXT NOT NULL CHECK(basis IN ('owner_review','automatic_policy'))
);

CREATE TRIGGER IF NOT EXISTS curation_source_privacy AFTER UPDATE OF sensitive ON messages
WHEN OLD.sensitive!=NEW.sensitive
BEGIN
  UPDATE source_versions SET version=version+1 WHERE message_id=NEW.id;
END;
CREATE TRIGGER IF NOT EXISTS curation_conversation_privacy AFTER UPDATE OF private,sensitive,platform ON conversations
WHEN OLD.private!=NEW.private OR OLD.sensitive!=NEW.sensitive OR OLD.platform!=NEW.platform
BEGIN
  UPDATE source_versions SET version=version+1 WHERE message_id IN (SELECT id FROM messages WHERE conversation_id=NEW.id);
END;
