use pre_drink::{
    agents::AgentRequest,
    briefs::BriefInput,
    db::Vault,
    import::parse_conversation,
    memory::MemoryInput,
    model::ImportReport,
    session::{Session, IDLE_TIMEOUT},
};
use serde_json::json;
const PASSWORD: &str = "a long unique vault phrase";
fn memory(v: &Vault, approved: bool, source: Option<String>) -> String {
    v.save_memory(MemoryInput {
        id: None,
        kind: "decision".into(),
        content: "Pre-Drink supports ChatGPT only.".into(),
        source_message_id: source,
        approved,
        private: false,
    })
    .unwrap()
}
fn sample(v: &mut Vault) -> (String, String) {
    let c=parse_conversation(&json!({"id":"story","title":"Pre-Drink planning","current_node":"m","mapping":{"m":{"parent":null,"message":{"author":{"role":"user"},"content":{"parts":["We will keep Pre-Drink local."]}}}}}),&mut ImportReport::default()).unwrap();
    let result = (c.id.clone(), c.messages[0].id.clone());
    v.import(&[c], ImportReport::default()).unwrap();
    result
}
fn request(id: &str, token: &str, tool: &str) -> AgentRequest {
    AgentRequest {
        client_id: id.into(),
        token: token.into(),
        tool: tool.into(),
        query: "Pre-Drink".into(),
        limit: Some(10),
        conversation_id: String::new(),
    }
}

#[test]
fn encrypted_database_and_journal_do_not_contain_plaintext() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("vault.enc.sqlite3");
    let mut v = Vault::create(&path, PASSWORD).unwrap();
    sample(&mut v);
    memory(&v, true, None);
    for name in ["vault.enc.sqlite3", "vault.enc.sqlite3-wal"] {
        let bytes = std::fs::read(dir.path().join(name)).unwrap();
        assert!(!bytes.starts_with(b"SQLite format 3\0"));
        assert!(!bytes.windows(b"Pre-Drink".len()).any(|s| s == b"Pre-Drink"));
        assert!(!bytes
            .windows(PASSWORD.len())
            .any(|s| s == PASSWORD.as_bytes()));
    }
    drop(v);
    assert!(Vault::open(&path, "the wrong password").is_err());
    let unkeyed = rusqlite::Connection::open(&path).unwrap();
    assert!(unkeyed
        .query_row("SELECT count(*) FROM sqlite_master", [], |r| r
            .get::<_, i64>(0))
        .is_err());
    drop(unkeyed);
    let opened = Vault::open(&path, PASSWORD).unwrap();
    opened.verify().unwrap();
    assert_eq!(opened.memories(None, "", false).unwrap().len(), 1);
}

#[test]
fn corruption_is_detected() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("vault");
    {
        let mut v = Vault::create(&path, PASSWORD).unwrap();
        sample(&mut v);
        v.checkpoint().unwrap();
    }
    let mut bytes = std::fs::read(&path).unwrap();
    bytes[120] ^= 0x80;
    std::fs::write(&path, bytes).unwrap();
    assert!(Vault::open(&path, PASSWORD).is_err());
}

#[test]
fn different_vaults_have_different_salts_and_ciphertext() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a");
    let b = dir.path().join("b");
    drop(Vault::create(&a, PASSWORD).unwrap());
    drop(Vault::create(&b, PASSWORD).unwrap());
    assert_ne!(
        pre_drink::crypto::salt(&a).unwrap(),
        pre_drink::crypto::salt(&b).unwrap()
    );
}

#[test]
fn weak_passwords_and_overwrites_are_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("vault");
    assert!(Vault::create(&path, "short").is_err());
    assert!(!path.exists());
    drop(Vault::create(&path, PASSWORD).unwrap());
    let original = std::fs::read(&path).unwrap();
    assert!(Vault::create(&path, "another secure phrase").is_err());
    assert_eq!(original, std::fs::read(path).unwrap());
}

#[test]
fn backup_is_encrypted_verified_and_opens_with_its_own_password() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("backup");
    let mut v = Vault::memory().unwrap();
    sample(&mut v);
    memory(&v, true, None);
    v.encrypted_backup(&path, PASSWORD).unwrap();
    assert!(Vault::open(&path, "wrong password").is_err());
    let restored = Vault::open(&path, PASSWORD).unwrap();
    assert_eq!(restored.stats().unwrap().messages, 1);
    assert_eq!(restored.memories(None, "", false).unwrap().len(), 1);
    assert!(v.encrypted_backup(&path, PASSWORD).is_err());
}

#[test]
fn migrates_old_plaintext_without_losing_private_flags() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("vault.sqlite3");
    let target = dir.path().join("vault.enc.sqlite3");
    let old = rusqlite::Connection::open(&source).unwrap();
    let schema = include_str!("../src/schema.sql")
        .replace("  had_source INTEGER NOT NULL DEFAULT 0,\n", "")
        .replace("PRAGMA user_version = 2;", "PRAGMA user_version = 1;");
    old.execute_batch(&schema).unwrap();
    old.execute("INSERT INTO conversations(id,platform,title,private,raw_json) VALUES ('old','chatgpt','Old private history',1,'{}')",[]).unwrap();
    drop(old);
    let v = Vault::migrate_plaintext(&source, &target, PASSWORD).unwrap();
    assert_eq!(v.stats().unwrap().private, 1);
    assert!(v.list("", false, 0).unwrap().is_empty());
    assert!(source.exists());
    assert!(std::fs::read(&source)
        .unwrap()
        .starts_with(b"SQLite format 3\0"));
    assert!(!std::fs::read(&target)
        .unwrap()
        .starts_with(b"SQLite format 3\0"));
}

