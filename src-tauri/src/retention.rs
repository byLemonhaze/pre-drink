//! Explicit owner-reviewed deletion; derived records retain missing-source markers.
use crate::{db::Vault, session::Session};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RemovalKind {
    Conversation,
    Snapshots,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RemovalInput {
    pub kind: RemovalKind,
    pub conversation_id: Option<String>,
}
#[derive(Serialize)]
pub struct RemovalPreview {
    pub title: String,
    pub messages: i64,
    pub snapshots: usize,
    pub memories: i64,
    pub briefs: i64,
    pub review_hash: String,
    pub session_generation: u64,
}
impl Vault {
    fn removal_plan(&self, input: &RemovalInput) -> Result<(RemovalPreview, Vec<String>), String> {
        let mut p = RemovalPreview {
            title: String::new(),
            messages: 0,
            snapshots: 0,
            memories: 0,
            briefs: 0,
            review_hash: String::new(),
            session_generation: 0,
        };
        let ids = match input.kind {
            RemovalKind::Conversation => {
                let id = input
                    .conversation_id
                    .as_deref()
                    .filter(|id| id.len() <= 256)
                    .ok_or("Select a saved conversation.")?;
                p.title = self
                    .connection
                    .query_row("SELECT title FROM conversations WHERE id=?1", [id], |r| {
                        r.get(0)
                    })
                    .map_err(|_| "Conversation no longer exists.")?;
                p.messages = self
                    .connection
                    .query_row(
                        "SELECT count(*) FROM messages WHERE conversation_id=?1",
                        [id],
                        |r| r.get(0),
                    )
                    .map_err(|e| e.to_string())?;
                p.memories = self.connection.query_row("SELECT count(*) FROM (SELECT id FROM memory WHERE source_message_id IN (SELECT id FROM messages WHERE conversation_id=?1) UNION SELECT s.note_id FROM curation_sources s JOIN messages msg ON msg.id=s.message_id WHERE msg.conversation_id=?1)", [id], |r| r.get(0)).map_err(|e| e.to_string())?;
                p.briefs = self.connection.query_row("SELECT count(DISTINCT b.brief_id) FROM brief_sources b JOIN memory m ON m.id=b.memory_id WHERE m.source_message_id IN (SELECT id FROM messages WHERE conversation_id=?1) OR m.id IN (SELECT s.note_id FROM curation_sources s JOIN messages msg ON msg.id=s.message_id WHERE msg.conversation_id=?1)", [id], |r| r.get(0)).map_err(|e| e.to_string())?;
                let mut q = self
                    .connection
                    .prepare(
                        "SELECT id FROM archive_snapshots WHERE conversation_id=?1 ORDER BY id",
                    )
                    .map_err(|e| e.to_string())?;
                let rows = q
                    .query_map([id], |r| r.get::<_, String>(0))
                    .map_err(|e| e.to_string())?;
                rows.collect::<Result<Vec<_>, _>>()
                    .map_err(|e| e.to_string())?
            }
            RemovalKind::Snapshots => {
                if input.conversation_id.is_some() {
                    return Err("Snapshot cleanup applies to the archive as a whole.".into());
                }
                p.title = "Older raw archive snapshots".into();
                let mut q = self
                    .connection
                    .prepare("SELECT id,raw_json FROM conversations ORDER BY id")
                    .map_err(|e| e.to_string())?;
                // Read one raw conversation at a time rather than materializing
                // the entire decrypted archive just to identify old snapshots.
                let conversations = q
                    .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
                    .map_err(|e| e.to_string())?;
                let mut ids = Vec::new();
                let mut q = self.connection.prepare("SELECT id FROM archive_snapshots WHERE conversation_id=?1 AND id!=?2 ORDER BY id").map_err(|e| e.to_string())?;
                for conversation in conversations {
                    let (id, raw) = conversation.map_err(|e| e.to_string())?;
                    let current = hex::encode(Sha256::digest(raw.as_bytes()));
                    for row in q
                        .query_map(rusqlite::params![id, current], |r| r.get::<_, String>(0))
                        .map_err(|e| e.to_string())?
                    {
                        ids.push(row.map_err(|e| e.to_string())?);
                    }
                }
                ids.sort();
                ids
            }
        };
        p.snapshots = ids.len();
        let snapshot = serde_json::json!({"input":input,"changes":self.connection.total_changes(),"title":p.title,"messages":p.messages,"memories":p.memories,"briefs":p.briefs,"snapshots":ids});
        p.review_hash = hex::encode(Sha256::digest(
            serde_json::to_vec(&snapshot).map_err(|e| e.to_string())?,
        ));
        Ok((p, ids))
    }
    fn remove_reviewed(&mut self, input: &RemovalInput, review_hash: &str) -> Result<(), String> {
        let (preview, snapshots) = self.removal_plan(input)?;
        if preview.review_hash != review_hash {
            return Err("Your archive changed. Review the deletion again.".into());
        }
        let tx = self.connection.transaction().map_err(|e| e.to_string())?;
        for id in snapshots {
            tx.execute("DELETE FROM archive_snapshots WHERE id=?1", [id])
                .map_err(|e| e.to_string())?;
        }
        if let RemovalKind::Conversation = input.kind {
            let id = input
                .conversation_id
                .as_deref()
                .ok_or("Select a conversation.")?;
            tx.execute("UPDATE memory SET approved=0,updated_at=max(updated_at+0.000001,unixepoch('subsec')) WHERE source_message_id IN (SELECT id FROM messages WHERE conversation_id=?1) OR id IN (SELECT s.note_id FROM curation_sources s JOIN messages msg ON msg.id=s.message_id WHERE msg.conversation_id=?1)", [id]).map_err(|e| e.to_string())?;
            tx.execute("DELETE FROM search_index WHERE conversation_id=?1", [id])
                .map_err(|e| e.to_string())?;
            tx.execute("DELETE FROM conversations WHERE id=?1", [id])
                .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())
    }
}
impl Session {
    pub fn preview_removal(&mut self, input: &RemovalInput) -> Result<RemovalPreview, String> {
        let (mut p, _) = self.access()?.removal_plan(input)?;
        p.session_generation = self.generation;
        self.touch();
        Ok(p)
    }
    pub fn apply_removal(
        &mut self,
        input: &RemovalInput,
        review_hash: &str,
        generation: u64,
    ) -> Result<(), String> {
        self.sharing = false;
        if generation != self.generation {
            return Err("The vault session changed. Review the deletion again.".into());
        }
        self.access()?.remove_reviewed(input, review_hash)?;
        self.touch();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_deletion_rolls_back_snapshots_and_search() {
        let mut v = Vault::memory().unwrap();
        let parsed = crate::import::parse_conversation(&serde_json::json!({"id":"synthetic","title":"Garden plan","mapping":{"m":{"message":{"author":{"role":"user"},"content":{"parts":["Native plants"]}}}}}), &mut crate::model::ImportReport::default()).unwrap();
        let id = parsed.id.clone();
        v.import(&[parsed], crate::model::ImportReport::default())
            .unwrap();
        let input = RemovalInput {
            kind: RemovalKind::Conversation,
            conversation_id: Some(id),
        };
        let p = v.removal_plan(&input).unwrap().0;
        v.connection.execute_batch("CREATE TRIGGER fail_delete BEFORE DELETE ON conversations BEGIN SELECT RAISE(ABORT,'synthetic failure'); END;").unwrap();
        assert!(v.remove_reviewed(&input, &p.review_hash).is_err());
        assert_eq!(v.removal_plan(&input).unwrap().0.snapshots, 1);
        assert_eq!(v.list("Native plants", false, 0).unwrap().len(), 1);
        v.verify().unwrap();
    }
}
