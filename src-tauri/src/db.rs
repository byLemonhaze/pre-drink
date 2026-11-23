use crate::model::{Conversation, ImportReport, Message, ParsedConversation, Stats};
use rusqlite::{params, Connection, OptionalExtension};
use std::{path::Path, time::Duration};

pub struct Vault {
    pub(crate) connection: Connection,
    pub(crate) path: String,
    pub(crate) privacy_on_open: crate::privacy_recheck::PrivacyUpdate,
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
        if version > 9 {
            return Err("This vault needs a newer Pre-Drink version.".into());
        }
        if version == 1 {
            connection.execute_batch("BEGIN IMMEDIATE; ALTER TABLE memory ADD COLUMN had_source INTEGER NOT NULL DEFAULT 0; UPDATE memory SET had_source=1 WHERE source_message_id IS NOT NULL; PRAGMA user_version=2; COMMIT;").map_err(|e| e.to_string())?;
        }
        connection
            .execute_batch(include_str!("schema.sql"))
            .map_err(|e| e.to_string())?;
        let mut migration_update = crate::privacy_recheck::PrivacyUpdate::default();
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
                                .map(|changed| migration_update.flagged += changed)
                                .map_err(|e| e.to_string())?;
                        }
                    }
                }
                migration_update.flagged += connection.execute("UPDATE conversations SET sensitive=1 WHERE sensitive=0 AND EXISTS(SELECT 1 FROM messages WHERE conversation_id=conversations.id AND sensitive=1)", []).map_err(|e| e.to_string())?;
                migration_update.approvals_reset = connection.query_row("SELECT (SELECT count(*) FROM memory WHERE approved=1) + (SELECT count(*) FROM briefs WHERE approved=1)", [], |row| row.get::<_, i64>(0)).map_err(|e| e.to_string())? as usize;
                migration_update.legacy_review_upgrade = matches!(version, 1 | 2);
                connection.execute_batch("UPDATE memory SET approved=0; UPDATE briefs SET approved=0; PRAGMA user_version=3;").map_err(|e| e.to_string())
            })();
            if let Err(error) = refreshed {
                let _ = connection.execute_batch("ROLLBACK;");
                return Err(error);
            }
            connection
                .execute_batch("COMMIT;")
                .map_err(|e| e.to_string())?;
        }
        // SQLite's supported table-rebuild pattern. No agent is connected during
        // initialization; FK enforcement is restored before returning the vault.
        if matches!(version, 4 | 5) {
            connection
                .execute_batch("PRAGMA foreign_keys=OFF;")
                .map_err(|e| e.to_string())?;
            let result = (|| -> Result<(), String> {
                let tx = connection
                    .unchecked_transaction()
                    .map_err(|e| e.to_string())?;
                if version == 4 {
                    tx.execute_batch(include_str!("provider_migration.sql"))
                        .map_err(|e| e.to_string())?;
                }
                tx.execute_batch(include_str!("provider_jobs_migration.sql"))
                    .map_err(|e| e.to_string())?;
                tx.execute_batch(include_str!("curation.sql"))
                    .map_err(|e| e.to_string())?;
                let violations: i64 = tx
                    .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| {
                        r.get(0)
                    })
                    .map_err(|e| e.to_string())?;
                if violations != 0 {
                    return Err(
                        "Provider upgrade found invalid references; nothing was changed.".into(),
                    );
                }
                tx.execute_batch("PRAGMA user_version=6;")
                    .map_err(|e| e.to_string())?;
                tx.commit().map_err(|e| e.to_string())
            })();
            connection
                .execute_batch("PRAGMA foreign_keys=ON;")
                .map_err(|e| e.to_string())?;
            result?;
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
        }
        if version < 7 {
            // Older revisions used one conflict bit for agents and owner holds.
            // Preserve all existing holds conservatively; only the owner can clear.
            tx.execute("INSERT OR IGNORE INTO curation_owner_holds SELECT id FROM curation_notes WHERE conflict=1", [])
                .map_err(|e| e.to_string())?;
        }
        tx.execute_batch(include_str!("import_jobs.sql"))
            .map_err(|e| e.to_string())?;
        tx.execute_batch(include_str!("agent_activity.sql"))
            .map_err(|e| e.to_string())?;
        tx.execute_batch("PRAGMA user_version=9;")
            .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        let mut vault = Self {
            connection,
            path,
            privacy_on_open: Default::default(),
        };
        vault.enforce_sensitive_rules()?;
        // The legacy migration already applied these changes before the current
        // classifier ran. Preserve its counts for the successful unlock notice.
        vault.privacy_on_open.flagged += migration_update.flagged;
        vault.privacy_on_open.approvals_reset += migration_update.approvals_reset;
        vault.privacy_on_open.legacy_review_upgrade = migration_update.legacy_review_upgrade;
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
            #[cfg(test)]
            if crate::storage_failure::take_fault("backup_export_full") {
                // A real SQLite capacity error, restricted to this attached copy.
                self.connection
                    .execute_batch("PRAGMA backup.max_page_count=1;")
                    .unwrap();
            }
            let exported = self
                .connection
                .execute_batch("SELECT sqlcipher_export('backup'); PRAGMA backup.user_version=9;");
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
                "import_job",
                "import_items",
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
            let flush = || {
                #[cfg(test)]
                crate::storage_failure::io_fault("backup_file_sync")?;
                std::fs::File::open(&staging).and_then(|f| f.sync_all())
            };
            flush().map_err(|_| "Could not flush the backup to disk.")?;
            // Same-folder hard link publishes atomically and refuses to overwrite a raced-in file.
            let publish = || {
                #[cfg(test)]
                crate::storage_failure::io_fault("backup_publish")?;
                std::fs::hard_link(&staging, target)
            };
            publish().map_err(|_| {
                "Couldn't publish the backup. Choose an unused filename on a local filesystem."
            })?;
            // Flushing the file alone does not persist its new directory entry.
            // After publication, preserve the verified copy even if this fails;
            // do not grant a cleanup receipt for uncertain durability.
            let sync_parent = || {
                #[cfg(test)]
                crate::storage_failure::io_fault("backup_directory_sync")?;
                std::fs::File::open(parent).and_then(|f| f.sync_all())
            };
            sync_parent().map_err(|_| "The backup file was created, but its folder could not be flushed to disk. Keep your original vault and try another backup filename on a local filesystem before removing old copies.")?;
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
        report: ImportReport,
    ) -> Result<ImportReport, String> {
        let tx = self.connection.transaction().map_err(|e| e.to_string())?;
        let report = Self::import_transaction(&tx, conversations, report)?;
        #[cfg(test)]
        crate::storage_failure::import_checkpoint(&self.path, &tx);
        tx.commit().map_err(|e| e.to_string())?;
        Ok(report)
    }

    pub(crate) fn import_transaction(
        tx: &rusqlite::Transaction<'_>,
        conversations: &[ParsedConversation],
        mut report: ImportReport,
    ) -> Result<ImportReport, String> {
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
                VALUES (?1,?7,?2,?3,?4,?5,?6) ON CONFLICT(id) DO UPDATE SET
                title=excluded.title, created_at=excluded.created_at, updated_at=excluded.updated_at,
                sensitive=conversations.sensitive OR excluded.sensitive, raw_json=excluded.raw_json",
                params![c.id,c.title,c.created_at,c.updated_at,c.sensitive,c.raw_json,c.platform]).map_err(|e| e.to_string())?;
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
        let tx = self
            .connection
            .unchecked_transaction()
            .map_err(|e| e.to_string())?;
        let previous: Option<bool> = tx
            .query_row("SELECT private FROM conversations WHERE id=?1", [id], |r| {
                r.get(0)
            })
            .optional()
            .map_err(|e| e.to_string())?;
        let previous = previous.ok_or("Conversation not found.")?;
        if previous == private {
            return Ok(());
        }
        tx.execute(
            "UPDATE conversations SET private=?1 WHERE id=?2",
            params![private, id],
        )
        .map_err(|e| e.to_string())?;
        // A privacy round trip must not revive an old release or review. Bind
        // manual notes and their briefs to the transition, like curated evidence.
        // Advance draft versions too: an unsubmitted pre-change review is stale.
        tx.execute("UPDATE briefs SET approved=0,updated_at=max(updated_at+0.000001,unixepoch('subsec'))
            WHERE id IN (SELECT brief_id FROM brief_sources WHERE memory_id IN (
                SELECT id FROM memory WHERE source_message_id IN (SELECT id FROM messages WHERE conversation_id=?1)
                OR id IN (SELECT s.note_id FROM curation_sources s JOIN messages msg ON msg.id=s.message_id WHERE msg.conversation_id=?1)))", [id])
            .map_err(|e| e.to_string())?;
        tx.execute("UPDATE memory SET approved=0,updated_at=max(updated_at+0.000001,unixepoch('subsec'))
            WHERE source_message_id IN (SELECT id FROM messages WHERE conversation_id=?1)
            OR id IN (SELECT s.note_id FROM curation_sources s JOIN messages msg ON msg.id=s.message_id WHERE msg.conversation_id=?1)", [id])
            .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())
    }

    pub fn stats(&self) -> Result<Stats, String> {
        self.connection.query_row("SELECT count(*),COALESCE(sum(private),0),COALESCE(sum(sensitive),0),(SELECT count(*) FROM messages) FROM conversations", [], |r| Ok(Stats {
            conversations:r.get(0)?,private:r.get(1)?,sensitive:r.get(2)?,messages:r.get(3)?,database_path:self.path.clone(),
        })).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod provider_upgrade_tests {
    use super::*;
    #[test]
    fn v4_provider_upgrade_preserves_grants_policy_and_all_child_records() {
        check_provider_upgrade(4);
    }
    #[test]
    fn v5_provider_upgrade_allows_claude_jobs_and_preserves_existing_state() {
        check_provider_upgrade(5);
    }
    fn check_provider_upgrade(version: i64) {
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch(include_str!("schema.sql")).unwrap();
        c.execute_batch(include_str!("../tests/fixtures/migrations/curation-v4.sql"))
            .unwrap();
        c.execute_batch("INSERT INTO agent_clients VALUES ('client','Fictional curator','hash',1,0);
          INSERT INTO agent_scopes VALUES ('client','grok',1);
          UPDATE curation_policy SET publish_session=1 WHERE provider='grok';
          INSERT INTO conversations(id,platform,title,raw_json) VALUES ('source','grok','Fictional garden','{}');
          INSERT INTO messages(id,conversation_id,role,content,position) VALUES ('message','source','user','Garden uses clay.',0);
          INSERT INTO curation_notes VALUES ('note','grok','project','Garden uses clay.','archive','',1,0,0,0,1);
          INSERT INTO curation_sources VALUES ('note','message',1);
          INSERT INTO curation_revisions VALUES ('note',1,'client','fictional preserved snapshot',1);
          INSERT INTO curation_publications VALUES ('note',1,'owner_review');
          INSERT INTO memory(id,type,content,approved,created_at,updated_at) VALUES ('note','project','Garden uses clay.',1,1,1);
          INSERT INTO curation_jobs VALUES ('job','grok','Garden',1,'0','running',1);
          INSERT INTO curation_job_sources VALUES ('job',1,'message',1);
          PRAGMA user_version=4;").unwrap();
        if version == 5 {
            c.execute_batch("PRAGMA foreign_keys=OFF;").unwrap();
            c.execute_batch(include_str!("../tests/fixtures/migrations/provider-v5.sql"))
                .unwrap();
            c.execute_batch(include_str!("curation.sql")).unwrap();
            c.execute_batch("PRAGMA user_version=5; PRAGMA foreign_keys=ON;")
                .unwrap();
        }
        let v = Vault::initialize(c, ":memory:".into()).unwrap();
        assert_eq!(
            v.connection
                .query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            9
        );
        assert_eq!(
            v.connection
                .query_row("PRAGMA foreign_keys", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            1
        );
        v.verify().unwrap();
        assert_eq!(v.clients().unwrap()[0].provider.as_deref(), Some("grok"));
        assert!(v.clients().unwrap()[0].can_curate);
        assert_eq!(
            v.connection
                .query_row(
                    "SELECT snapshot FROM curation_revisions WHERE note_id='note'",
                    [],
                    |r| r.get::<_, String>(0)
                )
                .unwrap(),
            "fictional preserved snapshot"
        );
        assert_eq!(
            v.connection
                .query_row(
                    "SELECT publish_session FROM curation_policy WHERE provider='grok'",
                    [],
                    |r| r.get::<_, i64>(0)
                )
                .unwrap(),
            1
        );
        for table in [
            "curation_notes",
            "curation_sources",
            "curation_revisions",
            "curation_publications",
            "curation_jobs",
            "curation_job_sources",
        ] {
            assert_eq!(
                v.connection
                    .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r
                        .get::<_, i64>(0))
                    .unwrap(),
                1
            );
        }
        assert_eq!(v.memories(None, "", true).unwrap().len(), 1);
        let claude = v
            .create_scoped_client("Fictional Claude", Some("claude"), true)
            .unwrap();
        let op = crate::curation::CurationOperation::SaveJob {
            id: None,
            expected_revision: 0,
            label: "Garden review".into(),
            checkpoint: 0,
            status: "running".into(),
            request_key: "claude-job".into(),
        };
        let job = v
            .save_curation_job("claude", &claude.client.id, &op)
            .unwrap();
        assert_eq!(job["revision"], 1);
        assert_eq!(job["source_count"], 0);
        assert_eq!(v.curation_policies().unwrap().as_array().unwrap().len(), 3);
        v.connection
            .execute_batch("DELETE FROM agent_clients WHERE id='client'")
            .unwrap();
        assert_eq!(
            v.connection
                .query_row(
                    "SELECT count(*) FROM agent_scopes WHERE client_id='client'",
                    [],
                    |r| r.get::<_, i64>(0)
                )
                .unwrap(),
            0
        );
    }
}

#[cfg(test)]
mod activity_upgrade_tests {
    use super::*;
    fn version_eight() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch(include_str!("schema.sql")).unwrap();
        c.execute_batch(include_str!("curation.sql")).unwrap();
        c.execute_batch(include_str!("import_jobs.sql")).unwrap();
        c.execute_batch("INSERT INTO agent_clients VALUES('fictional','Fictional reader','fictional-hash',0,1); INSERT INTO memory(id,type,content,approved,created_at,updated_at) VALUES('note','project','Garden bench',1,1,1); PRAGMA user_version=8;").unwrap();
        c
    }
    #[test]
    fn version_eight_adds_controls_without_new_permissions_or_approval_resets() {
        let v = Vault::initialize(version_eight(), ":memory:".into()).unwrap();
        let client = v.clients().unwrap().remove(0);
        assert!(
            !client.paused
                && !client.allow_chats
                && !client.can_curate
                && client.last_request_at.is_none()
        );
        assert_eq!(v.memories(None, "", true).unwrap().len(), 1);
        assert!(v.agent_activity(None).unwrap().is_empty());
        assert_eq!(
            v.connection
                .query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            9
        );
        v.verify().unwrap();
    }
    #[test]
    fn activity_upgrade_failure_rolls_back_version_and_new_tables() {
        let c = version_eight();
        // Retain a second handle to inspect the file after initialization fails.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("fictional-old.sqlite3");
        c.execute("VACUUM INTO ?1", [path.to_str().unwrap()])
            .unwrap();
        drop(c);
        let c = Connection::open(&path).unwrap();
        c.execute_batch("CREATE VIEW agent_activity AS SELECT 1 AS id;")
            .unwrap();
        assert!(Vault::initialize(c, path.to_string_lossy().into_owned()).is_err());
        let c = Connection::open(&path).unwrap();
        assert_eq!(
            c.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            8
        );
        assert_eq!(
            c.query_row(
                "SELECT count(*) FROM sqlite_master WHERE name='agent_controls'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
            0
        );
        assert_eq!(
            c.query_row("SELECT approved FROM memory WHERE id='note'", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            1
        );
        c.execute_batch("DROP VIEW agent_activity;").unwrap();
        let v = Vault::initialize(c, ":memory:".into()).unwrap();
        assert_eq!(v.clients().unwrap().len(), 1);
    }
}
