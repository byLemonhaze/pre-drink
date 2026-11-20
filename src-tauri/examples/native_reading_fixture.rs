//! Fictional long-chat and curation-library acceptance, in separate encrypted storage.
use pre_drink::{
    briefs::BriefInput,
    memory::MemoryInput,
    model::{ImportReport, Message, ParsedConversation},
    session::Session,
};
use serde_json::json;
use std::{fs, os::unix::fs::PermissionsExt};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::Builder::new()
        .prefix("pd-native-reading-")
        .tempdir_in("/tmp")?
        .keep();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700))?;
    let live = root.join("live");
    fs::create_dir(&live)?;
    let mut session = Session::new(live.clone());
    const PASSWORD: &str = "fictional reading garden phrase";
    session.unlock(PASSWORD, true, false)?;
    let v = session.access()?;
    let id = "fictional-long-garden";
    let messages = (0..2000)
        .map(|i| Message {
            id: format!("{id}-{i}"),
            conversation_id: id.into(),
            role: if i % 2 == 0 { "user" } else { "assistant" }.into(),
            content: if i == 0 {
                format!(
                    "# Fictional long garden plan\n\n{}\nlongtailorchid",
                    "Choose cedar planters. **Keep the walkway clear.**\n\n".repeat(400)
                )
            } else if i == 1999 {
                "Fictional alternate ending: branchorchid.".into()
            } else {
                format!("Fictional garden entry {i:04}. Grow native plants. readingmarker{i:04}.")
            },
            timestamp: None,
            branch_id: Some(if i == 1999 { "alternate" } else { "main" }.into()),
            parent_id: None,
            is_active: i != 1999,
            sensitive: false,
            position: i,
        })
        .collect();
    v.import(
        &[ParsedConversation {
            platform: "chatgpt".into(),
            id: id.into(),
            title: "Fictional long garden conversation".into(),
            created_at: None,
            updated_at: None,
            sensitive: false,
            raw_json: json!({"fictional":true}).to_string(),
            messages,
        }],
        ImportReport::default(),
    )?;
    for i in 0..3000 {
        let memory = v.save_memory(MemoryInput {
            id: None,
            kind: ["fact", "decision", "project"][i % 3].into(),
            content: format!(
                "Fictional garden memory {i:04}. Use cedar planters. notemarker{i:04}."
            ),
            source_message_id: Some(format!("{id}-1")),
            private: i == 2999,
        })?;
        if i < 100 {
            let review = v.approval_review("memory", &memory)?;
            v.approve_review("memory", &memory, &review.review_hash)?;
        }
    }
    for i in 0..500 {
        v.save_brief(BriefInput {
            id: None,
            title: format!("Fictional garden brief {i:04}"),
            content: format!("Use cedar planters and native plants. briefmarker{i:04}."),
            private: i == 499,
            source_ids: vec![],
        })?;
    }
    v.verify()?;
    assert_eq!(v.stats()?.messages, 2000);
    assert_eq!(v.memories(None, "", false)?.len(), 3000);
    assert_eq!(v.briefs(false, "")?.len(), 500);
    session.lock();
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "fixture_root":root, "app_data_dir":live, "fictional_password":PASSWORD,
            "messages":2000, "memories":3000, "briefs":500,
            "late_message":"readingmarker1998", "alternate_message":"branchorchid",
            "long_message_tail":"longtailorchid", "old_memory":"notemarker0000",
            "private_memory":"notemarker2999", "old_brief":"briefmarker0000"
        }))?
    );
    Ok(())
}
