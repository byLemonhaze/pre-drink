#[path = "support/historical_vault.rs"]
mod historical;
use historical::{connect, create, number, rows, snapshot, PASSWORD, TOKEN};
use pre_drink::{agents::AgentRequest, db::Vault};
use std::path::Path;

fn request(tool: &str) -> AgentRequest {
    AgentRequest {
        client_id: "client".into(),
        token: TOKEN.into(),
        tool: tool.into(),
        query: "cedar".into(),
        limit: None,
        conversation_id: String::new(),
        curation: None,
    }
}

fn check(version: i64, upgraded_v5: bool) {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("fictional-historical.sqlite3");
    create(&path, version, upgraded_v5);
    let c = connect(&path);
    let before = snapshot(&c);
    drop(c);
    let encrypted_before = std::fs::read(&path).unwrap();
    assert!(!encrypted_before.starts_with(b"SQLite format 3"));
    assert!(Vault::open(&path, "incorrect fictional phrase").is_err());
    assert_eq!(std::fs::read(&path).unwrap(), encrypted_before);
    let mut v = Vault::open(&path, PASSWORD).unwrap();
    v.verify().unwrap();
    assert_eq!(v.stats().unwrap().conversations, 4);
    let c = connect(&path);
    assert_eq!(number(&c, "PRAGMA user_version"), 8);
    for table in ["messages", "conversations"] {
        // Text, raw JSON and branch structure survive; only the privacy bit changes.
        let columns = if table == "messages" {
            "id,conversation_id,role,content,timestamp,branch_id,parent_id,is_active,position"
        } else {
            "id,platform,title,created_at,updated_at,private,raw_json"
        };
        let old = before[table]
            .iter()
            .map(|row| {
                let sensitive = if table == "messages" { 8 } else { 6 };
                row.iter()
                    .enumerate()
                    .filter(|(i, _)| *i != sensitive)
                    .map(|(_, v)| v.clone())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            rows(&c, &format!("SELECT {columns} FROM {table} ORDER BY rowid")),
            old
        );
    }
    for table in [
        "tags",
        "conversation_tags",
        "archive_snapshots",
        "brief_sources",
        "search_index",
    ] {
        assert_eq!(
            rows(&c, &format!("SELECT * FROM {table} ORDER BY rowid")),
            before[table]
        );
    }
    for (table, columns, ignored) in [
        (
            "memory",
            "id,type,content,source_message_id,had_source,private,created_at,updated_at",
            [4, 7],
        ),
        ("briefs", "id,title,content,private,updated_at", [3, 5]),
    ] {
        let expected = before[table]
            .iter()
            .map(|row| {
                row.iter()
                    .enumerate()
                    .filter(|(i, _)| !ignored.contains(i))
                    .map(|(_, value)| value.clone())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            rows(&c, &format!("SELECT {columns} FROM {table} ORDER BY rowid")),
            expected
        );
    }
    assert_eq!(
        number(&c, "SELECT private FROM conversations WHERE id='private'"),
        1
    );
    assert_eq!(
        number(&c, "SELECT sensitive FROM conversations WHERE id='flagged'"),
        1
    );
    assert_eq!(
        number(
            &c,
            "SELECT sensitive FROM conversations WHERE id='newly-sensitive'"
        ),
        1
    );
    assert_eq!(
        number(&c, "SELECT approved FROM memory WHERE id='linked-secret'"),
        0
    );
    assert_eq!(
        number(&c, "SELECT approved FROM briefs WHERE id='linked-brief'"),
        0
    );
    assert_eq!(
        number(&c, "SELECT approved FROM memory WHERE id='safe-note'"),
        i64::from(version >= 3)
    );
    assert_eq!(
        number(&c, "SELECT approved FROM briefs WHERE id='safe-brief'"),
        i64::from(version >= 3)
    );
    assert_eq!(
        number(&c, "SELECT had_source FROM memory WHERE id='orphan'"),
        1
    );
    assert_eq!(number(&c, "SELECT count(*) FROM import_job"), 0);
    if version < 4 {
        assert_eq!(
            number(
                &c,
                "SELECT allow_chats FROM agent_clients WHERE id='client'"
            ),
            0
        );
        assert_eq!(number(&c, "SELECT count(*) FROM agent_scopes"), 0);
        assert!(v.agent_call(&request("search_conversations")).is_err());
    } else {
        for table in [
            "agent_clients",
            "agent_scopes",
            "curation_notes",
            "curation_sources",
            "curation_revisions",
            "curation_requests",
            "curation_jobs",
            "curation_job_sources",
            "curation_processed_sources",
            "curation_publications",
        ] {
            assert_eq!(
                rows(&c, &format!("SELECT * FROM {table} ORDER BY rowid")),
                before[table]
            );
        }
        assert_eq!(
            number(
                &c,
                "SELECT count(*) FROM curation_owner_holds WHERE note_id='held'"
            ),
            1
        );
        assert_eq!(
            number(
                &c,
                "SELECT publish_session FROM curation_policy WHERE provider='chatgpt'"
            ),
            1
        );
        assert_eq!(number(&c, "SELECT count(*) FROM curation_policy"), 3);
        assert!(v.agent_call(&request("search_conversations")).is_ok());
    }
    let shared = v
        .agent_call(&request("get_context_pack"))
        .unwrap()
        .to_string();
    assert!(!shared.contains("fictional-lilac"));
    assert!(!shared.contains("orphaned"));
    assert_eq!(
        v.agent_call(&request("get_facts")).unwrap(),
        serde_json::json!([])
    );
    assert_eq!(
        v.agent_call(&request("get_decisions")).unwrap(),
        serde_json::json!([])
    );
    drop(c);
    // Fresh review after the old v2 reset must survive subsequent opens.
    for (kind, id) in [("memory", "safe-note"), ("brief", "safe-brief")] {
        let review = v.approval_review(kind, id).unwrap();
        v.approve_review(kind, id, &review.review_hash).unwrap();
    }
    v.checkpoint().unwrap();
    drop(v);
    let c = connect(&path);
    let once = snapshot(&c);
    drop(c);
    let v = Vault::open(&path, PASSWORD).unwrap();
    v.verify().unwrap();
    let c = connect(&path);
    assert_eq!(snapshot(&c), once);
    drop(c);
    let backup = root.path().join("fictional-upgraded-backup.sqlite3");
    v.encrypted_backup(&backup, PASSWORD).unwrap();
    let b = Vault::open(&backup, PASSWORD).unwrap();
    b.verify().unwrap();
    drop(b);
    let c = connect(&backup);
    assert_eq!(snapshot(&c), once);
    drop(c);
    // A migrated table must accept new provider jobs, not merely retain old rows.
    for provider in ["grok", "claude"] {
        let client = v
            .create_scoped_client("Fictional new curator", Some(provider), true)
            .unwrap();
        let mut r = request("curate_memory");
        r.client_id = client.client.id;
        r.token = client.token;
        r.curation = Some(pre_drink::curation::CurationOperation::SaveJob {
            id: None,
            expected_revision: 0,
            label: "Orchard review".into(),
            checkpoint: 0,
            status: "running".into(),
            request_key: "fictional-new-job".into(),
        });
        let job = v.agent_call(&r).unwrap();
        assert_eq!(job["source_count"], 0); // ChatGPT history remains foreign.
        assert_eq!(job["revision"], 1);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&backup).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}

macro_rules! upgrade_case {
    ($name:ident,$version:expr,$upgraded:expr) => {
        #[test]
        fn $name() {
            check($version, $upgraded);
        }
    };
}
upgrade_case!(historical_v2_resets_once_and_preserves_archive, 2, false);
upgrade_case!(
    historical_v3_preserves_reviewed_content_and_downgrades_broad_access,
    3,
    false
);
upgrade_case!(historical_v4_rebuild_preserves_curation_children, 4, false);
upgrade_case!(historical_v5_fresh_creation_upgrades, 5, false);
upgrade_case!(
    historical_v5_previously_upgraded_jobs_remain_usable,
    5,
    true
);
upgrade_case!(historical_v6_conflicts_become_owner_holds, 6, false);
upgrade_case!(
    historical_v7_adds_checkpoints_without_resetting_safe_reviews,
    7,
    false
);

fn failed_open(path: &Path) -> String {
    match Vault::open(path, PASSWORD) {
        Ok(_) => panic!("damaged fixture was accepted"),
        Err(e) => e,
    }
}

#[test]
fn historical_privacy_migration_failure_rolls_back_and_can_retry() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("fictional.sqlite3");
    create(&path, 2, false);
    let c = connect(&path);
    c.execute_batch("CREATE TRIGGER fictional_failure BEFORE UPDATE OF approved ON briefs BEGIN SELECT RAISE(ABORT,'fictional migration failure'); END;").unwrap();
    let before = snapshot(&c);
    drop(c);
    assert!(failed_open(&path).contains("fictional migration failure"));
    let c = connect(&path);
    assert_eq!(number(&c, "PRAGMA user_version"), 2);
    assert_eq!(snapshot(&c), before);
    c.execute_batch("DROP TRIGGER fictional_failure;").unwrap();
    drop(c);
    let v = Vault::open(&path, PASSWORD).unwrap();
    v.verify().unwrap();
}

#[test]
fn historical_provider_rebuild_rejects_broken_references_without_losing_rows() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("fictional.sqlite3");
    create(&path, 4, false);
    let c = connect(&path);
    c.execute_batch("PRAGMA foreign_keys=OFF; INSERT INTO curation_job_sources VALUES('missing-job',2,'safe-root',1);").unwrap();
    let before = snapshot(&c);
    drop(c);
    assert!(failed_open(&path).contains("invalid references"));
    let c = connect(&path);
    assert_eq!(number(&c, "PRAGMA user_version"), 4);
    assert_eq!(snapshot(&c), before);
    c.execute_batch("DELETE FROM curation_job_sources WHERE job_id='missing-job';")
        .unwrap();
    drop(c);
    let v = Vault::open(&path, PASSWORD).unwrap();
    v.verify().unwrap();
}

