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
        "source_versions",
        "curation_notes",
        "curation_sources",
        "curation_revisions",
        "curation_publications",
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
    crate::test_support::wait_for_checkpoint(&mut child.0, &path.with_extension("ready"));
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

// Faults are one-shot and local to the current test thread. This module and
// every call site are absent from production builds; no runtime fault settings.
thread_local! {
    static FAULT: std::cell::Cell<Option<&'static str>> = const { std::cell::Cell::new(None) };
}
struct Fault;
impl Fault {
    fn arm(point: &'static str) -> Self {
        FAULT.with(|f| assert!(f.replace(Some(point)).is_none()));
        Self
    }
    fn consumed(&self) {
        FAULT.with(|f| assert!(f.get().is_none(), "Fault boundary was not reached"));
    }
}
impl Drop for Fault {
    fn drop(&mut self) {
        FAULT.with(|f| f.set(None));
    }
}
pub(crate) fn take_fault(point: &str) -> bool {
    FAULT.with(|f| {
        if f.get() == Some(point) {
            f.set(None);
            true
        } else {
            false
        }
    })
}
pub(crate) fn io_fault(point: &str) -> std::io::Result<()> {
    if take_fault(point) {
        Err(std::io::Error::from_raw_os_error(libc::ENOSPC))
    } else {
        Ok(())
    }
}

fn session_baseline(root: &Path) -> crate::session::Session {
    let (v, _, _) = baseline(&root.join("vault.enc.sqlite3"));
    let mut s = crate::session::Session::new(root.into());
    s.vault = Some(v);
    s
}
fn assert_no_partial_files(root: &Path) {
    for entry in std::fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        assert!(
            !entry.file_name().to_string_lossy().contains(".partial"),
            "An ordinary failure left a partial file"
        );
    }
}

#[test]
fn backup_capacity_flush_and_publication_failures_preserve_source_and_allow_retry() {
    for point in [
        "backup_export_full",
        "backup_file_sync",
        "backup_publish",
        "backup_directory_sync",
    ] {
        let root = tempfile::tempdir().unwrap();
        let backups = tempfile::tempdir().unwrap();
        let mut s = session_baseline(root.path());
        let before = snapshot(s.access().unwrap());
        // A prior good receipt must not authorize cleanup after a failed backup.
        s.backup_with_receipt(&backups.path().join("prior.sqlite3"), PASSWORD)
            .unwrap();
        assert!(s.backup_receipt.is_some());
        let target = backups.path().join("attempt.sqlite3");
        let fault = Fault::arm(point);
        let error = s.backup_with_receipt(&target, PASSWORD).unwrap_err();
        fault.consumed();
        assert!(s.backup_receipt.is_none());
        assert_eq!(snapshot(s.access().unwrap()), before);
        assert_no_partial_files(backups.path());
        if point == "backup_directory_sync" {
            assert!(error.contains("file was created"));
            let saved = Vault::open(&target, PASSWORD).unwrap();
            saved.verify().unwrap();
            assert_eq!(snapshot(&saved), before);
            // Post-publication uncertainty preserves the file and refuses overwrite.
            assert!(s.backup_with_receipt(&target, PASSWORD).is_err());
        } else {
            assert!(
                !target.exists(),
                "Pre-publication failure published a backup"
            );
        }
        // Retry on the SAME connection: a failed SQLCipher export must detach.
        s.backup_with_receipt(&backups.path().join("retry.sqlite3"), PASSWORD)
            .unwrap();
        assert!(s.backup_receipt.is_some());
        s.lock();
        s.unlock(PASSWORD, false, false).unwrap();
        assert_eq!(snapshot(s.access().unwrap()), before);
        s.access().unwrap().verify().unwrap();
    }
}

#[test]
fn partial_restore_copy_is_removed_and_original_and_backup_survive() {
    let root = tempfile::tempdir().unwrap();
    let backups = tempfile::tempdir().unwrap();
    let mut s = session_baseline(root.path());
    let original = s.path().unwrap();
    let before = snapshot(s.access().unwrap());
    let backup = backups.path().join("verified.sqlite3");
    s.backup_with_receipt(&backup, PASSWORD).unwrap();
    let backup_bytes = std::fs::read(&backup).unwrap();
    let fault = Fault::arm("restore_partial_copy");
    assert!(s.prepare_restore(&backup, PASSWORD).is_err());
    fault.consumed();
    assert!(s.pending_replacement.is_none());
    assert!(!s.sharing);
    assert_eq!(s.path().unwrap(), original);
    assert_eq!(snapshot(s.access().unwrap()), before);
    assert!(std::fs::read_dir(root.path())
        .unwrap()
        .all(|e| !e.unwrap().path().is_dir()));
    assert_eq!(std::fs::read(&backup).unwrap(), backup_bytes);
    let preview = s.prepare_restore(&backup, PASSWORD).unwrap();
    s.apply_replacement(&preview.review_id, PASSWORD).unwrap();
    s.unlock(PASSWORD, false, false).unwrap();
    assert_eq!(snapshot(s.access().unwrap()), before);
}

