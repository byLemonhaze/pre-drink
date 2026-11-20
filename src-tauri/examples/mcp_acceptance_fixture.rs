//! Developer-only, fictional installed-client acceptance harness. Never opens user storage.
use pre_drink::{
    briefs::BriefInput, broker, db::Vault, import::parse_conversation, memory::MemoryInput,
    model::ImportReport, session::Session,
};
use serde_json::json;
use std::{
    fs::{DirBuilder, OpenOptions},
    io::{BufRead, Write},
    os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt},
    path::PathBuf,
    sync::{Arc, Mutex},
};
use zeroize::{Zeroize, Zeroizing};

fn note(content: &str, private: bool) -> MemoryInput {
    MemoryInput {
        id: None,
        kind: "fact".into(),
        content: content.into(),
        source_message_id: None,
        private,
    }
}

fn approve(vault: &mut Vault, kind: &str, id: &str) -> Result<(), String> {
    let review = vault.approval_review(kind, id)?;
    vault.approve_review(kind, id, &review.review_hash)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let executable = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or("Pass the MCP binary path")?,
    )
    .canonicalize()?;
    if !executable.is_file() {
        return Err("MCP binary is not a file".into());
    }
    // No caller-supplied vault directory, database, password or import is accepted.
    let directory = tempfile::Builder::new()
        .prefix("pd-client-")
        .tempdir_in("/tmp")?;
    let work = directory.path().join("work");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))?;
    DirBuilder::new().mode(0o700).create(&work)?;
    let mut random = [0u8; 32];
    getrandom::fill(&mut random).map_err(|_| "Randomness unavailable")?;
    let password = Zeroizing::new(hex::encode(random));
    random.zeroize();
    let mut session = Session::new(directory.path().into());
    session.isolated_storage = true;
    session.unlock(&password, true, false)?;
    let vault = session.access()?;
    let approved = vault.save_memory(note("garden-approved: fictional cedar planters", false))?;
    approve(vault, "memory", &approved)?;
    vault.save_memory(note("garden-draft: fictional unreviewed note", false))?;
    vault.save_memory(note("garden-private: fictional owner-only note", true))?;
    vault.save_memory(note(
        "garden-sensitive: fictional person@example.org",
        false,
    ))?;
    let brief = vault.save_brief(BriefInput {
        id: None,
        title: "Fictional garden brief".into(),
        content: "garden-brief: fictional seedlings need daylight".into(),
        private: false,
        source_ids: vec![],
    })?;
    approve(vault, "brief", &brief)?;
    let mut report = ImportReport::default();
    let conversation = parse_conversation(
        &json!({
            "id":"fictional-client-source", "title":"Fictional garden source",
            "current_node":"a", "mapping":{"a":{"parent":null,"message":{
                "author":{"role":"user"},"content":{"parts":["Fictional garden source evidence"]}
            }}}
        }),
        &mut report,
    )?;
    let conversation_id = conversation.id.clone();
    let source_id = conversation.messages[0].id.clone();
    vault.import(&[conversation], report)?;
    let mut source_note = note("garden-source: fictional imported evidence", false);
    source_note.source_message_id = Some(source_id);
    let source_note = vault.save_memory(source_note)?;
    approve(vault, "memory", &source_note)?;
    let mut client = vault.create_client("Fictional installed-client check", false)?;
    let socket = directory.path().join("agents.sock");
    let config_path = directory.path().join("client-config.json");
    let mut config = json!({"mcpServers":{"pre-drink":{
        "command":executable,"args":["--mcp"],"env":{
            "PRE_DRINK_SOCKET":socket,
            "PRE_DRINK_CLIENT_ID":client.client.id,
            "PRE_DRINK_CLIENT_TOKEN":client.token
        }
    }}});
    let encoded = Zeroizing::new(serde_json::to_vec(&config)?);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&config_path)?;
    file.write_all(&encoded)?;
    file.sync_all()?;
    if let serde_json::Value::String(secret) =
        &mut config["mcpServers"]["pre-drink"]["env"]["PRE_DRINK_CLIENT_TOKEN"]
    {
        secret.zeroize();
    }
    client.token.zeroize();
    drop(encoded);
    drop(config);
    drop(file);
    let store = Arc::new(Mutex::new(session));
    broker::serve(&socket, store.clone())?;
    println!(
        "{}",
        json!({"config":config_path,"work":work,"sharing":false})
    );
    std::io::stdout().flush()?;
    // Commands are available only on this developer process's stdin, never through MCP.
    for line in std::io::stdin().lock().lines() {
        let line = line?;
        let mut session = store.lock().map_err(|_| "Fixture lock failed")?;
        session.touch();
        let result: Result<(), String> = match line.trim() {
            "sharing_on" => session.access().map(|_| ()).map(|_| session.sharing = true),
            "sharing_off" => {
                session.sharing = false;
                Ok(())
            }
            "edit" => {
                let mut input = note("garden-edited: fictional changed plan", false);
                input.id = Some(approved.clone());
                session.access()?.save_memory(input).map(|_| ())
            }
            "restrict" => session.access()?.mark_private(&conversation_id, true),
            "lock" => {
                session.lock();
                Ok(())
            }
            "unlock" => session.unlock(&password, false, false),
            "revoke" => session.access()?.revoke_client(&client.client.id),
            "restart" => {
                session.lock();
                *session = Session::new(directory.path().into());
                session.unlock(&password, false, false)
            }
            "quit" => break,
            _ => Err("Unknown fixture command".into()),
        };
        println!(
            "{}",
            json!({"command":line,"ok":result.is_ok(),"sharing":session.sharing})
        );
        std::io::stdout().flush()?;
    }
    let mut session = store.lock().map_err(|_| "Fixture lock failed")?;
    if session.vault.is_none() {
        session.unlock(&password, false, false)?;
    }
    session.access()?.revoke_client(&client.client.id)?;
    session.lock();
    drop(session);
    // TempDir removes this process's invented data/config on normal exit, not secure erasure.
    Ok(())
}
