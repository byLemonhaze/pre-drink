//! Owner-only review snapshots. MCP cannot save or approve anything.
use crate::{db::Vault, memory::Memory};
use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Serialize)]
pub struct ApprovalReview {
    pub kind: String,
    pub id: String,
    pub title: String,
    pub content: String,
    pub sources: Vec<Memory>,
    pub restricted: bool,
    pub review_hash: String,
}

impl Vault {
    pub fn approval_review(&self, kind: &str, id: &str) -> Result<ApprovalReview, String> {
        let (title, content, sources, restricted, snapshot) = match kind {
            "memory" => {
                let m = self
                    .memories(None, "", false)?
                    .into_iter()
                    .find(|m| m.id == id)
                    .ok_or("Memory not found.")?;
                let snapshot = serde_json::to_value(&m).map_err(|e| e.to_string())?;
                (
                    m.kind,
                    m.content,
                    vec![],
                    m.private || m.sensitive || m.source_restricted,
                    snapshot,
                )
            }
            "brief" => {
                let b = self
                    .briefs(false, "")?
                    .into_iter()
                    .find(|b| b.id == id)
                    .ok_or("Brief not found.")?;
                let count: i64 = self
                    .connection
                    .query_row(
                        "SELECT count(*) FROM brief_sources WHERE brief_id=?1",
                        [id],
                        |r| r.get(0),
                    )
                    .map_err(|e| e.to_string())?;
                let mut sources: Vec<_> = self
                    .memories(None, "", false)?
                    .into_iter()
                    .filter(|m| b.source_ids.contains(&m.id))
                    .collect();
                sources.sort_by(|a, b| a.id.cmp(&b.id));
                let restricted = b.private
                    || b.sensitive
                    || count != sources.len() as i64
                    || sources
                        .iter()
                        .any(|m| !m.approved || m.private || m.sensitive || m.source_restricted);
                // Include current source contents and restrictions, not just their IDs.
                let snapshot =
                    serde_json::json!({"brief": b, "sources": sources, "source_count": count});
                (b.title, b.content, sources, restricted, snapshot)
            }
            _ => return Err("Unknown review type.".into()),
        };
        let encoded = serde_json::to_vec(&snapshot).map_err(|e| e.to_string())?;
        Ok(ApprovalReview {
            kind: kind.into(),
            id: id.into(),
            title,
            content,
            sources,
            restricted,
            review_hash: hex::encode(Sha256::digest(encoded)),
        })
    }

    pub fn approve_review(
        &mut self,
        kind: &str,
        id: &str,
        review_hash: &str,
    ) -> Result<(), String> {
        // The desktop session mutex covers this comparison and the write. Any changed
        // note, brief, source, or restriction requires a fresh visible review.
        let current = self.approval_review(kind, id)?;
        if current.review_hash != review_hash {
            return Err(
                "This item or its sources changed. Open a fresh review before approving.".into(),
            );
        }
        if current.restricted {
            return Err(
                "Private, sensitive, missing, or unapproved sources cannot be shared.".into(),
            );
        }
        let tx = self.connection.transaction().map_err(|e| e.to_string())?;
        match kind {
            "memory" => {
                tx.execute("UPDATE memory SET approved=1 WHERE id=?1", [id])
                    .map_err(|e| e.to_string())?;
            }
            "brief" => {
                tx.execute("UPDATE brief_sources SET reviewed_at=(SELECT updated_at FROM memory WHERE id=memory_id) WHERE brief_id=?1", [id]).map_err(|e| e.to_string())?;
                tx.execute("UPDATE briefs SET approved=1 WHERE id=?1", [id])
                    .map_err(|e| e.to_string())?;
            }
            _ => unreachable!(),
        }
        tx.commit().map_err(|e| e.to_string())
    }
}