#[test]
fn selector_write_and_publish_failures_preserve_original_and_retry() {
    const NEXT: &str = "fictional next recovery phrase";
    for point in ["switch_write", "switch_publish"] {
        for already_rotated in [false, true] {
            let root = tempfile::tempdir().unwrap();
            let mut s = session_baseline(root.path());
            let (current, next) = if already_rotated {
                let p = s.prepare_password_change(PASSWORD, NEXT).unwrap();
                s.apply_replacement(&p.review_id, NEXT).unwrap();
                s.unlock(NEXT, false, false).unwrap();
                (NEXT, PASSWORD)
            } else {
                (PASSWORD, NEXT)
            };
            let original = s.path().unwrap();
            let before = snapshot(s.access().unwrap());
            let p = s.prepare_password_change(current, next).unwrap();
            let fault = Fault::arm(point);
            assert!(s.apply_replacement(&p.review_id, next).is_err());
            fault.consumed();
            assert_eq!(s.path().unwrap(), original);
            assert!(s.pending_replacement.is_none());
            assert_no_partial_files(root.path());
            s.lock();
            drop(s);
            let mut s = crate::session::Session::new(root.path().into());
            s.unlock(current, false, false).unwrap();
            assert_eq!(snapshot(s.access().unwrap()), before);
            let p = s.prepare_password_change(current, next).unwrap();
            s.apply_replacement(&p.review_id, next).unwrap();
            s.unlock(next, false, false).unwrap();
            assert_eq!(snapshot(s.access().unwrap()), before);
        }
    }
}

#[test]
fn selector_flush_uncertainty_keeps_both_vaults_and_uses_committed_selection() {
    const NEXT: &str = "fictional next recovery phrase";
    let root = tempfile::tempdir().unwrap();
    let mut s = session_baseline(root.path());
    let original = s.path().unwrap();
    let before = snapshot(s.access().unwrap());
    let p = s.prepare_password_change(PASSWORD, NEXT).unwrap();
    let fault = Fault::arm("switch_directory_sync");
    let result = s.apply_replacement(&p.review_id, NEXT).unwrap();
    fault.consumed();
    assert!(result.durability_warning);
    assert!(s.vault.is_none());
    assert!(!s.sharing);
    assert_ne!(s.path().unwrap(), original);
    drop(s);
    let mut s = crate::session::Session::new(root.path().into());
    s.unlock(NEXT, false, false).unwrap();
    assert_eq!(snapshot(s.access().unwrap()), before);
    let old = Vault::open(&original, PASSWORD).unwrap();
    old.verify().unwrap();
    assert_eq!(snapshot(&old), before);
}

#[test]
fn parsed_import_cannot_cross_lock_reunlock_expiry_or_replacement() {
    for action in ["lock", "reunlock", "expire", "replace"] {
        let root = tempfile::tempdir().unwrap();
        let mut s = session_baseline(root.path());
        let before = snapshot(s.access().unwrap());
        let generation = s.generation;
        let parsed = [conversation("new", "neworchid plan")];
        let password = if action == "replace" {
            let next = "fictional replacement recovery phrase";
            let p = s.prepare_password_change(PASSWORD, next).unwrap();
            s.apply_replacement(&p.review_id, next).unwrap();
            s.unlock(next, false, false).unwrap();
            next
        } else {
            match action {
                "lock" => s.lock(),
                "reunlock" => {
                    s.lock();
                    s.unlock(PASSWORD, false, false).unwrap();
                }
                "expire" => {
                    s.last_activity = std::time::SystemTime::now() - crate::session::IDLE_TIMEOUT
                }
                _ => unreachable!(),
            }
            PASSWORD
        };
        assert!(s
            .finish_import(generation, &parsed, ImportReport::default())
            .is_err());
        if s.vault.is_none() {
            s.unlock(password, false, false).unwrap();
        }
        assert_eq!(snapshot(s.access().unwrap()), before);
        s.finish_import(s.generation, &parsed, ImportReport::default())
            .unwrap();
        assert_eq!(
            s.access()
                .unwrap()
                .list("neworchid", false, 0)
                .unwrap()
                .len(),
            1
        );
    }
}

pub(crate) fn import_checkpoint(path: &str, tx: &rusqlite::Transaction<'_>) {
    if std::env::var("PD_IMPORT_CRASH_PATH").as_deref() == Ok(path) {
        tx.cache_flush().unwrap();
        std::fs::write(
            Path::new(path).with_extension("ready"),
            b"fictional import ready",
        )
        .unwrap();
        loop {
            std::thread::park();
        }
    }
}

#[test]
fn killed_import_rolls_back_sources_approvals_and_index_then_retries() {
    let batch = || {
        [
            conversation("garden", "replacedorchid source"),
            conversation("new", "neworchid source"),
        ]
    };
    if let Some(path) = std::env::var_os("PD_IMPORT_CRASH_PATH") {
        let mut v = Vault::open(Path::new(&path), PASSWORD).unwrap();
        v.import(&batch(), ImportReport::default()).unwrap();
        panic!("Missed import commit boundary");
    }
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("fictional.sqlite3");
    let (v, _, _) = baseline(&path);
    let before = snapshot(&v);
    v.checkpoint().unwrap();
    drop(v);
    let mut child = KillOnDrop(Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "storage_failure::killed_import_rolls_back_sources_approvals_and_index_then_retries", "--nocapture"])
        .env("PD_IMPORT_CRASH_PATH", &path)
        .stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap());
    crate::test_support::wait_for_checkpoint(&mut child.0, &path.with_extension("ready"));
    child.0.kill().unwrap();
    child.0.wait().unwrap();
    let mut v = Vault::open(&path, PASSWORD).unwrap();
    v.verify().unwrap();
    assert_eq!(snapshot(&v), before);
    assert_eq!(v.list("originalorchid", false, 0).unwrap().len(), 1);
    assert!(v.list("neworchid", false, 0).unwrap().is_empty());
    v.import(&batch(), ImportReport::default()).unwrap();
    assert_eq!(v.list("neworchid", false, 0).unwrap().len(), 1);
    assert!(v.memories(None, "", true).unwrap().is_empty());
    assert!(v.briefs(true, "").unwrap().is_empty());
}
