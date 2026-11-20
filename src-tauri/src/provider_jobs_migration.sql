-- Schema 6 also widens saved jobs for vaults already opened by schema 5.
CREATE TABLE curation_jobs_v6 (
  id TEXT PRIMARY KEY,
  provider TEXT NOT NULL CHECK(provider IN ('chatgpt','grok','claude')),
  label TEXT NOT NULL,
  revision INTEGER NOT NULL DEFAULT 1,
  checkpoint TEXT NOT NULL DEFAULT '',
  status TEXT NOT NULL DEFAULT 'running' CHECK(status IN ('running','paused','completed','cancelled')),
  updated_at REAL NOT NULL
);
INSERT INTO curation_jobs_v6 SELECT * FROM curation_jobs;
DROP TABLE curation_jobs;
ALTER TABLE curation_jobs_v6 RENAME TO curation_jobs;
