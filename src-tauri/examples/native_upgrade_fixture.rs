//! Fictional native upgrade/import inputs; never opens existing owner storage.
use pre_drink::{
    briefs::BriefInput, crypto, import::parse_conversation, memory::MemoryInput,
    model::ImportReport, session::Session,
};
use rusqlite::Connection;
use serde_json::json;
use std::{fs, io::Write, os::unix::fs::PermissionsExt};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const PASSWORD: &str = "fictional upgrade garden phrase";
    let root = tempfile::Builder::new()
        .prefix("pd-native-upgrade-")
        .tempdir_in("/tmp")?
        .keep();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700))?;
    let live = root.join("live");
    fs::create_dir(&live)?;
    fs::set_permissions(&live, fs::Permissions::from_mode(0o700))?;
    let mut session = Session::new(live.clone());
    session.unlock(PASSWORD, true, false)?;
    let mut report = ImportReport::default();
    let conversation = parse_conversation(
        &json!({"id":"fictional-upgrade","title":"Fictional upgrade garden","current_node":"leaf","mapping":{
            "leaf":{"parent":null,"message":{"author":{"role":"user"},"content":{"parts":["Use cedar planters in the fictional garden."]}}}
        }}),
        &mut report,
    )?;
    let message = conversation.messages[0].id.clone();
    let vault = session.access()?;
    vault.import(&[conversation], report)?;
    let mut notes = Vec::new();
    for (content, source) in [
        ("Fictional linked cedar project.", Some(message.clone())),
        ("Fictional unrelated orchard project.", None),
    ] {
        let id = vault.save_memory(MemoryInput {
            id: None,
            kind: "project".into(),
            content: content.into(),
            source_message_id: source,
            private: false,
        })?;
        let review = vault.approval_review("memory", &id)?;
        vault.approve_review("memory", &id, &review.review_hash)?;
        notes.push(id);
    }
    let brief = vault.save_brief(BriefInput {
        id: None,
        title: "Fictional linked garden brief".into(),
        content: "Use cedar planters.".into(),
        private: false,
        source_ids: vec![notes[0].clone()],
    })?;
    let review = vault.approval_review("brief", &brief)?;
    vault.approve_review("brief", &brief, &review.review_hash)?;
    vault.checkpoint()?;
    let active = session.path()?;
    session.lock();
    // Emulate an older classifier that did not recognize this secret shape.
    // The native app, not this generator, must perform reclassification on open.
    let connection = Connection::open(&active)?;
    let key = crypto::key(PASSWORD, &crypto::salt(&active)?)?;
    crypto::configure(&connection, &key)?;
    connection.execute(
        "UPDATE messages SET content='my wifi password is fictional-lilac',sensitive=0 WHERE id=?1",
        [&message],
    )?;
    // The current evidence trigger correctly invalidates this synthetic edit.
    // Restore the historical approvals to model what the older app had saved.
    connection.execute("UPDATE memory SET approved=1 WHERE id=?1", [&notes[0]])?;
    connection.execute_batch(
        "UPDATE brief_sources SET reviewed_at=(SELECT updated_at FROM memory WHERE id=memory_id);",
    )?;
    let approved: i64 = connection.query_row(
        "SELECT (SELECT count(*) FROM memory WHERE approved=1)+(SELECT count(*) FROM briefs WHERE approved=1)",
        [], |row| row.get(0),
    )?;
    assert_eq!(approved, 3);
    connection.execute_batch(
        "DROP TABLE curation_owner_holds; PRAGMA user_version=6; PRAGMA wal_checkpoint(TRUNCATE);",
    )?;
    drop(connection);

    let future = root.join("future");
    fs::create_dir(&future)?;
    fs::set_permissions(&future, fs::Permissions::from_mode(0o700))?;
    let future_path = Session::new(future.clone()).path()?;
    fs::copy(&active, &future_path)?;
    fs::set_permissions(&future_path, fs::Permissions::from_mode(0o600))?;
    let connection = Connection::open(&future_path)?;
    crypto::configure(&connection, &key)?;
    connection.execute_batch("PRAGMA user_version=999; PRAGMA wal_checkpoint(TRUNCATE);")?;
    drop(connection);

    // A fully prepared encrypted input for the native no-ZIP Finish path.
    let ready = root.join("ready");
    fs::create_dir(&ready)?;
    fs::set_permissions(&ready, fs::Permissions::from_mode(0o700))?;
    let mut ready_session = Session::new(ready.clone());
    ready_session.unlock(PASSWORD, true, false)?;
    let ready_path = ready_session.path()?;
    ready_session.lock();
    let c = parse_conversation(
        &json!({"id":"fictional-ready","title":"Fictional prepared garden","current_node":"leaf","mapping":{
            "leaf":{"parent":null,"message":{"author":{"role":"user"},"content":{"parts":["preparedorchid: choose cedar planters"]}}}
        }}),
        &mut ImportReport::default(),
    )?;
    let connection = Connection::open(&ready_path)?;
    let ready_key = crypto::key(PASSWORD, &crypto::salt(&ready_path)?)?;
    crypto::configure(&connection, &ready_key)?;
    connection.execute("INSERT INTO import_job(id,fingerprint,label,total,saved,messages,skipped) VALUES(1,'fictional-ready','fictional-ready.zip',1,1,1,0)",[])?;
    connection.execute(
        "INSERT INTO import_items(position,conversation_id,payload) VALUES(0,?1,?2)",
        rusqlite::params![c.id, serde_json::to_string(&c)?],
    )?;
    connection.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
    drop(connection);

    let path = root.join("long-fictional-export.zip");
    let mut zip = zip::ZipWriter::new(fs::File::create_new(&path)?);
    zip.start_file(
        "conversations.json",
        zip::write::SimpleFileOptions::default(),
    )?;
    zip.write_all(b"[")?;
    for i in 0..5_000 {
        if i != 0 {
            zip.write_all(b",")?;
        }
        let mut mapping = serde_json::Map::new();
        for j in 0..20 {
            mapping.insert(format!("node-{j}"), json!({
                "parent":if j==0 {None} else {Some(format!("node-{}",j-1))},
                "message":{"author":{"role":"user"},"content":{"parts":[
                    format!("Fictional orchard {i}, row {j}. interruptionorchid. {}", "Cedar planters and clear garden paths. ".repeat(8))
                ]}}
            }));
        }
        serde_json::to_writer(
            &mut zip,
            &json!({"id":format!("fictional-long-{i}"),"title":format!("Fictional orchard {i}"),"current_node":"node-19","mapping":mapping}),
        )?;
    }
    zip.write_all(b"]")?;
    zip.finish()?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "fixture_root":root,"app_data_dir":live,"future_app_data_dir":future,"ready_app_data_dir":ready,
            "fictional_password":PASSWORD,
            "expected_restrictions":2,"expected_approvals_reset":2,
            "expected_unrelated_approved":1,"long_import_conversations":5000,
            "long_import_messages":100000
        }))?
    );
    Ok(())
}
