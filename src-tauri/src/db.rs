use crate::model::{Conversation, ImportReport, Message, ParsedConversation, Stats};
use rusqlite::{params, Connection};
use std::{path::Path, time::Duration};

pub struct Vault {
    pub(crate) connection: Connection,
    pub(crate) path: String,
}

impl Vault {
    pub fn memory() -> Result<Self, String> {
        Self::initialize(
            Connection::open_in_memory().map_err(|e| e.to_string())?,
            ":memory:".into(),
        )
    }

    pub fn open(path: &Path, password: &str) -> Result<Self, String> {
        let salt = crate::crypto::salt(path)?;
        let key = crate::crypto::key(password, &salt)?;
        let connection =
            Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE)
                .map_err(|_| "Couldn't open the encrypted vault.")?;
        crate::crypto::configure(&connection, &key)?;
        Self::initialize(connection, path.to_string_lossy().into_owned())
    }

    pub fn create(path: &Path, password: &str) -> Result<Self, String> {
        crate::crypto::validate_password(password)?;
        let salt = crate::crypto::new_salt()?;
        let key = crate::crypto::key(password, &salt)?;
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        options
            .open(path)
            .map_err(|_| "A vault already exists, or this folder is not writable.")?;
        let result = (|| {
            let connection = Connection::open(path).map_err(|e| e.to_string())?;
            crate::crypto::configure(&connection, &key)?;
            Self::initialize(connection, path.to_string_lossy().into_owned())
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(path);
        }
        result
    }

    fn initialize(connection: Connection, path: String) -> Result<Self, String> {
        connection
            .busy_timeout(Duration::from_secs(5))
            .map_err(|e| e.to_string())?;
        connection
            .execute_batch(
                "PRAGMA foreign_keys=ON; PRAGMA temp_store=MEMORY; PRAGMA secure_delete=ON;",
            )
            .map_err(|e| e.to_string())?;
        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        if version > 4 {
            return Err("This vault needs a newer Pre-Drink version.".into());
        }
        if version == 1 {
            connection.execute_batch("BEGIN IMMEDIATE; ALTER TABLE memory ADD COLUMN had_source INTEGER NOT NULL DEFAULT 0; UPDATE memory SET had_source=1 WHERE source_message_id IS NOT NULL; PRAGMA user_version=2; COMMIT;").map_err(|e| e.to_string())?;
        }
        connection
            .execute_batch(include_str!("schema.sql"))
            .map_err(|e| e.to_string())?;
        if version < 3 {
            // Reclassify existing vaults before any sharing can start. Keep old
            // restrictions; the new review contract requires explicit reapproval.
            connection
                .execute_batch("BEGIN IMMEDIATE;")
                .map_err(|e| e.to_string())?;
            let refreshed = (|| -> Result<(), String> {
                for (table, column) in [
                    ("messages", "content"),
                    ("conversations", "title"),
                    ("memory", "content"),
                    ("briefs", "title || ' ' || content"),
                ] {
                    let mut stmt = connection
                        .prepare(&format!(
                            "SELECT id,{column} FROM {table} WHERE sensitive=0"
                        ))
                        .map_err(|e| e.to_string())?;
                    let rows = stmt
                        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
                        .map_err(|e| e.to_string())?
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|e| e.to_string())?;
                    for (id, content) in rows {
                        if crate::privacy::is_sensitive(&content) {
                            connection
                                .execute(
                                    &format!("UPDATE {table} SET sensitive=1 WHERE id=?1"),
                                    [id],
                                )
                                .map_err(|e| e.to_string())?;
                        }
                    }
                }
                connection.execute_batch("UPDATE conversations SET sensitive=1 WHERE EXISTS(SELECT 1 FROM messages WHERE conversation_id=conversations.id AND sensitive=1); UPDATE memory SET approved=0; UPDATE briefs SET approved=0; PRAGMA user_version=3;").map_err(|e| e.to_string())
            })();
            if let Err(error) = refreshed {
                let _ = connection.execute_batch("ROLLBACK;");
                return Err(error);
            }
            connection
                .execute_batch("COMMIT;")
                .map_err(|e| e.to_string())?;
        }
        // Additive migration: preserve all manual notes and saved-version approvals.
        // Broad legacy chat grants are deliberately downgraded, never guessed from names.
        let tx = connection
            .unchecked_transaction()
            .map_err(|e| e.to_string())?;
        tx.execute_batch(include_str!("curation.sql"))
            .map_err(|e| e.to_string())?;
        if version < 4 {
            tx.execute("UPDATE agent_clients SET allow_chats=0", [])
                .map_err(|e| e.to_string())?;
            tx.execute_batch("PRAGMA user_version=4;")
                .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        let mut vault = Self { connection, path };
        vault.enforce_sensitive_rules()?;
        Ok(vault)
    }

    pub fn checkpoint(&self) -> Result<(), String> {
        self.connection
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .map_err(|e| e.to_string())
    }

    pub fn encrypted_backup(&self, target: &Path, password: &str) -> Result<(), String> {
        crate::crypto::validate_password(password)?;
        if target.exists() {
            return Err("Backups never overwrite an existing file. Choose a new filename.".into());
        }
        let parent = target.parent().ok_or("Choose a backup folder.")?;
        let staging = parent.join(format!(
            ".pre-drink-backup-{}.partial",
            uuid::Uuid::new_v4()
        ));
        let result: Result<(), String> = (|| {
            let mut options = std::fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            options
                .open(&staging)
                .map_err(|_| "The backup folder is not writable.")?;
            let salt = crate::crypto::new_salt()?;
            let key = crate::crypto::key(password, &salt)?;
            self.connection
                .execute(
                    "ATTACH DATABASE ?1 AS backup KEY ?2",
                    params![staging.to_string_lossy(), &*key],
                )
                .map_err(|_| "Couldn't create encrypted backup.")?;
            let exported = self
                .connection
                .execute_batch("SELECT sqlcipher_export('backup'); PRAGMA backup.user_version=4;");
            let detached = self.connection.execute_batch("DETACH DATABASE backup;");
            exported
                .and(detached)
                .map_err(|_| "Backup could not finish.")?;
            let verified = Self::open(&staging, password)?;
            verified.verify()?;
            for table in [
                "conversations",
                "messages",
                "memory",
                "briefs",
                "brief_sources",
                "archive_snapshots",
                "agent_clients",
                "agent_scopes",
                "curation_policy",
                "curation_notes",
                "curation_publications",
                "curation_sources",
                "curation_revisions",
                "curation_requests",
                "curation_jobs",
                "curation_job_sources",
                "curation_processed_sources",
                "source_versions",
                "tags",
                "conversation_tags",
            ] {
                let sql = format!("SELECT count(*) FROM {table}");
                let original: i64 = self
                    .connection
                    .query_row(&sql, [], |r| r.get(0))
                    .map_err(|e| e.to_string())?;
                let copied: i64 = verified
                    .connection
                    .query_row(&sql, [], |r| r.get(0))
                    .map_err(|e| e.to_string())?;
                if original != copied {
                    return Err("Backup verification failed.".into());
                }
            }
            verified
                .checkpoint()
                .map_err(|_| "Backup checkpoint failed.")?;
            drop(verified);
            std::fs::File::open(&staging)
                .and_then(|f| f.sync_all())
                .map_err(|_| "Could not flush the backup to disk.")?;
            // Same-folder hard link publishes atomically and refuses to overwrite a raced-in file.
            std::fs::hard_link(&staging, target).map_err(|_| {
                "Couldn't publish the backup. Choose an unused filename on a local filesystem."
            })?;
            Ok(())
        })();
        let _ = std::fs::remove_file(&staging);
        result
    }

    pub fn migrate_plaintext(source: &Path, target: &Path, password: &str) -> Result<Self, String> {
        if target.exists() {
            return Err("An encrypted vault already exists.".into());
        }
        let connection =
            Connection::open_with_flags(source, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE)
                .map_err(|e| e.to_string())?;
        connection
            .execute_batch("PRAGMA locking_mode=EXCLUSIVE; BEGIN EXCLUSIVE; COMMIT;")
            .map_err(|_| "Close other versions of Pre-Drink before migrating.")?;
        let legacy = Self::initialize(connection, source.to_string_lossy().into_owned())?;
        legacy.verify()?;
        legacy.encrypted_backup(target, password)?;
        legacy.checkpoint()?;
        Self::open(target, password)
    }

    pub fn verify(&self) -> Result<(), String> {
        let status: String = self
            .connection
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        if status != "ok" {
            return Err("Vault integrity check failed.".into());
        }
        let issues: i64 = self
            .connection
            .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| {
                r.get(0)
            })
            .map_err(|e| e.to_string())?;
        if issues != 0 {
            return Err("Vault relationships failed verification.".into());
        }
        Ok(())
    }

    pub fn import(
        &mut self,
        conversations: &[ParsedConversation],
        mut report: ImportReport,
    ) -> Result<ImportReport, String> {
        let tx = self.connection.transaction().map_err(|e| e.to_string())?;
        // Scan the FTS table once per import, not once per conversation. Temp
        // storage is memory-only and the transaction also covers index rebuilds.
        tx.execute_batch("CREATE TEMP TABLE import_ids (id TEXT PRIMARY KEY); CREATE TEMP TABLE import_message_ids (id TEXT PRIMARY KEY);")
            .map_err(|e| e.to_string())?;
        {
            let mut insert = tx
                .prepare_cached("INSERT OR IGNORE INTO import_ids(id) VALUES (?1)")
                .map_err(|e| e.to_string())?;
            for c in conversations {
                insert.execute([&c.id]).map_err(|e| e.to_string())?;
            }
        }
        tx.execute(
            "DELETE FROM search_index WHERE conversation_id IN (SELECT id FROM import_ids)",
            [],
        )
        .map_err(|e| e.to_string())?;
        for c in conversations {
            let exists: bool = tx
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM conversations WHERE id=?1)",
                    [&c.id],
                    |r| r.get(0),
                )
                .map_err(|e| e.to_string())?;
            if exists {
                report.updated_conversations += 1;
            } else {
                report.new_conversations += 1;
            }
            tx.execute("INSERT INTO conversations(id,platform,title,created_at,updated_at,sensitive,raw_json)
                VALUES (?1,'chatgpt',?2,?3,?4,?5,?6) ON CONFLICT(id) DO UPDATE SET
                title=excluded.title, created_at=excluded.created_at, updated_at=excluded.updated_at,
                sensitive=conversations.sensitive OR excluded.sensitive, raw_json=excluded.raw_json",
                params![c.id,c.title,c.created_at,c.updated_at,c.sensitive,c.raw_json]).map_err(|e| e.to_string())?;
            use sha2::{Digest, Sha256};
            let snapshot_id = hex::encode(Sha256::digest(c.raw_json.as_bytes()));
            tx.execute("INSERT OR IGNORE INTO archive_snapshots(id,conversation_id,raw_json,imported_at) VALUES (?1,?2,?3,unixepoch())", params![snapshot_id,c.id,c.raw_json]).map_err(|e|e.to_string())?;
            // Retain branches, but deactivate only missing messages. Avoid transient
            // active-path flips on identical imports: they invalidate evidence versions.
            tx.execute("DELETE FROM import_message_ids", [])
                .map_err(|e| e.to_string())?;
            {
                let mut insert = tx
                    .prepare_cached("INSERT INTO import_message_ids(id) VALUES (?1)")
                    .map_err(|e| e.to_string())?;
                for message in &c.messages {
                    insert.execute([&message.id]).map_err(|e| e.to_string())?;
                }
            }
            tx.execute("UPDATE messages SET is_active=0 WHERE conversation_id=?1 AND id NOT IN (SELECT id FROM import_message_ids)",[&c.id]).map_err(|e|e.to_string())?;
            {
                let mut insert = tx.prepare_cached("INSERT INTO messages(id,conversation_id,role,content,timestamp,branch_id,parent_id,is_active,sensitive,position)
                    VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10) ON CONFLICT(id) DO UPDATE SET
                    role=excluded.role,content=excluded.content,timestamp=excluded.timestamp,
                    branch_id=excluded.branch_id,parent_id=excluded.parent_id,is_active=excluded.is_active,
                    sensitive=messages.sensitive OR excluded.sensitive,position=excluded.position").map_err(|e| e.to_string())?;
                for m in &c.messages {
                    insert
                        .execute(params![
                            m.id,
                            m.conversation_id,
                            m.role,
                            m.content,
                            m.timestamp,
                            m.branch_id,
                            m.parent_id,
                            m.is_active,
                            m.sensitive,
                            m.position
                        ])
                        .map_err(|e| e.to_string())?;
                }
            }
            // Old retained branches must still protect the whole conversation.
            tx.execute("UPDATE conversations SET sensitive = sensitive OR EXISTS(SELECT 1 FROM messages WHERE conversation_id=?1 AND sensitive=1) WHERE id=?1", [&c.id]).map_err(|e| e.to_string())?;
        }
        tx.execute("INSERT INTO search_index(conversation_id,content) SELECT c.id,c.title FROM conversations c JOIN import_ids i ON i.id=c.id", []).map_err(|e| e.to_string())?;
        tx.execute("INSERT INTO search_index(conversation_id,content) SELECT m.conversation_id,m.content FROM messages m JOIN import_ids i ON i.id=m.conversation_id", []).map_err(|e| e.to_string())?;
        tx.execute_batch("DROP TABLE import_ids; DROP TABLE import_message_ids;")
            .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(report)
    }

    pub fn list(
        &self,
        query: &str,
        include_restricted: bool,
        offset: u32,
    ) -> Result<Vec<Conversation>, String> {
        if query.len() > 1000 {
            return Err("Keep searches under 1,000 characters.".into());
        }
        // Literal terms prevent user input from becoming FTS operators or invalid FTS syntax.
        let terms = query
            .split_whitespace()
            .map(|s| format!("\"{}\"", s.replace('"', "\"\"")))
            .collect::<Vec<_>>()
            .join(" AND ");
        let search = if terms.is_empty() {
            ""
        } else {
            "AND c.id IN (SELECT conversation_id FROM search_index WHERE search_index MATCH ?3)"
        };
        let sql = format!("SELECT c.id,c.platform,c.title,c.created_at,c.updated_at,c.private,c.sensitive,
            (SELECT count(*) FROM messages m WHERE m.conversation_id=c.id),
            COALESCE((SELECT substr(content,1,180) FROM messages m WHERE m.conversation_id=c.id ORDER BY is_active DESC,position LIMIT 1),'No readable text messages')
            FROM conversations c WHERE (?1 OR (c.private=0 AND c.sensitive=0)) {search}
            ORDER BY c.updated_at DESC,c.id LIMIT 100 OFFSET ?2");
        let mut statement = self.connection.prepare(&sql).map_err(|e| e.to_string())?;
        let row = |r: &rusqlite::Row<'_>| {
            Ok(Conversation {
                id: r.get(0)?,
                platform: r.get(1)?,
                title: r.get(2)?,
                created_at: r.get(3)?,
                updated_at: r.get(4)?,
                private: r.get(5)?,
                sensitive: r.get(6)?,
                message_count: r.get(7)?,
                preview: r.get(8)?,
            })
        };
        let items = if terms.is_empty() {
            statement.query_map(params![include_restricted, offset], row)
        } else {
            statement.query_map(params![include_restricted, offset, terms], row)
        }
        .map_err(|e| e.to_string())?;
        items
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn messages(&self, id: &str, include_restricted: bool) -> Result<Vec<Message>, String> {
        let allowed: bool = self.connection.query_row("SELECT EXISTS(SELECT 1 FROM conversations WHERE id=?1 AND (?2 OR (private=0 AND sensitive=0)))", params![id, include_restricted], |r| r.get(0)).map_err(|e| e.to_string())?;
        if !allowed {
            return Err(
                "This conversation is private, flagged as sensitive, or no longer available."
                    .into(),
            );
        }
        let mut statement = self.connection.prepare("SELECT id,conversation_id,role,content,timestamp,branch_id,parent_id,is_active,sensitive,position FROM messages WHERE conversation_id=?1 ORDER BY position,id").map_err(|e| e.to_string())?;
        let items = statement
            .query_map([id], |r| {
                Ok(Message {
                    id: r.get(0)?,
                    conversation_id: r.get(1)?,
                    role: r.get(2)?,
                    content: r.get(3)?,
                    timestamp: r.get(4)?,
                    branch_id: r.get(5)?,
                    parent_id: r.get(6)?,
                    is_active: r.get(7)?,
                    sensitive: r.get(8)?,
                    position: r.get(9)?,
                })
            })
            .map_err(|e| e.to_string())?;
        items
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn mark_private(&self, id: &str, private: bool) -> Result<(), String> {
        let changed = self
            .connection
            .execute(
                "UPDATE conversations SET private=?1 WHERE id=?2",
                params![private, id],
            )
            .map_err(|e| e.to_string())?;
        if changed == 0 {
            return Err("Conversation not found.".into());
        }
        Ok(())
    }

    pub fn stats(&self) -> Result<Stats, String> {
        self.connection.query_row("SELECT count(*),COALESCE(sum(private),0),COALESCE(sum(sensitive),0),(SELECT count(*) FROM messages) FROM conversations", [], |r| Ok(Stats {
            conversations:r.get(0)?,private:r.get(1)?,sensitive:r.get(2)?,messages:r.get(3)?,database_path:self.path.clone(),
        })).map_err(|e| e.to_string())
    }
}
