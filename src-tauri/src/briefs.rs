use crate::{db::Vault, privacy::is_sensitive};
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize)]
pub struct Brief {
    pub id: String,
    pub title: String,
    pub content: String,
    pub approved: bool,
    pub private: bool,
    pub sensitive: bool,
    pub blocked: bool,
    pub source_ids: Vec<String>,
    pub updated_at: f64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BriefInput {
    pub id: Option<String>,
    pub title: String,
    pub content: String,
    pub private: bool,
    pub source_ids: Vec<String>,
}
impl Vault {
    pub fn save_brief(&mut self, input: BriefInput) -> Result<String, String> {
        if input.title.trim().is_empty()
            || input.title.len() > 200
            || input.content.trim().is_empty()
            || input.content.len() > 32_000
            || input.source_ids.len() > 100
        {
            return Err("Give the brief a title and up to 32,000 bytes of content, with at most 100 source memories.".into());
        }
        let id = input
            .id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let sensitive = is_sensitive(&input.title) || is_sensitive(&input.content);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        let tx = self.connection.transaction().map_err(|e| e.to_string())?;
        if input.id.is_some() {
            let changed=tx.execute("UPDATE briefs SET title=?1,content=?2,approved=?3,private=?4,sensitive=?5,updated_at=?6 WHERE id=?7",params![input.title,input.content,false,input.private,sensitive,now,id]).map_err(|e|e.to_string())?;
            if changed == 0 {
                return Err("Brief no longer exists.".into());
            }
        } else {
            tx.execute("INSERT INTO briefs(id,title,content,approved,private,sensitive,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",params![id,input.title,input.content,false,input.private,sensitive,now]).map_err(|e|e.to_string())?;
        }
        // Existing provenance cannot be silently removed. Orphaned sources keep a brief blocked.
        for source in input.source_ids {
            let updated: f64 = tx
                .query_row(
                    "SELECT updated_at FROM memory WHERE id=?1",
                    [&source],
                    |r| r.get(0),
                )
                .map_err(|_| "A selected source no longer exists.")?;
            tx.execute("INSERT INTO brief_sources(brief_id,memory_id,reviewed_at) VALUES (?1,?2,?3) ON CONFLICT(brief_id,memory_id) DO UPDATE SET reviewed_at=excluded.reviewed_at",params![id,source,updated]).map_err(|e|e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(id)
    }
    pub fn briefs(&self, for_agent: bool, topic: &str) -> Result<Vec<Brief>, String> {
        let mut briefs = Vec::new();
        self.visit_briefs(topic, |b| {
            if !for_agent || (b.approved && !b.private && !b.sensitive && !b.blocked) {
                briefs.push(b);
            }
        })?;
        for brief in &mut briefs {
            self.load_brief_sources(brief)?;
        }
        Ok(briefs)
    }
    pub(crate) fn visit_briefs(
        &self,
        topic: &str,
        mut visit: impl FnMut(Brief),
    ) -> Result<(), String> {
        if topic.len() > 1000 {
            return Err("Topic too long.".into());
        }
        let mut stmt=self.connection.prepare("SELECT b.id,b.title,b.content,b.approved,b.private,b.sensitive,b.updated_at,
            EXISTS(SELECT 1 FROM brief_sources s LEFT JOIN memory m ON m.id=s.memory_id LEFT JOIN messages msg ON msg.id=m.source_message_id LEFT JOIN conversations c ON c.id=msg.conversation_id
            WHERE s.brief_id=b.id AND (m.id IN (SELECT id FROM curation_blocked) OR m.id IS NULL OR m.approved=0 OR m.private=1 OR m.sensitive=1 OR m.updated_at!=s.reviewed_at OR (m.had_source=1 AND (msg.id IS NULL OR c.id IS NULL OR c.private=1 OR c.sensitive=1 OR msg.sensitive=1))))
            FROM briefs b WHERE (?1='' OR instr(lower(b.title||' '||b.content),lower(?1))>0) ORDER BY b.updated_at DESC,b.id").map_err(|e|e.to_string())?;
        let rows = stmt
            .query_map([topic], |r| {
                Ok(Brief {
                    id: r.get(0)?,
                    title: r.get(1)?,
                    content: r.get(2)?,
                    approved: r.get(3)?,
                    private: r.get(4)?,
                    sensitive: r.get(5)?,
                    updated_at: r.get(6)?,
                    blocked: r.get(7)?,
                    source_ids: vec![],
                })
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            visit(row.map_err(|e| e.to_string())?);
        }
        Ok(())
    }
    pub(crate) fn load_brief_sources(&self, brief: &mut Brief) -> Result<(), String> {
        let mut sources = self.connection.prepare("SELECT memory_id FROM brief_sources WHERE brief_id=?1 AND memory_id IS NOT NULL ORDER BY memory_id").map_err(|e| e.to_string())?;
        brief.source_ids = sources
            .query_map([&brief.id], |r| r.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    pub fn delete_brief(&self, id: &str) -> Result<(), String> {
        self.connection
            .execute("DELETE FROM briefs WHERE id=?1", [id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