#[test]
fn unapproved_private_and_sensitive_memory_never_reaches_agents() {
    let v = Vault::memory().unwrap();
    memory(&v, false, None);
    let id = memory(&v, true, None);
    assert_eq!(v.memories(None, "", true).unwrap().len(), 1);
    v.save_memory(MemoryInput {
        id: Some(id.clone()),
        kind: "fact".into(),
        content: "Pre-Drink is private.".into(),
        source_message_id: None,
        approved: true,
        private: true,
    })
    .unwrap();
    assert!(v.memories(None, "", true).unwrap().is_empty());
    v.save_memory(MemoryInput {
        id: Some(id),
        kind: "fact".into(),
        content: "Pre-Drink email me@example.org".into(),
        source_message_id: None,
        approved: true,
        private: false,
    })
    .unwrap();
    assert!(v.memories(None, "", true).unwrap().is_empty());
}

#[test]
fn source_privacy_propagates_through_memory_and_briefs() {
    let mut v = Vault::memory().unwrap();
    let (conversation, source) = sample(&mut v);
    let note = memory(&v, true, Some(source));
    v.save_brief(BriefInput {
        id: None,
        title: "Pre-Drink spec".into(),
        content: "Pre-Drink supports ChatGPT only.".into(),
        approved: true,
        private: false,
        source_ids: vec![note.clone()],
    })
    .unwrap();
    assert_eq!(v.briefs(true, "").unwrap().len(), 1);
    v.mark_private(&conversation, true).unwrap();
    assert!(v.memories(None, "", true).unwrap().is_empty());
    assert!(v.briefs(true, "").unwrap().is_empty());
    assert!(!v
        .context_pack("Pre-Drink", false)
        .unwrap()
        .contains("supports ChatGPT only"));
    v.save_memory(MemoryInput {
        id: Some(note),
        kind: "fact".into(),
        content: "Pre-Drink source stays attached.".into(),
        source_message_id: None,
        approved: true,
        private: false,
    })
    .unwrap();
    assert!(v.memories(None, "", true).unwrap().is_empty());
}

#[test]
fn briefs_need_review_after_source_changes_and_fail_closed_on_deletion() {
    let mut v = Vault::memory().unwrap();
    let note = memory(&v, true, None);
    let id = v
        .save_brief(BriefInput {
            id: None,
            title: "Spec".into(),
            content: "Pre-Drink spec.".into(),
            approved: true,
            private: false,
            source_ids: vec![note.clone()],
        })
        .unwrap();
    v.save_memory(MemoryInput {
        id: Some(note.clone()),
        kind: "decision".into(),
        content: "Pre-Drink is a Mac app.".into(),
        source_message_id: None,
        approved: true,
        private: false,
    })
    .unwrap();
    assert!(v.briefs(true, "").unwrap().is_empty());
    v.save_brief(BriefInput {
        id: Some(id),
        title: "Spec".into(),
        content: "Pre-Drink updated spec.".into(),
        approved: true,
        private: false,
        source_ids: vec![note.clone()],
    })
    .unwrap();
    assert_eq!(v.briefs(true, "").unwrap().len(), 1);
    v.delete_memory(&note).unwrap();
    assert!(v.briefs(true, "").unwrap().is_empty());
}

#[test]
fn clients_need_valid_tokens_and_scopes_and_can_be_revoked() {
    let mut v = Vault::memory().unwrap();
    sample(&mut v);
    memory(&v, true, None);
    let client = v.create_client("Memory client", false).unwrap();
    let req = request(&client.client.id, &client.token, "get_decisions");
    assert_eq!(v.agent_call(&req).unwrap().as_array().unwrap().len(), 1);
    assert!(v
        .agent_call(&request(
            &client.client.id,
            &"0".repeat(64),
            "get_decisions"
        ))
        .is_err());
    assert!(v
        .agent_call(&request(
            &client.client.id,
            &client.token,
            "search_conversations"
        ))
        .is_err());
    assert!(v
        .agent_call(&request(&client.client.id, &client.token, "mark_private"))
        .is_err());
    v.revoke_client(&client.client.id).unwrap();
    assert!(v.agent_call(&req).is_err());
}

#[test]
fn chat_scope_does_not_bypass_privacy() {
    let mut v = Vault::memory().unwrap();
    let (id, _) = sample(&mut v);
    let c = v.create_client("Chat client", true).unwrap();
    let req = request(&c.client.id, &c.token, "search_conversations");
    assert_eq!(v.agent_call(&req).unwrap().as_array().unwrap().len(), 1);
    let mut mark = request(&c.client.id, &c.token, "mark_private");
    mark.conversation_id = id;
    v.agent_call(&mark).unwrap();
    assert_eq!(v.agent_call(&req).unwrap().as_array().unwrap().len(), 0);
}

#[test]
fn locking_and_idle_expiry_drop_vault_and_sharing() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Session::new(dir.path().into());
    s.unlock(PASSWORD, true, false).unwrap();
    s.sharing = true;
    s.last_activity =
        std::time::SystemTime::now() - IDLE_TIMEOUT - std::time::Duration::from_secs(1);
    assert!(s.access().is_err());
    assert!(!s.sharing);
    assert!(s.vault.is_none());
    s.unlock(PASSWORD, false, false).unwrap();
    assert!(!s.sharing);
    s.lock();
    assert!(s.access().is_err());
}