#[test]
fn historical_v2_upgrade_reports_review_resets_on_first_unlock_only() {
    let root = tempfile::tempdir().unwrap();
    let mut session = pre_drink::session::Session::new(root.path().to_owned());
    create(&session.path().unwrap(), 2, false);
    session.unlock(PASSWORD, false, false).unwrap();
    let status = session.status().unwrap();
    assert!(!status.sharing);
    assert_eq!(status.privacy_on_open.flagged, 2);
    assert_eq!(status.privacy_on_open.approvals_reset, 5);
    assert!(status.privacy_on_open.legacy_review_upgrade);
    session.lock();
    session.unlock(PASSWORD, false, false).unwrap();
    let status = session.status().unwrap();
    assert_eq!(status.privacy_on_open.flagged, 0);
    assert_eq!(status.privacy_on_open.approvals_reset, 0);
    assert!(!status.privacy_on_open.legacy_review_upgrade);
}

#[test]
fn historical_ddl_matches_recorded_source_hashes() {
    use sha2::{Digest, Sha256};
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/migrations/historical");
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("manifest.json")).unwrap()).unwrap();
    for file in manifest["files"].as_array().unwrap() {
        let raw = std::fs::read(root.join(file["file"].as_str().unwrap())).unwrap();
        assert_eq!(
            hex::encode(Sha256::digest(raw)),
            file["sha256"].as_str().unwrap()
        );
    }
}

