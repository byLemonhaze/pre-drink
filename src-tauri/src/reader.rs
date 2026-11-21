//! Owner reader pages. Search scans full text without materializing the whole chat.
use crate::{db::Vault, model::Message};
use rusqlite::params;
use serde::Serialize;

pub const MESSAGE_PAGE_SIZE: usize = 20;
#[derive(Serialize)]
pub struct MessagePage {
    pub messages: Vec<Message>,
    pub total: usize,
    pub offset: usize,
    pub has_alternates: bool,
}
impl Vault {
    pub fn message_page(
        &self,
        id: &str,
        include_restricted: bool,
        all_branches: bool,
        query: &str,
        offset: u32,
    ) -> Result<MessagePage, String> {
        if query.len() > 800 {
            return Err("Search text is too long.".into());
        }
        let allowed: bool = self.connection.query_row("SELECT EXISTS(SELECT 1 FROM conversations WHERE id=?1 AND (?2 OR (private=0 AND sensitive=0)))", params![id,include_restricted], |r| r.get(0)).map_err(|e|e.to_string())?;
        if !allowed {
            return Err(
                "This conversation is private, flagged as sensitive, or no longer available."
                    .into(),
            );
        }
        let term = query.trim().to_lowercase();
        let offset = offset as usize / MESSAGE_PAGE_SIZE * MESSAGE_PAGE_SIZE;
        let mut statement = self.connection.prepare("SELECT id,conversation_id,role,content,timestamp,branch_id,parent_id,is_active,sensitive,position FROM messages WHERE conversation_id=?1 ORDER BY position,id").map_err(|e|e.to_string())?;
        let rows = statement
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
        let mut page = MessagePage {
            messages: Vec::new(),
            total: 0,
            offset,
            has_alternates: false,
        };
        for row in rows {
            let message = row.map_err(|e| e.to_string())?;
            page.has_alternates |= !message.is_active;
            if (all_branches || message.is_active)
                && (term.is_empty() || message.content.to_lowercase().contains(&term))
            {
                if page.total >= offset && page.messages.len() < MESSAGE_PAGE_SIZE {
                    page.messages.push(message);
                }
                page.total += 1;
            }
        }
        // A deletion can shrink a list while its last page is open. Only this
        // exceptional clamp needs another scan; ordinary requests retain 20 rows.
        if page.total > 0 && offset >= page.total {
            return self.message_page(
                id,
                include_restricted,
                all_branches,
                query,
                ((page.total - 1) / MESSAGE_PAGE_SIZE * MESSAGE_PAGE_SIZE) as u32,
            );
        }
        if page.total == 0 {
            page.offset = 0;
        }
        Ok(page)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{import::parse_conversation, model::ImportReport};
    use serde_json::json;
    #[test]
    fn pages_search_complete_unicode_sources_and_preserve_branch_and_privacy_boundaries() {
        let mut v = Vault::memory().unwrap();
        let mut mapping = serde_json::Map::new();
        for i in 0..65 {
            mapping.insert(format!("{i:03}"), json!({"parent": if i==0 {None} else {Some(format!("{:03}",i-1))},"message":{"author":{"role":"user"},"content":{"parts":[if i==64 {format!("{} ÉRABLE 🌿 finalmarker", "garden ".repeat(12000))} else {format!("Garden {i}")}]}}}));
        }
        mapping.insert("alternate".into(),json!({"parent":"000","message":{"author":{"role":"assistant"},"content":{"parts":["alternateonly"]}}}));
        let c = parse_conversation(&json!({"id":"paged","title":"Fictional garden","current_node":"064","mapping":mapping}),&mut ImportReport::default()).unwrap();
        let id = c.id.clone();
        v.import(&[c], ImportReport::default()).unwrap();
        let first = v.message_page(&id, false, false, "", 0).unwrap();
        assert_eq!(
            (first.messages.len(), first.total, first.has_alternates),
            (20, 65, true)
        );
        let middle = v.message_page(&id, false, false, "", 20).unwrap();
        assert!(first
            .messages
            .iter()
            .all(|a| middle.messages.iter().all(|b| a.id != b.id)));
        let last = v.message_page(&id, false, false, "", u32::MAX).unwrap();
        assert_eq!((last.offset, last.messages.len()), (60, 5));
        let found = v.message_page(&id, false, false, "érable 🌿", 0).unwrap();
        assert_eq!(found.total, 1);
        assert!(found.messages[0].content.ends_with("finalmarker"));
        assert!(found.messages[0].content.len() > 48000);
        assert_eq!(
            v.message_page(&id, false, false, "alternateonly", 0)
                .unwrap()
                .total,
            0
        );
        assert_eq!(
            v.message_page(&id, false, true, "alternateonly", 0)
                .unwrap()
                .total,
            1
        );
        assert_eq!(
            v.message_page(&id, false, true, "no such phrase", 80)
                .unwrap()
                .offset,
            0
        );
        v.mark_private(&id, true).unwrap();
        assert!(v.message_page(&id, false, true, "", 0).is_err());
        assert_eq!(v.message_page(&id, true, true, "", 0).unwrap().total, 66);
        assert!(v.message_page("missing", true, true, "", 0).is_err());
    }
}
