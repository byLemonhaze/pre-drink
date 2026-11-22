PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;
PRAGMA temp_store = MEMORY;
PRAGMA secure_delete = ON;

CREATE TABLE IF NOT EXISTS conversations (
  id TEXT PRIMARY KEY,
  platform TEXT NOT NULL CHECK(platform IN ('chatgpt', 'claude', 'grok')),
  title TEXT NOT NULL,
  created_at REAL,
  updated_at REAL,
  private INTEGER NOT NULL DEFAULT 0 CHECK(private IN (0, 1)),
  sensitive INTEGER NOT NULL DEFAULT 0 CHECK(sensitive IN (0, 1)),
  raw_json TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS messages (
  id TEXT PRIMARY KEY,
  conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
  role TEXT NOT NULL CHECK(role IN ('user', 'assistant')),
  content TEXT NOT NULL,
  timestamp REAL,
  branch_id TEXT,
  parent_id TEXT,
  is_active INTEGER NOT NULL DEFAULT 1,
  sensitive INTEGER NOT NULL DEFAULT 0,
  position INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS messages_conversation ON messages(conversation_id, position);
CREATE TABLE IF NOT EXISTS memory (
  id TEXT PRIMARY KEY,
  type TEXT NOT NULL CHECK(type IN ('fact', 'decision', 'project')),
  content TEXT NOT NULL,
  source_message_id TEXT REFERENCES messages(id) ON DELETE SET NULL,
  approved INTEGER NOT NULL DEFAULT 0,
  had_source INTEGER NOT NULL DEFAULT 0,
  private INTEGER NOT NULL DEFAULT 0,
  sensitive INTEGER NOT NULL DEFAULT 0,
  created_at REAL NOT NULL,
  updated_at REAL NOT NULL
);
CREATE TABLE IF NOT EXISTS tags (id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE);
CREATE TABLE IF NOT EXISTS conversation_tags (
  conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
  tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
  PRIMARY KEY(conversation_id, tag_id)
);
CREATE VIRTUAL TABLE IF NOT EXISTS search_index USING fts5(conversation_id UNINDEXED, content, tokenize='unicode61');
CREATE TABLE IF NOT EXISTS agent_clients (
  id TEXT PRIMARY KEY, name TEXT NOT NULL, token_hash TEXT NOT NULL,
  allow_chats INTEGER NOT NULL DEFAULT 0, created_at REAL NOT NULL
);
CREATE TABLE IF NOT EXISTS archive_snapshots (
  id TEXT PRIMARY KEY, conversation_id TEXT NOT NULL REFERENCES conversations(id), raw_json TEXT NOT NULL, imported_at REAL NOT NULL
);
CREATE TABLE IF NOT EXISTS briefs (
  id TEXT PRIMARY KEY, title TEXT NOT NULL, content TEXT NOT NULL, approved INTEGER NOT NULL DEFAULT 0,
  private INTEGER NOT NULL DEFAULT 0, sensitive INTEGER NOT NULL DEFAULT 0, updated_at REAL NOT NULL
);
CREATE TABLE IF NOT EXISTS brief_sources (
  id INTEGER PRIMARY KEY, brief_id TEXT NOT NULL REFERENCES briefs(id) ON DELETE CASCADE,
  memory_id TEXT REFERENCES memory(id) ON DELETE SET NULL, reviewed_at REAL NOT NULL,
  UNIQUE(brief_id,memory_id)
);
PRAGMA user_version = 2;
