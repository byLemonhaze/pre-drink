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

fn client_request(client: &crate::agents::NewClient, tool: &str) -> crate::agents::AgentRequest {
    crate::agents::AgentRequest {
        client_id: client.client.id.clone(),
        token: client.token.clone(),
        tool: tool.into(),
        query: "garden".into(),
        limit: Some(50),
        conversation_id: String::new(),
        curation: None,
    }
}

#[test]
fn deletion_discards_ready_preparation_atomically_and_cannot_resurrect_history() {
    use crate::retention::{RemovalInput, RemovalKind};
    let root = tempfile::tempdir().unwrap();
    let s = store(root.path(), true);
    let input = {
        let mut session = s.lock().unwrap();
        let v = session.access().unwrap();
        let c = parse_export_conversation(&tree(0), false, &mut ImportReport::default()).unwrap();
        let input = RemovalInput {
            kind: RemovalKind::Conversation,
            conversation_id: Some(c.id.clone()),
        };
        v.import(&[c], ImportReport::default()).unwrap();
        prepared(v, "checkpointorchid staged duplicate");
        input
    };
    {
        let mut session = s.lock().unwrap();
        // A failed deletion must preserve the original and preparation together.
        session.access().unwrap().connection.execute_batch("CREATE TRIGGER fixture_fail_delete BEFORE DELETE ON conversations BEGIN SELECT RAISE(ABORT,'fictional failure'); END;").unwrap();
        let review = session.preview_removal(&input).unwrap();
        let pending = review.pending_import.as_ref().unwrap();
        assert_eq!((pending.saved, pending.total), (1, 1));
        assert!(session
            .apply_removal(&input, &review.review_hash, review.session_generation)
            .is_err());
        assert_eq!(session.import_state().unwrap().checkpoint.unwrap().saved, 1);
        assert_eq!(session.access().unwrap().stats().unwrap().conversations, 1);
        session
            .access()
            .unwrap()
            .connection
            .execute_batch("DROP TRIGGER fixture_fail_delete;")
            .unwrap();
        let review = session.preview_removal(&input).unwrap();
        session
            .apply_removal(&input, &review.review_hash, review.session_generation)
            .unwrap();
        assert!(
            session.import_state().unwrap().checkpoint.is_none(),
            "Deletion must remove pending copies, not only searchable history"
        );
        assert_eq!(session.access().unwrap().stats().unwrap().conversations, 0);
        assert_eq!(
            session
                .access()
                .unwrap()
                .connection
                .query_row("SELECT count(*) FROM import_items", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
    assert!(run_import(s.clone(), None).is_err());
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
}

#[test]
fn deleting_a_not_yet_prepared_conversation_also_invalidates_the_pending_zip() {
    use crate::retention::{RemovalInput, RemovalKind};
    let root = tempfile::tempdir().unwrap();
    let s = store(root.path(), true);
    let path = root.path().join("fictional.zip");
    zip(&path, &json!((0..45).map(tree).collect::<Vec<_>>()));
    let input = {
        let mut session = s.lock().unwrap();
        let items = [24, 99].map(|i| {
            parse_export_conversation(&tree(i), false, &mut ImportReport::default()).unwrap()
        });
        let input = RemovalInput {
            kind: RemovalKind::Conversation,
            conversation_id: Some(items[0].id.clone()),
        };
        session
            .access()
            .unwrap()
            .import(&items, ImportReport::default())
            .unwrap();
        input
    };
    ACTION.with(|a| a.set(Some("pause")));
    assert!(run_import(s.clone(), Some(path)).is_err());
    let mut session = s.lock().unwrap();
    assert_eq!(
        session.import_state().unwrap().checkpoint.unwrap().saved,
        20
    );
    let review = session.preview_removal(&input).unwrap();
    let pending = review.pending_import.as_ref().unwrap();
    assert_eq!((pending.saved, pending.total), (20, 45));
    session
        .apply_removal(&input, &review.review_hash, review.session_generation)
        .unwrap();
    assert!(session.import_state().unwrap().checkpoint.is_none());
    assert_eq!(session.access().unwrap().stats().unwrap().conversations, 1);
    assert_eq!(
        session.access().unwrap().list("99", true, 0).unwrap().len(),
        1
    );
}

#[test]
fn running_import_blocks_deletion_preview_and_submission_without_losing_work() {
    use crate::retention::{RemovalInput, RemovalKind};
    let root = tempfile::tempdir().unwrap();
    let s = store(root.path(), true);
    let path = root.path().join("fictional.zip");
    zip(&path, &json!([tree(0)]));
    let (input, review) = {
        let mut session = s.lock().unwrap();
        let c = parse_export_conversation(&tree(0), false, &mut ImportReport::default()).unwrap();
        let input = RemovalInput {
            kind: RemovalKind::Conversation,
            conversation_id: Some(c.id.clone()),
        };
        session
            .access()
            .unwrap()
            .import(&[c], ImportReport::default())
            .unwrap();
        let review = session.preview_removal(&input).unwrap();
        (input, review)
    };
    let checked = Cell::new(false);
    assert!(run_import_with_progress(s.clone(), Some(path), |phase| {
        if phase == "preparing" && !checked.replace(true) {
            let mut session = s.lock().unwrap();
            assert!(session
                .preview_removal(&input)
                .err()
                .unwrap()
                .contains("Pause"));
            assert!(session
                .apply_removal(&input, &review.review_hash, review.session_generation)
                .unwrap_err()
                .contains("Pause"));
            assert_eq!(session.access().unwrap().stats().unwrap().conversations, 1);
            session.pause_import().unwrap();
        }
    })
    .is_err());
    assert!(checked.get());
    assert!(s
        .lock()
        .unwrap()
        .import_state()
        .unwrap()
        .checkpoint
        .is_some());
}

#[test]
fn wiped_live_vault_restores_paused_import_and_resumes_without_old_client_access() {
    const BACKUP_PASSWORD: &str = "fictional checkpoint backup phrase";
    let root = tempfile::tempdir().unwrap();
    let live = root.path().join("disposable-live");
    std::fs::create_dir(&live).unwrap();
    let input = root.path().join("fictional.zip");
    let backup = root.path().join("backup.sqlite3");
    zip(&input, &json!((0..45).map(tree).collect::<Vec<_>>()));
    let s = store(&live, true);
    let client = s
        .lock()
        .unwrap()
        .access()
        .unwrap()
        .create_client("Fictional prior client", true)
        .unwrap();
    ACTION.with(|a| a.set(Some("pause")));
    assert!(run_import(s.clone(), Some(input.clone())).is_err());
    {
        let mut session = s.lock().unwrap();
        session
            .backup_with_receipt(&backup, BACKUP_PASSWORD)
            .unwrap();
        session.lock();
    }
    drop(s);
    let backup_bytes = std::fs::read(&backup).unwrap();
    std::fs::remove_dir_all(&live).unwrap();
    std::fs::create_dir(&live).unwrap();
    let mut restored = Session::new(live.clone());
    let review = restored.prepare_restore(&backup, BACKUP_PASSWORD).unwrap();
    assert_eq!(review.conversations, 0);
    assert_eq!(review.clients_to_remove, 1);
    restored
        .apply_replacement(&review.review_id, BACKUP_PASSWORD)
        .unwrap();
    drop(restored);
    let mut restored = Session::new(live);
    restored.unlock(BACKUP_PASSWORD, false, false).unwrap();
    let state = restored.import_state().unwrap();
    assert!(!state.running);
    assert_eq!(state.checkpoint.unwrap().saved, 20);
    assert!(!restored.sharing);
    assert!(restored
        .access()
        .unwrap()
        .agent_call(&client_request(&client, "search_conversations"))
        .is_err());
    let s = Arc::new(Mutex::new(restored));
    PARSED.with(|p| p.set(0));
    let result = run_import(s.clone(), Some(input)).unwrap();
    assert_eq!((result.new_conversations, result.messages), (45, 45));
    assert_eq!(PARSED.with(|p| p.get()), 25);
    assert!(s
        .lock()
        .unwrap()
        .import_state()
        .unwrap()
        .checkpoint
        .is_none());
    s.lock().unwrap().access().unwrap().verify().unwrap();
    assert_eq!(std::fs::read(backup).unwrap(), backup_bytes);
}

#[test]
fn password_rotation_preserves_pending_import_but_never_resumes_it_or_clients() {
    const NEXT: &str = "fictional rotated checkpoint phrase";
    let root = tempfile::tempdir().unwrap();
    let input = root.path().join("fictional.zip");
    zip(&input, &json!((0..25).map(tree).collect::<Vec<_>>()));
    let s = store(root.path(), true);
    let client = s
        .lock()
        .unwrap()
        .access()
        .unwrap()
        .create_client("Fictional old client", true)
        .unwrap();
    ACTION.with(|a| a.set(Some("pause")));
    assert!(run_import(s.clone(), Some(input.clone())).is_err());
    {
        let mut session = s.lock().unwrap();
        let original = session.path().unwrap();
        let review = session.prepare_password_change(PASSWORD, NEXT).unwrap();
        session.apply_replacement(&review.review_id, NEXT).unwrap();
        assert!(session.vault.is_none());
        assert!(session.unlock(PASSWORD, false, false).is_err());
        session.unlock(NEXT, false, false).unwrap();
        let state = session.import_state().unwrap();
        assert!(!state.running);
        assert_eq!(state.checkpoint.unwrap().saved, 20);
        assert!(!session.sharing);
        assert!(session
            .access()
            .unwrap()
            .agent_call(&client_request(&client, "get_projects"))
            .is_err());
        // The retained old copy is still recoverable only with its original password.
        assert_eq!(
            Vault::open(&original, PASSWORD)
                .unwrap()
                .import_checkpoint()
                .unwrap()
                .unwrap()
                .saved,
            20
        );
    }
    assert!(run_import(s.clone(), None)
        .unwrap_err()
        .contains("original ZIP"));
    assert_eq!(
        run_import(s.clone(), Some(input))
            .unwrap()
            .new_conversations,
        25
    );
    assert!(s
        .lock()
        .unwrap()
        .import_state()
        .unwrap()
        .checkpoint
        .is_none());
}

#[test]
fn resumed_mixed_provider_import_keeps_preparation_private_and_provider_walls_intact() {
    use crate::curation::CurationOperation;
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("mixed-fictional.zip");
    let claude: Value = serde_json::from_str(include_str!(
        "../tests/fixtures/claude/messages-v1/garden.input.json"
    ))
    .unwrap();
    let grok: Value = serde_json::from_str(include_str!(
        "../tests/fixtures/grok/account-v1/garden.input.json"
    ))
    .unwrap();
    let mut writer = zip::ZipWriter::new(File::create(&path).unwrap());
    for (name, data) in [
        (
            "chatgpt/conversations.json",
            json!((0..8).map(tree).collect::<Vec<_>>()),
        ),
        (
            "claude/conversations.json",
            json!((0..8)
                .map(|i| {
                    let mut c = claude[0].clone();
                    c["uuid"] = json!(format!("fictional-{i}"));
                    c
                })
                .collect::<Vec<_>>()),
        ),
        (
            "prod-grok-backend.json",
            json!({"conversations":(0..8).map(|i| {let mut c=grok["conversations"][0].clone();c["conversation"]["id"]=json!(format!("fictional-{i}"));c}).collect::<Vec<_>>()}),
        ),
    ] {
        writer
            .start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(data.to_string().as_bytes()).unwrap();
    }
    writer.finish().unwrap();
    let s = store(root.path(), true);
    let clients = {
        let mut session = s.lock().unwrap();
        ["chatgpt", "claude", "grok"].map(|p| {
            session
                .access()
                .unwrap()
                .create_scoped_client("Fictional curator", Some(p), true)
                .unwrap()
        })
    };
    ACTION.with(|a| a.set(Some("pause")));
    assert!(run_import(s.clone(), Some(path.clone())).is_err());
    {
        let mut session = s.lock().unwrap();
        assert_eq!(
            session.import_state().unwrap().checkpoint.unwrap().saved,
            20
        );
        let v = session.access().unwrap();
        for client in &clients {
            assert_eq!(
                v.agent_call(&client_request(client, "search_conversations"))
                    .unwrap(),
                json!([])
            );
            let mut r = client_request(client, "curate_memory");
            r.curation = Some(CurationOperation::ListSources {
                after: None,
                limit: Some(50),
            });
            assert_eq!(v.agent_call(&r).unwrap()["sources"], json!([]));
            assert_eq!(
                v.agent_call(&client_request(client, "get_projects"))
                    .unwrap(),
                json!([])
            );
        }
    }
    let report = run_import(s.clone(), Some(path)).unwrap();
    assert_eq!(
        (
            report.new_conversations,
            report.messages,
            report.skipped_messages
        ),
        (24, 48, 24)
    );
    let mut session = s.lock().unwrap();
    let v = session.access().unwrap();
    let mut source_ids = Vec::new();
    for client in &clients {
        let matches = v
            .agent_call(&client_request(client, "search_conversations"))
            .unwrap();
        assert_eq!(matches.as_array().unwrap().len(), 8);
        assert!(matches
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c["platform"].as_str() == client.client.provider.as_deref()));
        let mut r = client_request(client, "curate_memory");
        r.curation = Some(CurationOperation::ListSources {
            after: None,
            limit: Some(50),
        });
        source_ids.push(
            v.agent_call(&r).unwrap()["sources"][0]["message_id"]
                .as_str()
                .unwrap()
                .to_owned(),
        );
    }
    for (i, client) in clients.iter().enumerate() {
        for (j, id) in source_ids.iter().enumerate() {
            let mut r = client_request(client, "curate_memory");
            r.curation = Some(CurationOperation::ReadSource {
                message_id: id.clone(),
                offset: None,
            });
            assert_eq!(v.agent_call(&r).is_ok(), i == j);
        }
        assert_eq!(
            v.agent_call(&client_request(client, "get_projects"))
                .unwrap(),
            json!([])
        );
    }
    v.verify().unwrap();
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
        10
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
    crate::test_support::wait_for_checkpoint(&mut child.0, &path.with_extension("ready"));
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
