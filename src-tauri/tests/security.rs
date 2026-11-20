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
fn memory(v: &mut Vault, approved: bool, source: Option<String>) -> String {
    let id = v
        .save_memory(MemoryInput {
            id: None,
            kind: "decision".into(),
            content: "Pre-Drink supports ChatGPT only.".into(),
            source_message_id: source,
            private: false,
        })
        .unwrap();
    if approved {
        approve(v, "memory", &id);
    }
    id
}
fn approve(v: &mut Vault, kind: &str, id: &str) {
    let review = v.approval_review(kind, id).unwrap();
    v.approve_review(kind, id, &review.review_hash).unwrap();
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
        curation: None,
    }
}

#[test]
fn encrypted_database_and_journal_do_not_contain_plaintext() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("vault.enc.sqlite3");
    let mut v = Vault::create(&path, PASSWORD).unwrap();
    sample(&mut v);
    memory(&mut v, true, None);
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
    memory(&mut v, true, None);
    v.encrypted_backup(&path, PASSWORD).unwrap();
    assert!(Vault::open(&path, "wrong password").is_err());
    let restored = Vault::open(&path, PASSWORD).unwrap();
    assert_eq!(restored.stats().unwrap().messages, 1);
    assert_eq!(restored.memories(None, "", false).unwrap().len(), 1);
    assert_eq!(restored.memories(None, "", true).unwrap().len(), 1);
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
        + "\nPRAGMA user_version = 1;";
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
    let mut v = Vault::memory().unwrap();
    memory(&mut v, false, None);
    let id = memory(&mut v, true, None);
    assert_eq!(v.memories(None, "", true).unwrap().len(), 1);
    v.save_memory(MemoryInput {
        id: Some(id.clone()),
        kind: "fact".into(),
        content: "Pre-Drink is private.".into(),
        source_message_id: None,
        private: true,
    })
    .unwrap();
    assert!(v.memories(None, "", true).unwrap().is_empty());
    v.save_memory(MemoryInput {
        id: Some(id),
        kind: "fact".into(),
        content: "Pre-Drink email me@example.org".into(),
        source_message_id: None,
        private: false,
    })
    .unwrap();
    assert!(v.memories(None, "", true).unwrap().is_empty());
}

#[test]
fn source_privacy_propagates_through_memory_and_briefs() {
    let mut v = Vault::memory().unwrap();
    let (conversation, source) = sample(&mut v);
    let note = memory(&mut v, true, Some(source));
    let brief = v
        .save_brief(BriefInput {
            id: None,
            title: "Pre-Drink spec".into(),
            content: "Pre-Drink supports ChatGPT only.".into(),
            private: false,
            source_ids: vec![note.clone()],
        })
        .unwrap();
    approve(&mut v, "brief", &brief);
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
        private: false,
    })
    .unwrap();
    assert!(v.memories(None, "", true).unwrap().is_empty());
}

#[test]
fn briefs_need_review_after_source_changes_and_fail_closed_on_deletion() {
    let mut v = Vault::memory().unwrap();
    let note = memory(&mut v, true, None);
    let id = v
        .save_brief(BriefInput {
            id: None,
            title: "Spec".into(),
            content: "Pre-Drink spec.".into(),
            private: false,
            source_ids: vec![note.clone()],
        })
        .unwrap();
    approve(&mut v, "brief", &id);
    v.save_memory(MemoryInput {
        id: Some(note.clone()),
        kind: "decision".into(),
        content: "Pre-Drink is a Mac app.".into(),
        source_message_id: None,
        private: false,
    })
    .unwrap();
    assert!(v.briefs(true, "").unwrap().is_empty());
    approve(&mut v, "memory", &note);
    v.save_brief(BriefInput {
        id: Some(id.clone()),
        title: "Spec".into(),
        content: "Pre-Drink updated spec.".into(),
        private: false,
        source_ids: vec![note.clone()],
    })
    .unwrap();
    approve(&mut v, "brief", &id);
    assert_eq!(v.briefs(true, "").unwrap().len(), 1);
    v.delete_memory(&note).unwrap();
    assert!(v.briefs(true, "").unwrap().is_empty());
}

