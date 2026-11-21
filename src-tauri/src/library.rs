//! Owner-only bounded library responses. Full-text matching scans all saved rows,
//! retaining at most two pages; it never collects the full library for the UI.
use crate::{briefs::Brief, db::Vault, memory::Memory};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const PAGE_SIZE: usize = 25;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LibraryQuery {
    pub collection: String,
    #[serde(default)]
    pub query: String,
    pub kind: Option<String>,
    #[serde(default)]
    pub include_restricted: bool,
    #[serde(default)]
    pub eligible_only: bool,
    #[serde(default)]
    pub exclude_ids: Vec<String>,
    #[serde(default)]
    pub offset: usize,
}
#[derive(Clone, Serialize)]
#[serde(tag = "collection", rename_all = "snake_case")]
pub enum LibraryItem {
    Memory(Memory),
    Brief(Brief),
}
#[derive(Serialize)]
pub struct LibraryPage {
    pub items: Vec<LibraryItem>,
    pub total: usize,
    pub offset: usize,
}

struct Collector {
    total: usize,
    requested: usize,
    page: Vec<LibraryItem>,
    tail: Vec<LibraryItem>,
}
impl Collector {
    fn add(&mut self, item: LibraryItem) {
        if self.total.is_multiple_of(PAGE_SIZE) {
            self.tail.clear();
        }
        if self.total / PAGE_SIZE == self.requested / PAGE_SIZE {
            self.page.push(item.clone());
        }
        self.tail.push(item);
        self.total += 1;
    }
    fn finish(self) -> LibraryPage {
        let offset = self.requested.min(self.total.saturating_sub(1)) / PAGE_SIZE * PAGE_SIZE;
        LibraryPage {
            items: if self.requested >= self.total {
                self.tail
            } else {
                self.page
            },
            total: self.total,
            offset,
        }
    }
}
impl Vault {
    pub fn library_page(&self, q: &LibraryQuery) -> Result<LibraryPage, String> {
        if !matches!(q.collection.as_str(), "memory" | "brief" | "all")
            || q.kind
                .as_deref()
                .is_some_and(|k| !matches!(k, "fact" | "decision" | "project"))
            || q.query.len() > 1000
            || q.exclude_ids.len() > 100
            || q.exclude_ids.iter().any(|id| id.len() > 200)
        {
            return Err(
                "Invalid library filter. Use a supported collection and a shorter search.".into(),
            );
        }
        let query = q.query.trim().to_lowercase();
        let excluded: HashSet<_> = q.exclude_ids.iter().map(String::as_str).collect();
        let mut results = Collector {
            total: 0,
            requested: q.offset,
            page: Vec::new(),
            tail: Vec::new(),
        };
        // Preserve the sharing picker's existing brief-first order.
        if q.collection != "memory" {
            self.visit_briefs("", |b| {
                let restricted = b.private || b.sensitive || b.blocked;
                if !excluded.contains(b.id.as_str())
                    && (q.include_restricted || !restricted)
                    && (!q.eligible_only || (b.approved && !restricted))
                    && format!("{} {}", b.title, b.content)
                        .to_lowercase()
                        .contains(&query)
                {
                    results.add(LibraryItem::Brief(b));
                }
            })?;
        }
        if q.collection != "brief" {
            self.visit_memories(q.kind.as_deref(), "", false, |m| {
                let restricted = m.private || m.sensitive || m.source_restricted;
                let text = if q.collection == "all" {
                    format!("{} {}", m.kind, m.content)
                } else {
                    m.content.clone()
                };
                if !excluded.contains(m.id.as_str())
                    && (q.include_restricted || !restricted)
                    && (!q.eligible_only || (m.approved && !restricted))
                    && text.to_lowercase().contains(&query)
                {
                    results.add(LibraryItem::Memory(m));
                }
            })?;
        }
        let mut page = results.finish();
        // Source IDs are needed only by displayed brief cards/editors, never for
        // counting or filtering the rest of the library.
        for item in &mut page.items {
            if let LibraryItem::Brief(brief) = item {
                self.load_brief_sources(brief)?;
            }
        }
        Ok(page)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;
    fn query(collection: &str) -> LibraryQuery {
        LibraryQuery {
            collection: collection.into(),
            query: String::new(),
            kind: None,
            include_restricted: false,
            eligible_only: false,
            exclude_ids: vec![],
            offset: 0,
        }
    }
    fn fixture() -> Vault {
        let v = Vault::memory().unwrap();
        let tx = v.connection.unchecked_transaction().unwrap();
        for n in 0..103 {
            tx.execute("INSERT INTO memory(id,type,content,approved,private,created_at,updated_at) VALUES (?1,'fact',?2,1,?3,1,?4)", params![format!("note-{n:03}"), format!("Fictional ÉTÉ garden {n:03}"), n==102, n]).unwrap();
        }
        for n in 0..30 {
            tx.execute("INSERT INTO briefs(id,title,content,approved,updated_at) VALUES (?1,?2,'Fictional garden plan',1,?3)", params![format!("brief-{n:03}"), format!("ÉTÉ brief {n:03}"), n]).unwrap();
        }
        tx.execute("INSERT INTO brief_sources(brief_id,memory_id,reviewed_at) VALUES ('brief-029','note-101',101)", []).unwrap();
        tx.commit().unwrap();
        v
    }
    fn ids(page: &LibraryPage) -> Vec<String> {
        page.items
            .iter()
            .map(|i| match i {
                LibraryItem::Memory(m) => m.id.clone(),
                LibraryItem::Brief(b) => b.id.clone(),
            })
            .collect()
    }
    #[test]
    fn full_search_privacy_filters_and_deleted_last_pages_are_consistent() {
        let v = fixture();
        let mut q = query("memory");
        let first = v.library_page(&q).unwrap();
        assert_eq!((first.total, first.items.len(), first.offset), (102, 25, 0));
        assert_eq!(ids(&first)[0], "note-101");
        q.offset = usize::MAX;
        let last = v.library_page(&q).unwrap();
        assert_eq!((last.offset, last.items.len()), (100, 2));
        assert_eq!(ids(&last), ["note-001", "note-000"]);
        q.query = "été garden 000".into();
        let found = v.library_page(&q).unwrap();
        assert_eq!((found.total, found.offset), (1, 0));
        assert_eq!(ids(&found), ["note-000"]);
        q.query = "102".into();
        assert_eq!(v.library_page(&q).unwrap().total, 0);
        q.include_restricted = true;
        assert_eq!(v.library_page(&q).unwrap().total, 1);
        q.eligible_only = true; // eligibility can never be overridden by restricted view
        assert_eq!(v.library_page(&q).unwrap().total, 0);
        q = query("memory");
        q.offset = 100;
        v.connection
            .execute("DELETE FROM memory WHERE id IN ('note-000','note-001')", [])
            .unwrap();
        let clamped = v.library_page(&q).unwrap();
        assert_eq!(
            (clamped.offset, clamped.items.len(), clamped.total),
            (75, 25, 100)
        );
    }
    #[test]
    fn mixed_selection_keeps_order_and_source_picker_does_not_truncate_at_early_matches() {
        let v = fixture();
        let mut q = query("all");
        q.include_restricted = true;
        let first = v.library_page(&q).unwrap();
        assert_eq!((first.total, first.items.len()), (133, 25));
        match &first.items[0] {
            LibraryItem::Brief(b) => assert_eq!(b.source_ids, ["note-101"]),
            _ => panic!("briefs must come first"),
        }
        q.offset = 25;
        let second = ids(&v.library_page(&q).unwrap());
        assert_eq!(second[4], "brief-000");
        assert_eq!(second[5], "note-102");
        q = query("memory");
        q.eligible_only = true;
        q.exclude_ids = (2..102).map(|n| format!("note-{n:03}")).collect();
        assert_eq!(ids(&v.library_page(&q).unwrap()), ["note-001", "note-000"]);
        q.exclude_ids.push("note-000".into());
        assert!(v.library_page(&q).is_err());
        q = query("brief");
        q.query = "été brief 000".into();
        assert_eq!(ids(&v.library_page(&q).unwrap()), ["brief-000"]);
        v.connection
            .execute("UPDATE memory SET private=1 WHERE id='note-101'", [])
            .unwrap();
        q.query.clear();
        assert_eq!(v.library_page(&q).unwrap().total, 29);
    }
    #[test]
    fn large_library_response_retains_only_a_page_of_complete_items() {
        let v = Vault::memory().unwrap();
        let text = "Fictional garden. ".repeat(500);
        let tx = v.connection.unchecked_transaction().unwrap();
        for n in 0..2000 {
            tx.execute("INSERT INTO memory(id,type,content,created_at,updated_at) VALUES (?1,'project',?2,1,?3)", params![format!("fictional-{n:04}"), text, n]).unwrap();
        }
        tx.commit().unwrap();
        let page = v.library_page(&query("memory")).unwrap();
        assert_eq!((page.total, page.items.len()), (2000, 25));
        for item in &page.items {
            match item {
                LibraryItem::Memory(m) => assert_eq!(m.content, text),
                _ => unreachable!(),
            }
        }
        let paged_bytes = serde_json::to_vec(&page).unwrap().len();
        let all_bytes = serde_json::to_vec(&v.memories(None, "", false).unwrap())
            .unwrap()
            .len();
        assert!(paged_bytes * 50 < all_bytes);
        println!("fictional_library_items=2000 full_bytes={all_bytes} page_bytes={paged_bytes}");
    }
}
