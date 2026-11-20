//! Destructive acceptance test: every file lives under a disposable temp directory.
use pre_drink::{
    agents::AgentRequest, briefs::BriefInput, crypto, db::Vault, import::parse_conversation,
    memory::MemoryInput, model::ImportReport, session::Session,
};
use rusqlite::{types::Value, Connection};
use serde_json::json;
use std::path::Path;

const ORIGINAL_PASSWORD: &str = "synthetic original vault phrase";
const BACKUP_PASSWORD: &str = "synthetic independent backup phrase";

fn snapshot(path: &Path, password: &str) -> Vec<Vec<Vec<Value>>> {
    let connection = Connection::open(path).unwrap();
    let key = crypto::key(password, &crypto::salt(path).unwrap()).unwrap();
    crypto::configure(&connection, &key).unwrap();
    [
        "conversations",
        "messages",
        "memory",
        "briefs",
        "brief_sources",
        "archive_snapshots",
        "agent_clients",
        "tags",
        "conversation_tags",
        "search_index",
    ]
    .into_iter()
    .map(|table| {
        let mut query = connection
            .prepare(&format!("SELECT * FROM {table} ORDER BY rowid"))
            .unwrap();
        let columns = query.column_count();
        query
            .query_map([], |row| {
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

#[test]
fn wipe_synthetic_vault_restore_backup_and_verify_data_provenance_and_policy() {
    wipe_and_restore(false);
}

#[test]
fn wipe_synthetic_vault_restore_through_app_lifecycle_and_reset_client_access() {
    wipe_and_restore(true);
}

fn wipe_and_restore(through_app: bool) {
    let root = tempfile::tempdir().unwrap();
    let live = root.path().join("disposable-live");
    std::fs::create_dir(&live).unwrap();
    let path = live.join("vault.enc.sqlite3");
    let backup = root.path().join("verified-backup.sqlite3");
    let mut v = Vault::create(&path, ORIGINAL_PASSWORD).unwrap();
    let c = parse_conversation(&json!({
        "id":"recovery-fixture", "title":"Synthetic garden archive", "current_node":"active",
        "mapping": {
            "root":{"parent":null,"message":{"author":{"role":"user"},"content":{"parts":["Plan a garden."]}}},
            "active":{"parent":"root","message":{"author":{"role":"assistant"},"content":{"parts":["Use native plants."]}}},
            "alternate":{"parent":"root","message":{"author":{"role":"assistant"},"content":{"parts":["An earlier garden plan."]}}}
        }
    }), &mut ImportReport::default()).unwrap();
    let conversation = c.id.clone();
    let source = c.messages[0].id.clone();
    v.import(&[c], ImportReport::default()).unwrap();
    let note = v
        .save_memory(MemoryInput {
            id: None,
            kind: "project".into(),
            content: "Native garden project.".into(),
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
            title: "Garden plan".into(),
            content: "Plant a native garden.".into(),
            private: false,
            source_ids: vec![note],
        })
        .unwrap();
    let review = v.approval_review("brief", &brief).unwrap();
    v.approve_review("brief", &brief, &review.review_hash)
        .unwrap();
    v.mark_private(&conversation, true).unwrap();
    v.save_memory(MemoryInput {
        id: None,
        kind: "fact".into(),
        content: "Fictional contact: garden@example.invalid".into(),
        source_message_id: None,
        private: false,
    })
    .unwrap();
    let client = v.create_client("Synthetic recovery client", false).unwrap();
    v.encrypted_backup(&backup, BACKUP_PASSWORD).unwrap();
    v.checkpoint().unwrap();
    drop(v);
    let mut expected = snapshot(&path, ORIGINAL_PASSWORD);
    let backup_bytes = std::fs::read(&backup).unwrap();
    assert_eq!(snapshot(&backup, BACKUP_PASSWORD), expected);
    // Delete the entire synthetic live directory, including any journals.
    std::fs::remove_dir_all(&live).unwrap();
    assert!(!path.exists());
    std::fs::create_dir(&live).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&live, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    let mut restored = Session::new(live.clone());
    if through_app {
        let preview = restored.prepare_restore(&backup, BACKUP_PASSWORD).unwrap();
        assert_eq!(preview.conversations, 1);
        assert_eq!(preview.messages, 3);
        assert_eq!(preview.memories, 2);
        assert_eq!(preview.briefs, 1);
        assert_eq!(preview.clients_to_remove, 1);
        assert!(preview.previous_vault.is_none());
        assert!(!path.exists(), "Inspection must not publish a vault");
        let result = restored
            .apply_replacement(&preview.review_id, BACKUP_PASSWORD)
            .unwrap();
        assert!(!result.durability_warning);
        assert!(result.previous_vault.is_none());
        assert!(restored.vault.is_none());
        // Recreate the session to use the persisted generation selector, not in-memory state.
        drop(restored);
        restored = Session::new(live);
        // In-app restore intentionally discards old agent credentials; every other
        // logical table, including approvals, provenance and FTS, must match.
        expected[6].clear(); // agent_clients, in snapshot's explicit table order
    } else {
        std::fs::copy(&backup, &path).unwrap();
    }
    let active = restored.path().unwrap();
    assert!(Vault::open(&active, ORIGINAL_PASSWORD).is_err());
    restored.unlock(BACKUP_PASSWORD, false, false).unwrap();
    assert!(
        !restored.sharing,
        "Restoring registrations must never enable sharing"
    );
    let v = restored.access().unwrap();
    v.verify().unwrap();
    assert_eq!(v.messages(&conversation, true).unwrap().len(), 3);
    assert_eq!(v.stats().unwrap().private, 1);
    assert!(v.memories(None, "", true).unwrap().is_empty());
    assert!(v.briefs(true, "").unwrap().is_empty());
    assert_eq!(v.clients().unwrap().len(), usize::from(!through_app));
    let request = AgentRequest {
        client_id: client.client.id.clone(),
        token: client.token,
        tool: "get_projects".into(),
        query: String::new(),
        limit: None,
        conversation_id: String::new(),
    };
    if through_app {
        assert!(
            v.agent_call(&request).is_err(),
            "Restored credentials must not regain access"
        );
    } else {
        assert_eq!(v.agent_call(&request).unwrap(), json!([]));
    }
    assert_eq!(snapshot(&active, BACKUP_PASSWORD), expected);
    if !through_app {
        v.revoke_client(&client.client.id).unwrap();
    }
    assert!(v.agent_call(&request).is_err());
    restored.lock();
    assert_eq!(std::fs::read(&backup).unwrap(), backup_bytes);
    assert!(
        backup.exists(),
        "Original backup remains available after restore"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(active).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}
