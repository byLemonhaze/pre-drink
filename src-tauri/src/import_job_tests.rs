use super::*;
use serde_json::{json, Value};
use std::{cell::Cell, io::Write, path::Path, sync::Mutex};
const PASSWORD: &str = "fictional resumable garden phrase";
thread_local! {
    static ACTION: Cell<Option<&'static str>> = const { Cell::new(None) };
    pub(super) static PARSED: Cell<usize> = const { Cell::new(0) };
}
pub(super) fn after_checkpoint(store: &Store) {
    if let Some(action) = ACTION.with(|a| a.take()) {
        if action == "concurrent" {
            assert!(run_import(store.clone(), None)
                .unwrap_err()
                .contains("already running"));
        }
        let mut s = store.lock().unwrap();
        match action {
            "pause" | "concurrent" => s.pause_import().unwrap(),
            "lock" => {
                s.lock();
                s.unlock(PASSWORD, false, false).unwrap();
            }
            "expiry" => {
                s.last_activity = std::time::SystemTime::now() - crate::session::IDLE_TIMEOUT;
                s.expire();
            }
            _ => panic!("Unknown fixture action"),
        }
    }
}
fn tree(id: usize) -> Value {
    json!({"id":format!("fictional-{id}"),"title":format!("Fictional garden {id}"),"current_node":"leaf","mapping":{
        "leaf":{"parent":null,"message":{"author":{"role":"user"},"content":{"parts":[format!("checkpointorchid {id}")]} }},"hidden":{"parent":null,"message":{"author":{"role":"tool"},"content":{"parts":["fictional tool metadata"]}}}},"fixture_marker":"only synthetic"})
}
fn zip(path: &Path, data: &Value) {
    let mut w = zip::ZipWriter::new(File::create(path).unwrap());
    w.start_file(
        "conversations.json",
        zip::write::SimpleFileOptions::default(),
    )
    .unwrap();
    w.write_all(data.to_string().as_bytes()).unwrap();
    w.finish().unwrap();
}
fn store(path: &Path, create: bool) -> Store {
    let mut s = Session::new(path.to_path_buf());
    s.unlock(PASSWORD, create, false).unwrap();
    Arc::new(Mutex::new(s))
}
fn prepared(v: &mut Vault, text: &str) {
    let mut data = tree(0);
    data["mapping"]["leaf"]["message"]["content"]["parts"][0] = json!(text);
    let item = parse_export_conversation(&data, false, &mut ImportReport::default()).unwrap();
    v.begin_import("fixture", "fictional.zip", 1).unwrap();
    v.save_import_batch(0, &[item], 0).unwrap();
}
fn run_token() -> ImportRun {
    ImportRun {
        cancelled: Arc::new(AtomicBool::new(false)),
        phase: Arc::new(AtomicU8::new(2)),
    }
}

#[test]
fn paused_checkpoints_survive_restart_backup_and_resume_without_reparsing_prefix() {
    let root = tempfile::tempdir().unwrap();
    let live = root.path().join("live");
    std::fs::create_dir(&live).unwrap();
    let input = root.path().join("fictional.zip");
    zip(&input, &json!((0..45).map(tree).collect::<Vec<_>>()));
    let s = store(&live, true);
    ACTION.with(|a| a.set(Some("pause")));
    assert!(run_import(s.clone(), Some(input.clone()))
        .unwrap_err()
        .contains("paused"));
    let backup = root.path().join("backup.sqlite3");
    {
        let mut guard = s.lock().unwrap();
        let v = guard.access().unwrap();
        assert_eq!(v.import_checkpoint().unwrap().unwrap().saved, 20);
        assert_eq!(v.stats().unwrap().conversations, 0);
        assert!(v.list("checkpointorchid", true, 0).unwrap().is_empty());
        assert!(v.memories(None, "", true).unwrap().is_empty());
        v.encrypted_backup(&backup, PASSWORD).unwrap();
        v.checkpoint().unwrap();
        guard.lock();
    }
    drop(s);
    let backup_vault = Vault::open(&backup, PASSWORD).unwrap();
    assert_eq!(backup_vault.import_checkpoint().unwrap().unwrap().saved, 20);
    assert_eq!(backup_vault.stats().unwrap().conversations, 0);
    for dir in [&live, root.path()] {
        for entry in std::fs::read_dir(dir).unwrap().flatten() {
            if entry.path().extension().is_some_and(|x| x == "sqlite3") {
                assert!(!std::fs::read(entry.path())
                    .unwrap()
                    .windows(16)
                    .any(|w| w == b"checkpointorchid"));
            }
        }
    }
    let s = store(&live, false);
    let wrong = root.path().join("different.zip");
    zip(&wrong, &json!([tree(99)]));
    assert!(run_import(s.clone(), Some(wrong))
        .unwrap_err()
        .contains("does not match"));
    assert_eq!(
        s.lock()
            .unwrap()
            .import_state()
            .unwrap()
            .checkpoint
            .unwrap()
            .saved,
        20
    );
    let renamed = root.path().join("renamed.zip");
    std::fs::rename(&input, &renamed).unwrap();
    PARSED.with(|p| p.set(0));
    let report = run_import(s.clone(), Some(renamed.clone())).unwrap();
    assert_eq!(PARSED.with(|p| p.get()), 25);
    assert_eq!((report.new_conversations, report.messages), (45, 45));
    assert_eq!(report.skipped_messages, 45);
    assert_eq!(report.warnings.len(), 1);
    assert!(s
        .lock()
        .unwrap()
        .import_state()
        .unwrap()
        .checkpoint
        .is_none());
    let repeat = run_import(s.clone(), Some(renamed)).unwrap();
    assert_eq!(
        (repeat.new_conversations, repeat.updated_conversations),
        (0, 45)
    );
    s.lock().unwrap().access().unwrap().verify().unwrap();
}

