//! Fictional owner-UI fixture. Never reads the default app directory or personal exports.
use pre_drink::{
    briefs::BriefInput, crypto, db::Vault, import::parse_conversation, memory::MemoryInput,
    model::ImportReport, session::Session,
};
use rusqlite::Connection;
use serde_json::json;
use std::{fs, os::unix::fs::PermissionsExt};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const OLD: &str = "fictional previous garden phrase";
    const CURRENT: &str = "fictional current garden phrase";
    let root = tempfile::Builder::new()
        .prefix("pd-native-walkthrough-")
        .tempdir_in("/tmp")?
        .keep();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700))?;
    let live = root.join("live");
    fs::create_dir(&live)?;
    let mut session = Session::new(live.clone());
    session.unlock(OLD, true, false)?;
    let mut report = ImportReport::default();
    let timestamp = parse_conversation(
        &json!({"id":"fictional-clock","title":"Fictional garden timestamp","current_node":"clock","mapping":{"clock":{"parent":null,"message":{"author":{"role":"user"},"content":{"parts":["Garden record created_at: 1795132800"]}}}}}),
        &mut report,
    )?;
    let timestamp_id = timestamp.id.clone();
    let message_id = timestamp.messages[0].id.clone();
    let sensitive = parse_conversation(
        &json!({"id":"fictional-sensitive","title":"Fictional restricted garden message","current_node":"restricted","mapping":{"restricted":{"parent":null,"message":{"author":{"role":"user"},"content":{"parts":["my wifi password is fictional-lilac"]}}}}}),
        &mut report,
    )?;
    let v = session.access()?;
    v.import(&[timestamp, sensitive], report)?;
    let note = v.save_memory(MemoryInput {
        id: None,
        kind: "fact".into(),
        content: "The fictional garden uses cedar planters.".into(),
        source_message_id: Some(message_id.clone()),
        private: false,
    })?;
    let p = v.approval_review("memory", &note)?;
    v.approve_review("memory", &note, &p.review_hash)?;
    let brief = v.save_brief(BriefInput {
        id: None,
        title: "Fictional garden brief".into(),
        content: "Use cedar planters for the fictional garden.".into(),
        private: false,
        source_ids: vec![note.clone()],
    })?;
    let p = v.approval_review("brief", &brief)?;
    v.approve_review("brief", &brief, &p.review_hash)?;
    let review = session.prepare_password_change(OLD, CURRENT)?;
    session.apply_replacement(&review.review_id, CURRENT)?;
    let active = session.path()?;
    // Model old false-positive restrictions and historical approvals on the new
    // fixture generation. This is fixture setup, not native UI evidence.
    let connection = Connection::open(&active)?;
    let key = crypto::key(CURRENT, &crypto::salt(&active)?)?;
    crypto::configure(&connection, &key)?;
    connection.execute("UPDATE messages SET sensitive=1 WHERE id=?1", [&message_id])?;
    connection.execute(
        "UPDATE conversations SET sensitive=1 WHERE id=?1",
        [&timestamp_id],
    )?;
    connection.execute("UPDATE memory SET approved=1 WHERE id=?1", [&note])?;
    connection.execute("UPDATE briefs SET approved=1 WHERE id=?1", [&brief])?;
    connection.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
    drop(connection);
    let v = Vault::open(&active, CURRENT)?;
    v.verify()?;
    drop(v);
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"fixture_root":root,"app_data_dir":live,"fictional_password":CURRENT,"previous_copy":"original","expected_clear_candidates":2,"expected_approvals_to_reset":2})
        )?
    );
    Ok(())
}
