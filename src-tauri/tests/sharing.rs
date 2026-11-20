use pre_drink::{
    briefs::BriefInput,
    db::Vault,
    import::parse_conversation,
    memory::MemoryInput,
    model::ImportReport,
    session::{Session, IDLE_TIMEOUT},
    sharing::{save_markdown, ShareItem, ShareKind, ShareRequest, ShareSelection},
};
use serde_json::json;
use std::{cell::Cell, time::SystemTime};

fn note(
    v: &mut Vault,
    content: &str,
    private: bool,
    approve: bool,
    source: Option<String>,
) -> String {
    let id = v
        .save_memory(MemoryInput {
            id: None,
            kind: "fact".into(),
            content: content.into(),
            private,
            source_message_id: source,
        })
        .unwrap();
    if approve {
        approve_item(v, "memory", &id);
    }
    id
}
fn approve_item(v: &mut Vault, kind: &str, id: &str) {
    let review = v.approval_review(kind, id).unwrap();
    v.approve_review(kind, id, &review.review_hash).unwrap();
}
fn selection(ids: &[String]) -> ShareSelection {
    ShareSelection {
        items: ids
            .iter()
            .map(|id| ShareItem {
                kind: ShareKind::Memory,
                id: id.clone(),
            })
            .collect(),
    }
}
fn request(s: &mut Session, selection: ShareSelection) -> ShareRequest {
    let p = s.prepare_share(&selection).unwrap();
    ShareRequest {
        selection,
        review_hash: p.preview.review_hash,
        session_generation: p.session_generation,
    }
}
fn session(v: Vault) -> Session {
    let mut s = Session::new(std::path::PathBuf::new());
    s.vault = Some(v);
    s
}
fn imported(v: &mut Vault, text: &str) -> (String, String) {
    let c = parse_conversation(&json!({"id":"synthetic-story","title":"Unshared archive title","current_node":"m","mapping":{"m":{"parent":null,"message":{"author":{"role":"user"},"content":{"parts":[text]}}}}}), &mut ImportReport::default()).unwrap();
    let ids = (c.id.clone(), c.messages[0].id.clone());
    v.import(&[c], ImportReport::default()).unwrap();
    ids
}

#[test]
fn selected_preview_is_exact_output_without_provenance_or_exclusion_leaks() {
    let mut v = Vault::memory().unwrap();
    let (_, source) = imported(&mut v, "Unshared source body.");
    let id = note(
        &mut v,
        "Gardening café 🌱\nChoose native plants.",
        false,
        true,
        Some(source.clone()),
    );
    let private = note(&mut v, "Hidden owner note", true, false, None);
    let draft = note(&mut v, "Draft owner note", false, false, None);
    let sensitive = note(
        &mut v,
        "Contact fictional@example.invalid",
        false,
        false,
        None,
    );
    let mut s = session(v);
    let chosen = selection(&[id.clone(), private, draft, sensitive, "missing".into()]);
    let p = s.prepare_share(&chosen).unwrap();
    assert_eq!(p.preview.included.len(), 1);
    assert_eq!(p.preview.excluded.len(), 4);
    assert_eq!(p.preview.bytes, p.preview.markdown.len());
    for hidden in [
        &id,
        &source,
        "Hidden owner note",
        "Draft owner note",
        "fictional@example.invalid",
        "Unshared archive title",
        "Unshared source body",
    ] {
        assert!(!p.preview.markdown.contains(hidden));
    }
    assert!(p
        .preview
        .markdown
        .contains("> Gardening café 🌱\n> Choose native plants."));
    let r = ShareRequest {
        selection: chosen,
        review_hash: p.preview.review_hash,
        session_generation: p.session_generation,
    };
    let output = s
        .with_validated_share(&r, |text| Ok(text.to_owned()))
        .unwrap();
    assert_eq!(p.preview.markdown, output);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("context.md");
    s.with_validated_share(&r, |text| save_markdown(&path, text))
        .unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), output.as_bytes());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    assert!(!s.sharing);
}

#[test]
fn stale_edits_privacy_deletion_and_session_changes_never_deliver() {
    for change in ["edit", "private", "delete", "lock", "expire", "new_session"] {
        let mut v = Vault::memory().unwrap();
        let id = note(&mut v, "Choose native plants.", false, true, None);
        let mut s = session(v);
        let r = request(&mut s, selection(std::slice::from_ref(&id)));
        match change {
            "edit" | "private" => {
                s.access()
                    .unwrap()
                    .save_memory(MemoryInput {
                        id: Some(id),
                        kind: "fact".into(),
                        content: "Choose native plants again.".into(),
                        source_message_id: None,
                        private: change == "private",
                    })
                    .unwrap();
            }
            "delete" => s.access().unwrap().delete_memory(&id).unwrap(),
            "lock" => s.lock(),
            "expire" => s.last_activity = SystemTime::now() - IDLE_TIMEOUT,
            "new_session" => s.generation += 1,
            _ => unreachable!(),
        }
        let called = Cell::new(false);
        assert!(
            s.with_validated_share(&r, |_| {
                called.set(true);
                Ok(())
            })
            .is_err(),
            "{change}"
        );
        assert!(!called.get());
    }
}

