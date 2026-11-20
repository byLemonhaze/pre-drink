use pre_drink::{
    briefs::BriefInput,
    db::Vault,
    import::parse_conversation,
    memory::MemoryInput,
    model::ImportReport,
    retention::{RemovalInput, RemovalKind},
    session::Session,
};
use serde_json::json;
fn imported(v: &mut Vault, text: &str) -> (String, String) {
    let c = parse_conversation(&json!({"id":"synthetic","title":"Garden history","current_node":"m","mapping":{"m":{"parent":null,"message":{"author":{"role":"user"},"content":{"parts":[text]}}}}}), &mut ImportReport::default()).unwrap();
    let result = (c.id.clone(), c.messages[0].id.clone());
    v.import(&[c], ImportReport::default()).unwrap();
    result
}
fn session() -> Session {
    let mut s = Session::new(std::path::PathBuf::new());
    s.vault = Some(Vault::memory().unwrap());
    s
}
#[test]
fn conversation_deletion_removes_search_and_evidence_but_keeps_blocked_curated_records() {
    let mut s = session();
    let v = s.access().unwrap();
    let (id, source) = imported(v, "Choose native plants.");
    let note = v
        .save_memory(MemoryInput {
            id: None,
            kind: "project".into(),
            content: "Native planting project.".into(),
            source_message_id: Some(source),
            private: false,
        })
        .unwrap();
    let r = v.approval_review("memory", &note).unwrap();
    v.approve_review("memory", &note, &r.review_hash).unwrap();
    let brief = v
        .save_brief(BriefInput {
            id: None,
            title: "Planting handoff".into(),
            content: "Choose native plants.".into(),
            source_ids: vec![note.clone()],
            private: false,
        })
        .unwrap();
    let r = v.approval_review("brief", &brief).unwrap();
    v.approve_review("brief", &brief, &r.review_hash).unwrap();
    let input = RemovalInput {
        kind: RemovalKind::Conversation,
        conversation_id: Some(id.clone()),
    };
    let p = s.preview_removal(&input).unwrap();
    assert_eq!(
        (p.messages, p.snapshots, p.memories, p.briefs),
        (1, 1, 1, 1)
    );
    s.sharing = true;
    s.apply_removal(&input, &p.review_hash, p.session_generation)
        .unwrap();
    assert!(!s.sharing);
    let v = s.access().unwrap();
    v.verify().unwrap();
    assert_eq!(v.stats().unwrap().conversations, 0);
    assert_eq!(v.stats().unwrap().messages, 0);
    assert!(v.list("native", true, 0).unwrap().is_empty());
    let notes = v.memories(None, "", false).unwrap();
    assert_eq!(notes.len(), 1);
    assert!(!notes[0].approved);
    assert!(notes[0].source_restricted);
    assert!(notes[0].source_message_id.is_none());
    assert!(v.briefs(false, "").unwrap()[0].blocked);
    assert!(v.briefs(true, "").unwrap().is_empty());
    imported(v, "Choose native plants.");
    assert!(
        v.approval_review("memory", &note).unwrap().restricted,
        "Reimport must not silently reattach deleted evidence"
    );
}
#[test]
fn old_snapshot_cleanup_preserves_current_raw_version_and_message_search() {
    let mut s = session();
    imported(s.access().unwrap(), "First planting plan.");
    imported(s.access().unwrap(), "Second planting plan.");
    let input = RemovalInput {
        kind: RemovalKind::Snapshots,
        conversation_id: None,
    };
    let p = s.preview_removal(&input).unwrap();
    assert_eq!(p.snapshots, 1);
    s.apply_removal(&input, &p.review_hash, p.session_generation)
        .unwrap();
    assert_eq!(s.preview_removal(&input).unwrap().snapshots, 0);
    let v = s.access().unwrap();
    assert_eq!(v.list("Second planting", false, 0).unwrap().len(), 1);
    v.verify().unwrap();
    let (id, _) = imported(v, "Second planting plan.");
    let p = s
        .preview_removal(&RemovalInput {
            kind: RemovalKind::Conversation,
            conversation_id: Some(id),
        })
        .unwrap();
    assert_eq!(p.snapshots, 1);
}
#[test]
fn deletion_rejects_stale_preview_and_session_without_mutating_archive() {
    let mut s = session();
    let (id, _) = imported(s.access().unwrap(), "Original planting plan.");
    let input = RemovalInput {
        kind: RemovalKind::Conversation,
        conversation_id: Some(id),
    };
    let p = s.preview_removal(&input).unwrap();
    imported(s.access().unwrap(), "Updated planting plan.");
    assert!(s
        .apply_removal(&input, &p.review_hash, p.session_generation)
        .is_err());
    assert_eq!(s.access().unwrap().stats().unwrap().conversations, 1);
    let p = s.preview_removal(&input).unwrap();
    s.generation += 1;
    assert!(s
        .apply_removal(&input, &p.review_hash, p.session_generation)
        .is_err());
    assert_eq!(s.access().unwrap().stats().unwrap().messages, 1);
}