#[test]
fn historical_backup_restore_migrates_only_the_copy_and_revokes_old_clients() {
    let root = tempfile::tempdir().unwrap();
    let backup = root.path().join("fictional-old-backup.sqlite3");
    create(&backup, 4, false);
    let original = std::fs::read(&backup).unwrap();
    let live = root.path().join("fresh-live");
    std::fs::create_dir(&live).unwrap();
    let mut s = pre_drink::session::Session::new(live);
    let review = s.prepare_restore(&backup, PASSWORD).unwrap();
    assert_eq!(review.clients_to_remove, 1);
    assert_eq!(review.conversations, 4);
    assert!(
        !s.apply_replacement(&review.review_id, PASSWORD)
            .unwrap()
            .durability_warning
    );
    assert!(!s.status().unwrap().unlocked);
    s.unlock(PASSWORD, false, false).unwrap();
    assert!(!s.status().unwrap().sharing);
    let v = s.access().unwrap();
    v.verify().unwrap();
    assert!(v.clients().unwrap().is_empty());
    assert!(v.agent_call(&request("get_facts")).is_err());
    let c = connect(&s.path().unwrap());
    assert_eq!(number(&c, "PRAGMA user_version"), 8);
    assert_eq!(number(&c, "SELECT count(*) FROM curation_revisions"), 1);
    assert_eq!(number(&c, "SELECT count(*) FROM curation_owner_holds"), 1);
    assert_eq!(
        number(
            &c,
            "SELECT sum(publish_archive+publish_session) FROM curation_policy"
        ),
        0
    );
    assert_eq!(std::fs::read(&backup).unwrap(), original);
    let old = connect(&backup);
    assert_eq!(number(&old, "PRAGMA user_version"), 4);
}
