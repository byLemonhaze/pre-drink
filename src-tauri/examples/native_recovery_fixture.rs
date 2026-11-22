//! Disposable native recovery inputs. Never opens existing app storage or exports.
use pre_drink::{
    import::parse_conversation, memory::MemoryInput, model::ImportReport, session::Session,
};
use serde_json::json;
use std::{fs, io::Write, os::unix::fs::PermissionsExt, path::Path};

fn write_export(path: &Path, json: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut zip = zip::ZipWriter::new(fs::File::create_new(path)?);
    zip.start_file(
        "conversations.json",
        zip::write::SimpleFileOptions::default(),
    )?;
    zip.write_all(json.as_bytes())?;
    zip.finish()?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const PASSWORD: &str = "fictional recovery garden phrase";
    const BACKUP: &str = "fictional recovery backup phrase";
    let root = tempfile::Builder::new()
        .prefix("pd-native-recovery-")
        .tempdir_in("/tmp")?
        .keep();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700))?;
    let live = root.join("live");
    fs::create_dir(&live)?;
    fs::set_permissions(&live, fs::Permissions::from_mode(0o700))?;
    let mut session = Session::new(live.clone());
    session.unlock(PASSWORD, true, false)?;
    let v = session.access()?;
    let mut report = ImportReport::default();
    let c = parse_conversation(
        &json!({"id":"fictional-recovery-garden","title":"Fictional recovery garden","current_node":"reply","mapping":{
            "question":{"parent":null,"message":{"author":{"role":"user"},"content":{"parts":["Choose cedar planters for the fictional garden."]}}},
            "reply":{"parent":"question","message":{"author":{"role":"assistant"},"content":{"parts":["Keep a clear walkway. recoveryorchid"]}}}
        }}),
        &mut report,
    )?;
    v.import(&[c], report)?;
    let note = v.save_memory(MemoryInput {
        id: None,
        kind: "project".into(),
        content: "Fictional backup baseline: cedar planters.".into(),
        source_message_id: None,
        private: false,
    })?;
    let review = v.approval_review("memory", &note)?;
    v.approve_review("memory", &note, &review.review_hash)?;
    let backup = root.join("verified-backup.sqlite3");
    v.encrypted_backup(&backup, BACKUP)?;
    let mut damaged = fs::read(&backup)?;
    damaged.truncate(damaged.len() - 101);
    fs::write(root.join("damaged-backup.sqlite3"), damaged)?;
    fs::set_permissions(
        root.join("damaged-backup.sqlite3"),
        fs::Permissions::from_mode(0o600),
    )?;
    v.save_memory(MemoryInput {
        id: None,
        kind: "decision".into(),
        content: "Fictional current-only decision: paint the bench cream.".into(),
        source_message_id: None,
        private: false,
    })?;
    v.verify()?;
    v.checkpoint()?;
    session.lock();
    write_export(
        &root.join("malformed-export.zip"),
        "[{broken fictional JSON]",
    )?;
    let export = json!([{"id":"fictional-recovery-import","title":"Fictional imported orchard","current_node":"leaf","mapping":{
        "leaf":{"parent":null,"message":{"author":{"role":"user"},"content":{"parts":["Grow a pear tree. importedorchid"]}}}
    }}]);
    let valid = root.join("valid-export.zip");
    write_export(&valid, &export.to_string())?;
    assert_eq!(pre_drink::import::read_export_zip(&valid)?.0.len(), 1);
    assert!(pre_drink::import::read_export_zip(&root.join("malformed-export.zip")).is_err());
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "fixture_root":root, "app_data_dir":live,
            "fictional_password":PASSWORD, "fictional_backup_password":BACKUP,
            "current_memories":2, "backup_memories":1
        }))?
    );
    Ok(())
}
