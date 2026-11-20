//! Owner-initiated reclassification. Merely opening/updating never clears old flags.
use crate::{db::Vault, privacy::is_sensitive};
use rusqlite::params;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashSet;

#[derive(Serialize)]
pub struct PrivacyRecheck {
    pub flagged: usize,
    pub cleared: usize,
    pub approvals_to_reset: i64,
    pub review_hash: String,
}
struct Change {
    table: &'static str,
    id: String,
    sensitive: bool,
}
impl Vault {
    fn privacy_recheck_plan(&self) -> Result<(PrivacyRecheck, Vec<Change>), String> {
        let mut changes = Vec::new();
        let mut digest = Sha256::new();
        digest.update(b"pre-drink-privacy-recheck-v1");
        let mut sensitive_conversations = HashSet::new();
        let mut flagged = 0;
        let mut cleared = 0;
        // Messages first: retained/inactive branches also restrict their conversation.
        for (table, column, parent) in [
            ("messages", "content", "conversation_id"),
            ("conversations", "title", "id"),
            ("memory", "content", "''"),
            ("briefs", "title || ' ' || content", "''"),
        ] {
            let mut stmt = self
                .connection
                .prepare(&format!(
                    "SELECT id,{column},sensitive,{parent} FROM {table} ORDER BY id"
                ))
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, bool>(2)?,
                        r.get::<_, String>(3)?,
                    ))
                })
                .map_err(|e| e.to_string())?;
            for row in rows {
                let (id, content, previous, parent) = row.map_err(|e| e.to_string())?;
                // Length-delimited serialization binds the preview to the saved contents.
                digest.update(
                    serde_json::to_vec(&(table, &id, &content, previous, &parent))
                        .map_err(|e| e.to_string())?,
                );
                let sensitive = is_sensitive(&content)
                    || (table == "conversations" && sensitive_conversations.contains(&id));
                if table == "messages" && sensitive {
                    sensitive_conversations.insert(parent);
                }
                if previous != sensitive {
                    if sensitive {
                        flagged += 1;
                    } else {
                        cleared += 1;
                    }
                    changes.push(Change {
                        table,
                        id,
                        sensitive,
                    });
                }
            }
        }
        let approved: i64 = self.connection.query_row("SELECT (SELECT count(*) FROM memory WHERE approved=1)+(SELECT count(*) FROM briefs WHERE approved=1)", [], |r| r.get(0)).map_err(|e| e.to_string())?;
        digest.update(approved.to_be_bytes());
        Ok((
            PrivacyRecheck {
                flagged,
                cleared,
                approvals_to_reset: if changes.is_empty() { 0 } else { approved },
                review_hash: hex::encode(digest.finalize()),
            },
            changes,
        ))
    }

    pub fn privacy_recheck_preview(&self) -> Result<PrivacyRecheck, String> {
        self.privacy_recheck_plan().map(|(review, _)| review)
    }

    pub fn apply_privacy_recheck(&mut self, review_hash: &str) -> Result<PrivacyRecheck, String> {
        let (review, changes) = self.privacy_recheck_plan()?;
        if review.review_hash != review_hash {
            return Err("Your vault changed. Review the recheck again before applying it.".into());
        }
        let tx = self.connection.transaction().map_err(|e| e.to_string())?;
        for change in &changes {
            tx.execute(
                &format!("UPDATE {} SET sensitive=?1 WHERE id=?2", change.table),
                params![change.sensitive, change.id],
            )
            .map_err(|e| e.to_string())?;
        }
        if !changes.is_empty() {
            // Clearing an inherited restriction must not make an old approval live.
            // Manual private flags and original content/provenance are never modified.
            tx.execute_batch("UPDATE memory SET approved=0; UPDATE briefs SET approved=0;")
                .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(review)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn legacy_fixture() -> Vault {
        let v = Vault::memory().unwrap();
        v.connection.execute_batch("INSERT INTO conversations(id,platform,title,sensitive,private,raw_json) VALUES ('c','chatgpt','Timestamp discussion',1,1,'{}');
            INSERT INTO messages(id,conversation_id,role,content,sensitive,position) VALUES ('m','c','user','Created at 1795132800',1,0);
            INSERT INTO memory(id,type,content,source_message_id,had_source,approved,created_at,updated_at) VALUES ('note','fact','Keep the project local.','m',1,1,1,1);
            INSERT INTO briefs(id,title,content,approved,updated_at) VALUES ('brief','Spec','Keep the project local.',1,1);
            INSERT INTO brief_sources(brief_id,memory_id,reviewed_at) VALUES ('brief','note',1);").unwrap();
        v
    }

    #[test]
    fn recheck_is_explicit_preserves_privacy_and_revokes_prior_approvals() {
        let mut v = legacy_fixture();
        let preview = v.privacy_recheck_preview().unwrap();
        assert_eq!(
            (preview.cleared, preview.flagged, preview.approvals_to_reset),
            (2, 0, 2)
        );
        assert_eq!(v.stats().unwrap().sensitive, 1); // preview cannot mutate
        v.apply_privacy_recheck(&preview.review_hash).unwrap();
        let stats = v.stats().unwrap();
        assert_eq!((stats.sensitive, stats.private), (0, 1));
        assert!(v.memories(None, "", true).unwrap().is_empty());
        assert!(!v.memories(None, "", false).unwrap()[0].approved);
        assert!(!v.briefs(false, "").unwrap()[0].approved);
        assert_eq!(
            v.messages("c", true).unwrap()[0].content,
            "Created at 1795132800"
        );
        assert_eq!(
            v.memories(None, "", false).unwrap()[0]
                .source_message_id
                .as_deref(),
            Some("m")
        );
        let next = v.privacy_recheck_preview().unwrap();
        assert_eq!(
            (next.cleared, next.flagged, next.approvals_to_reset),
            (0, 0, 0)
        );
    }

    #[test]
    fn retained_secret_branch_blocks_conversation_and_cannot_be_overridden() {
        let mut v = legacy_fixture();
        let secret = format!("sk-{}", "aB".repeat(24));
        v.connection.execute("INSERT INTO messages(id,conversation_id,role,content,is_active,sensitive,position) VALUES ('old','c','user',?1,0,0,1)", [secret]).unwrap();
        let preview = v.privacy_recheck_preview().unwrap();
        assert_eq!(preview.flagged, 1);
        v.apply_privacy_recheck(&preview.review_hash).unwrap();
        assert_eq!(v.stats().unwrap().sensitive, 1);
        v.mark_private("c", false).unwrap();
        assert!(v.list("", false, 0).unwrap().is_empty());
        assert!(v.approval_review("memory", "note").unwrap().restricted);
    }

    #[test]
    fn reimport_does_not_silently_clear_legacy_restrictions() {
        let mut v = legacy_fixture();
        let parsed = crate::model::ParsedConversation {
            id: "c".into(),
            title: "Timestamp discussion".into(),
            created_at: None,
            updated_at: None,
            sensitive: false,
            raw_json: "{}".into(),
            messages: vec![crate::model::Message {
                id: "m".into(),
                conversation_id: "c".into(),
                role: "user".into(),
                content: "Created at 1795132800".into(),
                timestamp: None,
                branch_id: None,
                parent_id: None,
                is_active: true,
                sensitive: false,
                position: 0,
            }],
        };
        v.import(&[parsed], crate::model::ImportReport::default())
            .unwrap();
        assert_eq!(v.stats().unwrap().sensitive, 1);
        assert!(v.messages("c", true).unwrap()[0].sensitive);
        assert!(v.approval_review("memory", "note").unwrap().restricted);
    }

    #[test]
    fn stale_preview_and_partial_changes_fail_closed() {
        let mut v = legacy_fixture();
        let preview = v.privacy_recheck_preview().unwrap();
        v.connection
            .execute(
                "UPDATE messages SET content='Changed at 1795132801' WHERE id='m'",
                [],
            )
            .unwrap();
        assert!(v.apply_privacy_recheck(&preview.review_hash).is_err());
        assert_eq!(v.stats().unwrap().sensitive, 1);
        assert!(!v.memories(None, "", false).unwrap()[0].approved);
        // Recreate a legacy approval to check rollback independently of the
        // evidence-change trigger, which already revoked it above.
        v.connection
            .execute("UPDATE memory SET approved=1", [])
            .unwrap();
        let preview = v.privacy_recheck_preview().unwrap();
        v.connection.execute_batch("CREATE TRIGGER fail_recheck BEFORE UPDATE ON conversations BEGIN SELECT RAISE(ABORT,'synthetic failure'); END;").unwrap();
        assert!(v.apply_privacy_recheck(&preview.review_hash).is_err());
        assert!(v.messages("c", true).unwrap()[0].sensitive);
        assert!(v.memories(None, "", false).unwrap()[0].approved);
    }
}