#[test]
fn clients_need_valid_tokens_and_scopes_and_can_be_revoked() {
    let mut v = Vault::memory().unwrap();
    sample(&mut v);
    memory(&mut v, true, None);
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

#[test]
fn memory_only_agents_receive_no_archive_metadata() {
    let mut v = Vault::memory().unwrap();
    let (_, source) = sample(&mut v);
    memory(&mut v, true, Some(source.clone()));
    let c = v.create_client("Memory only", false).unwrap();
    let result = v
        .agent_call(&request(&c.client.id, &c.token, "get_decisions"))
        .unwrap();
    let object = result[0].as_object().unwrap();
    assert_eq!(object.len(), 4);
    for key in ["id", "kind", "content", "updated_at"] {
        assert!(object.contains_key(key));
    }
    let pack = v.context_pack("Pre-Drink", false).unwrap();
    assert!(!pack.contains("Source message:"));
    assert!(!pack.contains(&source));
    assert!(!result.to_string().contains("Pre-Drink planning"));
}

#[test]
fn saves_are_drafts_and_approval_is_bound_to_reviewed_version() {
    let mut v = Vault::memory().unwrap();
    let id = memory(&mut v, false, None);
    let review = v.approval_review("memory", &id).unwrap();
    approve(&mut v, "memory", &id);
    v.save_memory(MemoryInput {
        id: Some(id.clone()),
        kind: "fact".into(),
        content: "A different saved note.".into(),
        source_message_id: None,
        private: false,
    })
    .unwrap();
    assert!(v.memories(None, "", true).unwrap().is_empty());
    assert!(v
        .approve_review("memory", &id, &review.review_hash)
        .is_err());
    approve(&mut v, "memory", &id);
    assert_eq!(v.memories(None, "", true).unwrap().len(), 1);
    // Old or forged command payloads cannot turn saving into approval.
    assert!(serde_json::from_value::<MemoryInput>(json!({"id":id,"kind":"fact","content":"Changed","source_message_id":null,"private":false,"approved":true})).is_err());
    assert!(serde_json::from_value::<BriefInput>(json!({"id":null,"title":"Changed","content":"Changed","source_ids":[],"private":false,"approved":true})).is_err());
}

#[test]
fn brief_review_rejects_changed_sources_and_changed_text() {
    let mut v = Vault::memory().unwrap();
    let note = memory(&mut v, true, None);
    let id = v
        .save_brief(BriefInput {
            id: None,
            title: "Spec".into(),
            content: "Pre-Drink spec.".into(),
            private: false,
            source_ids: vec![note.clone()],
        })
        .unwrap();
    let original = v.approval_review("brief", &id).unwrap();
    assert_eq!(original.sources.len(), 1);
    approve(&mut v, "brief", &id);
    v.save_memory(MemoryInput {
        id: Some(note.clone()),
        kind: "fact".into(),
        content: "A changed source.".into(),
        source_message_id: None,
        private: false,
    })
    .unwrap();
    approve(&mut v, "memory", &note);
    assert!(v
        .approve_review("brief", &id, &original.review_hash)
        .is_err());
    let fresh = v.approval_review("brief", &id).unwrap();
    v.save_brief(BriefInput {
        id: Some(id.clone()),
        title: "New spec".into(),
        content: "Revised brief.".into(),
        private: false,
        source_ids: vec![note],
    })
    .unwrap();
    assert!(v.briefs(true, "").unwrap().is_empty());
    assert!(v.approve_review("brief", &id, &fresh.review_hash).is_err());
    approve(&mut v, "brief", &id);
    assert_eq!(v.briefs(true, "").unwrap().len(), 1);
}

fn synthetic_secrets() -> Vec<String> {
    vec![
        format!("sk-{}", "aB".repeat(24)),
        format!("sk-proj-{}", "qZ_".repeat(16)),
        format!("ghp_{}", "aB".repeat(20)),
        format!("github_pat_{}", "aBc_".repeat(20)),
        format!("AKIA{}", "Z".repeat(16)),
        ["-----BEGIN ", "PRIVATE KEY-----"].concat(),
        format!(
            "eyJ{}.{}.{}",
            "Abc".repeat(8),
            "Def".repeat(8),
            "Ghi".repeat(8)
        ),
        format!("xoxb-{}", "abc-".repeat(10)),
        "aB3dE5gH7jK9mN2pQ4sT6vW8yZ0cF1iL".into(),
    ]
}

#[test]
fn credentials_are_blocked_across_all_sharing_layers() {
    use pre_drink::privacy::contains_secret;
    let mut v = Vault::memory().unwrap();
    let client = v.create_client("Chat enabled", true).unwrap();
    for (index, secret) in synthetic_secrets().into_iter().enumerate() {
        assert!(
            contains_secret(&secret),
            "synthetic pattern {index} was missed"
        );
        let note = v
            .save_memory(MemoryInput {
                id: None,
                kind: "fact".into(),
                content: format!("Pre-Drink {secret}"),
                source_message_id: None,
                private: false,
            })
            .unwrap();
        let review = v.approval_review("memory", &note).unwrap();
        assert!(review.restricted);
        assert!(v
            .approve_review("memory", &note, &review.review_hash)
            .is_err());
        let brief = v
            .save_brief(BriefInput {
                id: None,
                title: "Pre-Drink".into(),
                content: secret.clone(),
                private: false,
                source_ids: vec![],
            })
            .unwrap();
        let review = v.approval_review("brief", &brief).unwrap();
        assert!(v
            .approve_review("brief", &brief, &review.review_hash)
            .is_err());
        let chat = parse_conversation(&json!({"id":format!("secret-{index}"),"title":"Pre-Drink","mapping":{"m":{"message":{"author":{"role":"user"},"content":{"parts":[secret]}}}}}), &mut ImportReport::default()).unwrap();
        v.import(&[chat], ImportReport::default()).unwrap();
    }
    assert!(v.list("Pre-Drink", false, 0).unwrap().is_empty());
    assert!(v
        .agent_call(&request(&client.client.id, &client.token, "get_facts"))
        .unwrap()
        .as_array()
        .unwrap()
        .is_empty());
    assert!(v.briefs(true, "").unwrap().is_empty());
    assert!(!contains_secret(
        "A normal project description with a UUID 134c4edf-8321-4496-b80d-886ddab94cd3."
    ));
}

#[test]
fn old_vaults_are_rescanned_before_sharing_and_approvals_survive_later_reopens() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("vault");
    let mut v = Vault::create(&path, PASSWORD).unwrap();
    let clean = memory(&mut v, true, None);
    let secret = synthetic_secrets().remove(0);
    let note = v
        .save_memory(MemoryInput {
            id: None,
            kind: "fact".into(),
            content: secret,
            source_message_id: None,
            private: false,
        })
        .unwrap();
    drop(v);
    // Simulate a v2 database whose old detector missed a credential.
    let c = rusqlite::Connection::open(&path).unwrap();
    let key = pre_drink::crypto::key(PASSWORD, &pre_drink::crypto::salt(&path).unwrap()).unwrap();
    pre_drink::crypto::configure(&c, &key).unwrap();
    c.execute_batch("UPDATE memory SET sensitive=0,approved=1; PRAGMA user_version=2;")
        .unwrap();
    drop(c);
    let mut v = Vault::open(&path, PASSWORD).unwrap();
    let review = v.approval_review("memory", &note).unwrap();
    assert!(review.restricted);
    assert!(v.memories(None, "", true).unwrap().is_empty());
    approve(&mut v, "memory", &clean);
    drop(v);
    let v = Vault::open(&path, PASSWORD).unwrap();
    assert_eq!(v.memories(None, "", true).unwrap().len(), 1);
}

#[test]
fn privacy_recheck_disables_sharing_even_if_the_preview_is_stale() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Session::new(dir.path().into());
    s.vault = Some(Vault::memory().unwrap());
    s.sharing = true;
    assert!(s
        .apply_privacy_recheck("stale-synthetic-preview", &[])
        .is_err());
    assert!(!s.sharing);
    let preview = s.access().unwrap().privacy_recheck_preview().unwrap();
    s.sharing = true;
    s.apply_privacy_recheck(&preview.review_hash, &[]).unwrap();
    assert!(!s.sharing);
}

#[test]
fn build_and_storage_identity_are_explicit_without_exposing_passwords() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Session::new(dir.path().into());
    let status = s.status().unwrap();
    assert_eq!(status.app_version, env!("CARGO_PKG_VERSION"));
    assert!(!status.build_revision.is_empty());
    assert!(status.built_at.parse::<u64>().is_ok());
    assert!(!status.isolated_storage);
    s.isolated_storage = true;
    assert!(s.status().unwrap().isolated_storage);
}