#[test]
fn malformed_or_duplicate_late_records_never_publish_partial_history() {
    for duplicate in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let s = store(root.path(), true);
        let mut data = (0..25).map(tree).collect::<Vec<_>>();
        data[24] = if duplicate {
            tree(0)
        } else {
            json!({"id":"malformed"})
        };
        let path = root.path().join("bad.zip");
        zip(&path, &json!(data));
        assert!(run_import(s.clone(), Some(path)).is_err());
        let mut guard = s.lock().unwrap();
        assert_eq!(guard.import_state().unwrap().checkpoint.unwrap().saved, 20);
        assert_eq!(guard.access().unwrap().stats().unwrap().conversations, 0);
        assert!(guard
            .access()
            .unwrap()
            .publish_import(&run_token())
            .is_err());
        guard.discard_import().unwrap();
        assert!(guard.import_state().unwrap().checkpoint.is_none());
        assert_eq!(
            guard
                .access()
                .unwrap()
                .connection
                .query_row("SELECT count(*) FROM import_items", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
}

#[test]
fn lock_reunlock_expiry_and_concurrent_start_cannot_cross_import_runs() {
    for action in ["lock", "expiry", "concurrent"] {
        let root = tempfile::tempdir().unwrap();
        let s = store(root.path(), true);
        let path = root.path().join("fictional.zip");
        zip(&path, &json!((0..25).map(tree).collect::<Vec<_>>()));
        ACTION.with(|a| a.set(Some(action)));
        assert!(run_import(s.clone(), Some(path.clone())).is_err());
        let mut guard = s.lock().unwrap();
        if guard.vault.is_none() {
            guard.unlock(PASSWORD, false, false).unwrap();
        }
        assert_eq!(guard.import_state().unwrap().checkpoint.unwrap().saved, 20);
        assert_eq!(guard.access().unwrap().stats().unwrap().conversations, 0);
        drop(guard);
        assert_eq!(run_import(s, Some(path)).unwrap().new_conversations, 25);
    }
}

#[test]
fn final_save_capacity_failure_preserves_checkpoint_and_retries_atomically() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("fictional.sqlite3");
    let mut v = Vault::create(&path, PASSWORD).unwrap();
    prepared(&mut v, &"fictionalgarden ".repeat(30000));
    v.connection.execute_batch("VACUUM;").unwrap();
    let pages: i64 = v
        .connection
        .query_row("PRAGMA page_count", [], |r| r.get(0))
        .unwrap();
    v.connection
        .pragma_update(None, "max_page_count", pages)
        .unwrap();
    assert!(v.publish_import(&run_token()).unwrap_err().contains("full"));
    assert_eq!(v.stats().unwrap().conversations, 0);
    assert_eq!(v.import_checkpoint().unwrap().unwrap().saved, 1);
    v.connection
        .pragma_update(None, "max_page_count", 2147483646_i64)
        .unwrap();
    assert_eq!(v.publish_import(&run_token()).unwrap().new_conversations, 1);
    assert!(v.import_checkpoint().unwrap().is_none());
    v.verify().unwrap();
}

#[test]
fn ready_checkpoint_finishes_without_zip_and_rechecks_current_privacy() {
    let root = tempfile::tempdir().unwrap();
    let s = store(root.path(), true);
    {
        let mut guard = s.lock().unwrap();
        let v = guard.access().unwrap();
        prepared(v, "my wifi password is fictional-lilac");
        // Model a checkpoint from an earlier classifier.
        let text: String = v
            .connection
            .query_row("SELECT payload FROM import_items", [], |r| r.get(0))
            .unwrap();
        let mut c: ParsedConversation = serde_json::from_str(&text).unwrap();
        c.sensitive = false;
        c.messages[0].sensitive = false;
        v.connection
            .execute(
                "UPDATE import_items SET payload=?1",
                [serde_json::to_string(&c).unwrap()],
            )
            .unwrap();
    }
    let phases = std::cell::RefCell::new(Vec::new());
    assert_eq!(
        run_import_with_progress(s.clone(), None, |phase| phases.borrow_mut().push(phase))
            .unwrap()
            .new_conversations,
        1
    );
    assert_eq!(*phases.borrow(), vec!["reading", "preparing", "saving"]);
    let mut guard = s.lock().unwrap();
    let v = guard.access().unwrap();
    assert!(v.list("", false, 0).unwrap().is_empty());
    assert_eq!(v.list("", true, 0).unwrap().len(), 1);
}

#[test]
fn schema_seven_upgrade_adds_empty_checkpoints_without_resetting_approvals() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("fictional.sqlite3");
    let v = Vault::create(&path, PASSWORD).unwrap();
    v.connection.execute_batch("INSERT INTO memory(id,type,content,approved,created_at,updated_at) VALUES('safe','fact','Cedar planters.',1,1,1); DROP TABLE import_items; DROP TABLE import_job; PRAGMA user_version=7;").unwrap();
    v.checkpoint().unwrap();
    drop(v);
    let v = Vault::open(&path, PASSWORD).unwrap();
    assert!(v.import_checkpoint().unwrap().is_none());
    assert_eq!(v.memories(None, "", true).unwrap().len(), 1);
    assert_eq!(
        v.connection
            .query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        8
    );
}