#[test]
fn changed_imported_evidence_revokes_memory_and_brief_and_stale_export() {
    let mut v = Vault::memory().unwrap();
    let (conversation, source) = imported(&mut v, "Choose native plants.");
    let id = note(&mut v, "Planting plan.", false, true, Some(source));
    let brief = v
        .save_brief(BriefInput {
            id: None,
            title: "Garden handoff".into(),
            content: "Plan native planting.".into(),
            private: false,
            source_ids: vec![id.clone()],
        })
        .unwrap();
    approve_item(&mut v, "brief", &brief);
    let mut s = session(v);
    let chosen = ShareSelection {
        items: vec![ShareItem {
            kind: ShareKind::Brief,
            id: brief.clone(),
        }],
    };
    let r = request(&mut s, chosen.clone());
    imported(s.access().unwrap(), "Choose native plants.");
    assert!(
        s.with_validated_share(&r, |_| Ok(())).is_ok(),
        "Unchanged imports preserve approval"
    );
    imported(s.access().unwrap(), "The planting plan has changed.");
    assert!(s
        .with_validated_share::<()>(&r, |_| panic!("Must not deliver changed evidence"))
        .is_err());
    assert!(s
        .access()
        .unwrap()
        .memories(None, "", true)
        .unwrap()
        .is_empty());
    assert!(s
        .prepare_share(&chosen)
        .unwrap()
        .preview
        .included
        .is_empty());
    approve_item(s.access().unwrap(), "memory", &id);
    assert!(
        s.prepare_share(&chosen)
            .unwrap()
            .preview
            .included
            .is_empty(),
        "Brief still needs source review"
    );
    approve_item(s.access().unwrap(), "brief", &brief);
    let r = request(&mut s, chosen);
    s.access()
        .unwrap()
        .mark_private(&conversation, true)
        .unwrap();
    assert!(s
        .with_validated_share::<()>(&r, |_| panic!("Must not deliver private evidence"))
        .is_err());
}

#[test]
fn rejects_empty_duplicate_oversized_selections_and_zero_eligible_delivery() {
    let mut v = Vault::memory().unwrap();
    let id = note(&mut v, "A draft note.", false, false, None);
    assert!(v.share_preview(&selection(&[])).is_err());
    assert!(v
        .share_preview(&selection(&[id.clone(), id.clone()]))
        .is_err());
    assert!(v.share_preview(&selection(&vec![id.clone(); 51])).is_err());
    let mut s = session(v);
    let r = request(&mut s, selection(&[id]));
    assert!(s
        .with_validated_share::<()>(&r, |_| panic!("No eligible output"))
        .is_err());
    let mut ids = vec![];
    for _ in 0..9 {
        ids.push(note(
            s.access().unwrap(),
            &"garden ".repeat(2200),
            false,
            true,
            None,
        ));
    }
    assert!(
        s.prepare_share(&selection(&ids)).is_err(),
        "Oversize selection must not truncate silently"
    );
}

#[test]
fn export_refuses_overwrite_symlink_and_invalid_destinations_and_propagates_delivery_errors() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("existing.md");
    std::fs::write(&path, "Keep existing text").unwrap();
    assert!(save_markdown(&path, "Replacement").is_err());
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "Keep existing text"
    );
    assert!(save_markdown(&dir.path().join("wrong.txt"), "text").is_err());
    assert!(save_markdown(&dir.path().join("absent/output.md"), "text").is_err());
    #[cfg(unix)]
    {
        let link = dir.path().join("link.md");
        std::os::unix::fs::symlink(&path, &link).unwrap();
        assert!(save_markdown(&link, "Replacement").is_err());
        assert_eq!(std::fs::read_to_string(path).unwrap(), "Keep existing text");
    }
    let mut v = Vault::memory().unwrap();
    let id = note(&mut v, "Garden plan.", false, true, None);
    let mut s = session(v);
    let r = request(&mut s, selection(&[id]));
    let error = s.with_validated_share::<()>(&r, |_| Err("Synthetic clipboard unavailable".into()));
    assert_eq!(error.unwrap_err(), "Synthetic clipboard unavailable");
}
