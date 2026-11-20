//! Fault tests use fresh encrypted files only. No live vault or disk filling.
use crate::{
    briefs::BriefInput, db::Vault, import::parse_conversation, memory::MemoryInput,
    model::ImportReport,
};
use rusqlite::types::Value;
use serde_json::json;
use std::{
    path::Path,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

const PASSWORD: &str = "fictional storage recovery phrase";

fn conversation(id: &str, text: &str) -> crate::model::ParsedConversation {
    parse_conversation(
        &json!({"id":id,"title":"Fictional garden","current_node":"leaf","mapping":{
            "leaf":{"parent":null,"message":{"author":{"role":"user"},"content":{"parts":[text]}}}
        }}),
        &mut ImportReport::default(),
    )
    .unwrap()
}

fn snapshot(v: &Vault) -> Vec<Vec<Vec<Value>>> {
    [
        "conversations",
        "messages",
        "archive_snapshots",
        "memory",
        "briefs",
        "brief_sources",
        "search_index",
    ]
    .into_iter()
    .map(|table| {
        let mut stmt = v
            .connection
            .prepare(&format!("SELECT * FROM {table} ORDER BY rowid"))
            .unwrap();
        let columns = stmt.column_count();
        stmt.query_map([], |row| {
            (0..columns)
                .map(|i| row.get(i))
                .collect::<Result<Vec<Value>, _>>()
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
    })
    .collect()
}

fn baseline(path: &Path) -> (Vault, String, String) {
    let mut v = Vault::create(path, PASSWORD).unwrap();
    let c = conversation("garden", "originalorchid: choose cedar planters");
    let source = c.messages[0].id.clone();
    v.import(&[c], ImportReport::default()).unwrap();
    let note = v
        .save_memory(MemoryInput {
            id: None,
            kind: "fact".into(),
            content: "Choose cedar planters.".into(),
            source_message_id: Some(source),
            private: false,
        })
        .unwrap();
    let review = v.approval_review("memory", &note).unwrap();
    v.approve_review("memory", &note, &review.review_hash)
        .unwrap();
    let brief = v
        .save_brief(BriefInput {
            id: None,
            title: "Garden handoff".into(),
            content: "Choose cedar planters.".into(),
            source_ids: vec![note.clone()],
            private: false,
        })
        .unwrap();
    let review = v.approval_review("brief", &brief).unwrap();
    v.approve_review("brief", &brief, &review.review_hash)
        .unwrap();
    (v, note, brief)
}

#[test]
fn database_full_preserves_archive_approvals_and_search_then_allows_retry() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("fictional.sqlite3");
    let (mut v, note, brief) = baseline(&path);
    v.connection.execute_batch("VACUUM;").unwrap();
    let before = snapshot(&v);
    let pages: i64 = v
        .connection
        .query_row("PRAGMA page_count", [], |r| r.get(0))
        .unwrap();
    v.connection
        .pragma_update(None, "max_page_count", pages)
        .unwrap();

    // Combine an existing-source update with a large new conversation. Whether
    // capacity is exhausted early or late, the whole import must roll back.
    let batch = [
        conversation("garden", "replacedorchid: changed source"),
        conversation("large", &"native plants ".repeat(30000)),
    ];
    let error = v.import(&batch, ImportReport::default()).unwrap_err();
    assert!(
        error.contains("full"),
        "Expected capacity failure, got {error}"
    );
    assert_eq!(snapshot(&v), before);

    let error = v
        .save_memory(MemoryInput {
            id: Some(note.clone()),
            kind: "fact".into(),
            content: "cedar ".repeat(2600),
            source_message_id: None,
            private: false,
        })
        .unwrap_err();
    assert!(
        error.contains("full"),
        "Expected capacity failure, got {error}"
    );
    assert_eq!(snapshot(&v), before);
    let error = v
        .save_brief(BriefInput {
            id: Some(brief),
            title: "Changed handoff".into(),
            content: "cedar ".repeat(5200),
            source_ids: vec![note],
            private: false,
        })
        .unwrap_err();
    assert!(
        error.contains("full"),
        "Expected capacity failure, got {error}"
    );
    assert_eq!(snapshot(&v), before);

    v.connection
        .pragma_update(None, "max_page_count", 2_147_483_646_i64)
        .unwrap();
    v.verify().unwrap();
    drop(v);
    let mut v = Vault::open(&path, PASSWORD).unwrap();
    assert_eq!(snapshot(&v), before);
    assert_eq!(v.list("originalorchid", false, 0).unwrap().len(), 1);
    v.import(&batch, ImportReport::default()).unwrap();
    assert_eq!(v.stats().unwrap().conversations, 2);
    assert!(v.list("originalorchid", false, 0).unwrap().is_empty());
    assert_eq!(v.list("replacedorchid", false, 0).unwrap().len(), 1);
    assert!(v.memories(None, "", true).unwrap().is_empty());
    assert!(v.briefs(true, "").unwrap().is_empty());
    v.verify().unwrap();
}

struct KillOnDrop(Child);
impl Drop for KillOnDrop {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn killed_writer_recovers_committed_encrypted_state() {
    const CHILD_PATH: &str = "PRE_DRINK_FICTIONAL_CRASH_CHILD";
    if let Some(path) = std::env::var_os(CHILD_PATH) {
        let path = std::path::PathBuf::from(path);
        let v = Vault::open(&path, PASSWORD).unwrap();
        v.connection.execute_batch("BEGIN IMMEDIATE; UPDATE memory SET content='uncommittedorchid',approved=0; UPDATE conversations SET private=1; DELETE FROM search_index;").unwrap();
        // Force dirty cache pages toward the journal/WAL before killing the
        // process; an open transaction must never become a committed revision.
        v.connection.cache_flush().unwrap();
        std::fs::write(path.with_extension("ready"), b"fictional writer ready").unwrap();
        loop {
            std::thread::park();
        }
    }
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("fictional.sqlite3");
    let (v, _, _) = baseline(&path);
    let before = snapshot(&v);
    v.checkpoint().unwrap();
    drop(v);
    let mut child = KillOnDrop(
        Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "storage_failure::killed_writer_recovers_committed_encrypted_state",
                "--nocapture",
            ])
            .env(CHILD_PATH, &path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(30);
    while !path.with_extension("ready").exists() {
        assert!(
            child.0.try_wait().unwrap().is_none(),
            "Fictional writer exited before fault boundary"
        );
        assert!(
            Instant::now() < deadline,
            "Fictional writer did not reach fault boundary"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    child.0.kill().unwrap();
    child.0.wait().unwrap();
    let mut v = Vault::open(&path, PASSWORD).unwrap();
    v.verify().unwrap();
    assert_eq!(snapshot(&v), before);
    assert_eq!(v.list("originalorchid", false, 0).unwrap().len(), 1);
    assert_eq!(v.memories(None, "", true).unwrap().len(), 1);
    assert_eq!(v.briefs(true, "").unwrap().len(), 1);
    v.import(
        &[conversation("retry", "recoveredorchid")],
        ImportReport::default(),
    )
    .unwrap();
    assert_eq!(v.list("recoveredorchid", false, 0).unwrap().len(), 1);
}