#[test]
fn killed_final_save_keeps_prepared_work_and_original_archive() {
    if let Some(path) = std::env::var_os("PD_IMPORT_CRASH_PATH") {
        let path = PathBuf::from(path);
        let s = store(path.parent().unwrap(), false);
        run_import(s, None).unwrap();
        panic!("Missed commit boundary");
    }
    let root = tempfile::tempdir().unwrap();
    let s = store(root.path(), true);
    let path = {
        let mut guard = s.lock().unwrap();
        prepared(guard.access().unwrap(), "checkpointorchid");
        guard.access().unwrap().checkpoint().unwrap();
        let path = guard.path().unwrap();
        guard.lock();
        path
    };
    drop(s);
    struct Child(std::process::Child);
    impl Drop for Child {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let mut child = Child(
        std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "import_jobs::tests::killed_final_save_keeps_prepared_work_and_original_archive",
                "--nocapture",
            ])
            .env("PD_IMPORT_CRASH_PATH", &path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap(),
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while !path.with_extension("ready").exists() {
        assert!(child.0.try_wait().unwrap().is_none());
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    child.0.kill().unwrap();
    child.0.wait().unwrap();
    let s = store(root.path(), false);
    assert_eq!(
        s.lock()
            .unwrap()
            .access()
            .unwrap()
            .stats()
            .unwrap()
            .conversations,
        0
    );
    assert_eq!(
        s.lock()
            .unwrap()
            .import_state()
            .unwrap()
            .checkpoint
            .unwrap()
            .saved,
        1
    );
    assert_eq!(run_import(s.clone(), None).unwrap().new_conversations, 1);
    s.lock().unwrap().access().unwrap().verify().unwrap();
}

#[test]
fn failed_checkpoint_write_preserves_the_previous_cursor() {
    let root = tempfile::tempdir().unwrap();
    let mut v = Vault::create(&root.path().join("fictional.sqlite3"), PASSWORD).unwrap();
    v.begin_import("fictional-digest", "fictional.zip", 1)
        .unwrap();
    v.connection.execute_batch("VACUUM;").unwrap();
    let pages: i64 = v
        .connection
        .query_row("PRAGMA page_count", [], |r| r.get(0))
        .unwrap();
    v.connection
        .pragma_update(None, "max_page_count", pages)
        .unwrap();
    let mut data = tree(0);
    data["mapping"]["leaf"]["message"]["content"]["parts"][0] =
        json!("fictional data ".repeat(30000));
    let item = parse_export_conversation(&data, false, &mut ImportReport::default()).unwrap();
    assert!(v
        .save_import_batch(0, &[item], 0)
        .unwrap_err()
        .contains("full"));
    assert_eq!(v.import_checkpoint().unwrap().unwrap().saved, 0);
    assert_eq!(
        v.connection
            .query_row("SELECT count(*) FROM import_items", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    v.connection
        .pragma_update(None, "max_page_count", 2147483646_i64)
        .unwrap();
    let item = parse_export_conversation(&data, false, &mut ImportReport::default()).unwrap();
    v.save_import_batch(0, &[item], 0).unwrap();
    assert_eq!(v.import_checkpoint().unwrap().unwrap().saved, 1);
    assert_eq!(v.stats().unwrap().conversations, 0);
}
