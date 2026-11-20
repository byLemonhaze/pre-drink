//! Owner-initiated reclassification. Merely opening/updating never clears old flags.
use crate::{db::Vault, privacy::is_sensitive};
use rusqlite::params;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashSet;

#[derive(Serialize)]
pub struct ClearCandidate {
    pub key: String,
    pub kind: String,
    pub content: String,
}
#[derive(Serialize)]
pub struct PrivacyRecheck {
    pub flagged: usize,
    pub cleared: usize,
    pub approvals_to_reset: i64,
    pub review_hash: String,
    pub clear_candidates: Vec<ClearCandidate>,
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
        let mut clear_candidates = Vec::new();
        digest.update(self.connection.total_changes().to_be_bytes());
        // Messages first: retained/inactive branches also restrict their conversation.
        for (table, column, parent) in [
            ("messages", "content", "conversation_id"),
            ("conversations", "title", "id"),
            ("memory", "content", "''"),
            ("curation_notes", "content || ' ' || session_label", "''"),
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
                        // Never offer a truncated item for clearance. Oversized items
                        // stay restricted; review/edit them in their original view.
                        if clear_candidates.len() < 100 && content.len() <= 16_000 {
                            clear_candidates.push(ClearCandidate {
                                key: format!("{table}:{id}"),
                                kind: table.into(),
                                content: content.clone(),
                            });
                        }
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
                clear_candidates,
                approvals_to_reset: if changes.is_empty() { 0 } else { approved },
                review_hash: hex::encode(digest.finalize()),
            },
            changes,
        ))
    }

    /// Updated rules can add restrictions on open, never silently remove them.
    pub(crate) fn enforce_sensitive_rules(&mut self) -> Result<(), String> {
        let (_, changes) = self.privacy_recheck_plan()?;
        let tx = self.connection.transaction().map_err(|e| e.to_string())?;
        let mut changed = 0;
        for change in changes.into_iter().filter(|c| c.sensitive) {
            changed += tx
                .execute(
                    &format!("UPDATE {} SET sensitive=1 WHERE id=?1", change.table),
                    [change.id],
                )
                .map_err(|e| e.to_string())?;
        }
        changed += tx.execute("UPDATE conversations SET sensitive=1 WHERE sensitive=0 AND EXISTS(SELECT 1 FROM messages WHERE conversation_id=conversations.id AND sensitive=1)", []).map_err(|e| e.to_string())?;
        if changed > 0 {
            tx.execute_batch("UPDATE memory SET approved=0; UPDATE briefs SET approved=0;")
                .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())
    }

    pub fn privacy_recheck_preview(&self) -> Result<PrivacyRecheck, String> {
        self.privacy_recheck_plan().map(|(review, _)| review)
    }

    pub fn apply_privacy_recheck(
        &mut self,
        review_hash: &str,
        clear_ids: &[String],
    ) -> Result<PrivacyRecheck, String> {
        let (mut review, changes) = self.privacy_recheck_plan()?;
        if review.review_hash != review_hash {
            return Err("Your vault changed. Review the recheck again before applying it.".into());
        }
        let selected: HashSet<_> = clear_ids.iter().collect();
        if selected.len() != clear_ids.len()
            || clear_ids
                .iter()
                .any(|id| !review.clear_candidates.iter().any(|c| &c.key == id))
        {
            return Err("Select only items shown in the current review.".into());
        }
        let changes: Vec<_> = changes
            .into_iter()
            .filter(|c| c.sensitive || selected.contains(&format!("{}:{}", c.table, c.id)))
            .collect();
        review.cleared = changes.iter().filter(|c| !c.sensitive).count();
        review.approvals_to_reset = if changes.is_empty() {
            0
        } else {
            review.approvals_to_reset
        };
        let tx = self.connection.transaction().map_err(|e| e.to_string())?;
        for change in &changes {
            if change.table == "conversations" && !change.sensitive {
                let blocked: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM messages WHERE conversation_id=?1 AND sensitive=1)", [&change.id], |r| r.get(0)).map_err(|e| e.to_string())?;
                if blocked {
                    return Err("Review and clear flagged messages before clearing their conversation. Nothing was changed.".into());
                }
            }
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
        v.apply_privacy_recheck(
            &preview.review_hash,
            &preview
                .clear_candidates
                .iter()
                .map(|c| c.key.clone())
                .collect::<Vec<_>>(),
        )
        .unwrap();
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
        v.apply_privacy_recheck(
            &preview.review_hash,
            &preview
                .clear_candidates
                .iter()
                .map(|c| c.key.clone())
                .collect::<Vec<_>>(),
        )
        .unwrap();
        assert_eq!(v.stats().unwrap().sensitive, 1);
        v.mark_private("c", false).unwrap();
        assert!(v.list("", false, 0).unwrap().is_empty());
        assert!(v.approval_review("memory", "note").unwrap().restricted);
    }

    #[test]
    fn clearing_requires_each_item_and_rejects_hidden_or_secret_ids() {
        let mut v = legacy_fixture();
        let p = v.privacy_recheck_preview().unwrap();
        let result = v.apply_privacy_recheck(&p.review_hash, &[]).unwrap();
        assert_eq!(result.cleared, 0);
        assert!(v.messages("c", true).unwrap()[0].sensitive);
        let p = v.privacy_recheck_preview().unwrap();
        assert!(v
            .apply_privacy_recheck(&p.review_hash, &["conversations:c".into()])
            .is_err());
        assert!(v
            .apply_privacy_recheck(&p.review_hash, &["messages:unknown".into()])
            .is_err());
        let p = v.privacy_recheck_preview().unwrap();
        v.apply_privacy_recheck(&p.review_hash, &["messages:m".into()])
            .unwrap();
        assert_eq!(v.stats().unwrap().sensitive, 1); // parent still restricted
        assert!(!v.messages("c", true).unwrap()[0].sensitive);
        v.connection.execute("UPDATE messages SET content='my wifi password is fictional-lilac', sensitive=1 WHERE id='m'", []).unwrap();
        let p = v.privacy_recheck_preview().unwrap();
        assert!(p.clear_candidates.is_empty());
        assert!(v
            .apply_privacy_recheck(&p.review_hash, &["messages:m".into()])
            .is_err());
        v.mark_private("c", false).unwrap();
        assert!(v.list("", false, 0).unwrap().is_empty());
    }

    #[test]
    fn stronger_rules_restrict_existing_data_without_clearing_legacy_flags() {
        let mut v = legacy_fixture();
        v.connection.execute_batch("INSERT INTO conversations(id,platform,title,sensitive,raw_json) VALUES ('new','chatgpt','Example',0,'{}'); INSERT INTO messages(id,conversation_id,role,content,sensitive,position) VALUES ('n','new','user','passcode is 1234',0,0);").unwrap();
        v.enforce_sensitive_rules().unwrap();
        assert_eq!(v.stats().unwrap().sensitive, 2);
        assert!(v.messages("c", true).unwrap()[0].sensitive);
        assert!(v.messages("new", true).unwrap()[0].sensitive);
        assert!(!v.memories(None, "", false).unwrap()[0].approved);
        assert!(v.list("", false, 0).unwrap().is_empty());
    }

    #[test]
    fn only_complete_visible_candidates_can_be_cleared() {
        let mut v = legacy_fixture();
        for n in 0..101 {
            v.connection.execute("INSERT INTO memory(id,type,content,sensitive,created_at,updated_at) VALUES (?1,'fact','Fictional note',1,1,1)", [format!("extra-{n:03}")]).unwrap();
        }
        v.connection.execute("INSERT INTO memory(id,type,content,sensitive,created_at,updated_at) VALUES ('oversized','fact',?1,1,1,1)", ["x".repeat(16_001)]).unwrap();
        let p = v.privacy_recheck_preview().unwrap();
        assert_eq!(p.clear_candidates.len(), 100);
        assert!(v
            .apply_privacy_recheck(&p.review_hash, &["memory:extra-100".into()])
            .is_err());
        assert!(v
            .apply_privacy_recheck(&p.review_hash, &["memory:oversized".into()])
            .is_err());
        assert!(v
            .apply_privacy_recheck(&p.review_hash, &["messages:m".into(), "messages:m".into()])
            .is_err());
    }

    #[test]
    fn opening_existing_vault_restricts_new_secret_shapes_before_agent_reads() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault.enc.sqlite3");
        let password = "fictional migration verification phrase";
        let v = Vault::create(&path, password).unwrap();
        // Model a schema-v3 file made by the prior rules, including FTS content.
        v.connection.execute_batch("INSERT INTO conversations(id,platform,title,sensitive,raw_json) VALUES ('c','chatgpt','Fictional garden',0,'{}'); INSERT INTO messages(id,conversation_id,role,content,sensitive,position) VALUES ('m','c','user','my wifi password is fictional-lilac',0,0); INSERT INTO search_index(conversation_id,content) VALUES ('c','my wifi password is fictional-lilac');").unwrap();
        let client = v.create_client("Synthetic chat client", true).unwrap();
        drop(v);
        let mut v = Vault::open(&path, password).unwrap();
        let p = v.privacy_recheck_preview().unwrap();
        assert!(p.clear_candidates.is_empty());
        v.apply_privacy_recheck(&p.review_hash, &[]).unwrap();
        let mut request = crate::agents::AgentRequest {
            client_id: client.client.id,
            token: client.token,
            tool: "search_conversations".into(),
            query: "wifi".into(),
            limit: Some(10),
            conversation_id: String::new(),
            curation: None,
        };
        assert_eq!(v.agent_call(&request).unwrap(), serde_json::json!([]));
        request.tool = "get_context_pack".into();
        assert!(!v
            .agent_call(&request)
            .unwrap()
            .to_string()
            .contains("fictional-lilac"));
        assert!(v.messages("c", true).unwrap()[0].sensitive);
    }

    #[test]
    fn reimport_does_not_silently_clear_legacy_restrictions() {
        let mut v = legacy_fixture();
        let parsed = crate::model::ParsedConversation {
            platform: "chatgpt".into(),
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
        assert!(v
            .apply_privacy_recheck(
                &preview.review_hash,
                &preview
                    .clear_candidates
                    .iter()
                    .map(|c| c.key.clone())
                    .collect::<Vec<_>>()
            )
            .is_err());
        assert_eq!(v.stats().unwrap().sensitive, 1);
        assert!(!v.memories(None, "", false).unwrap()[0].approved);
        // Recreate a legacy approval to check rollback independently of the
        // evidence-change trigger, which already revoked it above.
        v.connection
            .execute("UPDATE memory SET approved=1", [])
            .unwrap();
        let preview = v.privacy_recheck_preview().unwrap();
        v.connection.execute_batch("CREATE TRIGGER fail_recheck BEFORE UPDATE ON conversations BEGIN SELECT RAISE(ABORT,'synthetic failure'); END;").unwrap();
        assert!(v
            .apply_privacy_recheck(
                &preview.review_hash,
                &preview
                    .clear_candidates
                    .iter()
                    .map(|c| c.key.clone())
                    .collect::<Vec<_>>()
            )
            .is_err());
        assert!(v.messages("c", true).unwrap()[0].sensitive);
        assert!(v.memories(None, "", false).unwrap()[0].approved);
    }
}
