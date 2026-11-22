//! Large, fully prepared fictional input for observing the native saving phase.
//! No artificial delays or fault controls are added to the distributed app.
use pre_drink::{crypto, import::parse_conversation, model::ImportReport, session::Session};
use serde_json::json;
use std::{fs, os::unix::fs::PermissionsExt};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const PASSWORD: &str = "fictional saving garden phrase";
    const CONVERSATIONS: usize = 10_000;
    const MESSAGES: usize = 20;
    let root = tempfile::Builder::new()
        .prefix("pd-native-saving-")
        .tempdir_in("/tmp")?;
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700))?;
    let live = root.path().join("live");
    fs::create_dir(&live)?;
    fs::set_permissions(&live, fs::Permissions::from_mode(0o700))?;
    let mut session = Session::new(live.clone());
    session.unlock(PASSWORD, true, false)?;
    let path = session.path()?;
    session.lock();
    let mut connection = rusqlite::Connection::open(&path)?;
    crypto::configure(&connection, &crypto::key(PASSWORD, &crypto::salt(&path)?)?)?;
    let tx = connection.transaction()?;
    tx.execute(
        "INSERT INTO import_job(id,fingerprint,label,total,saved,messages,skipped) VALUES(1,'fictional-saving','fictional-saving.zip',?1,?1,?2,0)",
        rusqlite::params![CONVERSATIONS as i64, (CONVERSATIONS * MESSAGES) as i64],
    )?;
    let mut report = ImportReport::default();
    {
        let mut insert = tx.prepare(
            "INSERT INTO import_items(position,conversation_id,payload) VALUES(?1,?2,?3)",
        )?;
        for i in 0..CONVERSATIONS {
            let mut mapping = serde_json::Map::new();
            for j in 0..MESSAGES {
                mapping.insert(format!("node-{j}"), json!({
                    "parent":if j == 0 {None} else {Some(format!("node-{}", j-1))},
                    "message":{"author":{"role":"user"},"content":{"parts":[
                        format!("Fictional orchard {i}, step {j}. savingorchid. Choose cedar planters and a clear garden path.")
                    ]}}
                }));
            }
            let conversation = parse_conversation(
                &json!({
                    "id":format!("fictional-saving-{i}"),"title":format!("Fictional saving orchard {i}"),
                    "current_node":format!("node-{}",MESSAGES-1),"mapping":mapping
                }),
                &mut report,
            )?;
            assert_eq!(conversation.messages.len(), MESSAGES);
            insert.execute(rusqlite::params![
                i as i64,
                conversation.id,
                serde_json::to_string(&conversation)?
            ])?;
        }
    }
    tx.commit()?;
    connection.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
    drop(connection);
    session.unlock(PASSWORD, false, false)?;
    assert_eq!(session.access()?.stats()?.conversations, 0);
    let checkpoint = session
        .import_state()?
        .checkpoint
        .ok_or("Missing checkpoint")?;
    assert_eq!(
        (checkpoint.saved, checkpoint.total),
        (CONVERSATIONS, CONVERSATIONS)
    );
    session.access()?.verify()?;
    session.lock();
    let root = root.keep();
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "fixture_root":root,"app_data_dir":live,"fictional_password":PASSWORD,
            "conversations":CONVERSATIONS,"messages":CONVERSATIONS*MESSAGES
        }))?
    );
    Ok(())
}
