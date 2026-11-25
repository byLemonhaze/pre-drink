//! Small fictional design fixture. Always creates fresh encrypted temporary storage.
use pre_drink::{
    briefs::BriefInput, import::parse_conversation, memory::MemoryInput, model::ImportReport,
    session::Session,
};
use serde_json::json;
use std::{fs, os::unix::fs::PermissionsExt};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::Builder::new()
        .prefix("pd-native-cabinet-")
        .tempdir_in("/tmp")?
        .keep();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700))?;
    let live = root.join("live");
    fs::create_dir(&live)?;
    fs::set_permissions(&live, fs::Permissions::from_mode(0o700))?;
    let mut session = Session::new(live.clone());
    session.unlock("fictional cabinet garden phrase", true, false)?;
    let mut report = ImportReport::default();
    let conversation = parse_conversation(
        &json!({"id":"fictional-cabinet-garden","title":"Fictional garden · design notes","current_node":"garden","mapping":{"garden":{"parent":null,"message":{"author":{"role":"user"},"content":{"parts":["# A small garden, made for everyday life\n\nUse native plants and cedar planters. Keep the central walkway clear.\n\n## First season\n- Plant in spring\n- Observe sunlight\n- Record what thrives"]}}}}}),
        &mut report,
    )?;
    let source = conversation.messages[0].id.clone();
    let vault = session.access()?;
    vault.import(&[conversation], report)?;
    let mut approved = vec![];
    for (i, (kind, content)) in [
        ("decision", "Use native plants in the courtyard. They suit the climate and leave more time to enjoy the garden."),
        ("project", "Courtyard garden — create a quiet place to read, with cedar planters and a clear central walkway. Begin with one small bed this spring."),
        ("fact", "The fictional courtyard gets morning light and afternoon shade. The eastern bed is the best starting point."),
        ("decision", "Keep a simple seasonal notebook. Record what thrives before buying more plants."),
        ("fact", "The fictional garden bench is made from reclaimed cedar."),
        ("project", "Plan a late-summer review. Compare photographs, note which plants established well, and choose the next small improvement."),
        ("fact", "This fictional private note is withheld from sharing."),
    ].into_iter().enumerate() {
        let id = vault.save_memory(MemoryInput {
            id: None, kind: kind.into(), content: content.into(),
            source_message_id: Some(source.clone()), private: i == 6,
        })?;
        if i % 2 == 0 && i != 6 {
            let review = vault.approval_review("memory", &id)?;
            vault.approve_review("memory", &id, &review.review_hash)?;
            approved.push(id);
        }
    }
    vault.save_brief(BriefInput {
        id: None, title: "Fictional courtyard · first season".into(),
        content: "A calm place to read, built one small bed at a time.\n\nUse native plants and cedar planters. Observe the morning light, keep the path clear, and record what thrives before expanding.".into(),
        private: false, source_ids: approved,
    })?;
    vault.verify()?;
    vault.checkpoint()?;
    session.lock();
    println!("{}", json!({"fixture_root":root,"app_data_dir":live}));
    Ok(())
}
