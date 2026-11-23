//! Disposable UI fixture. Its random client credentials are never persisted or
//! printed, and no external client connects. Never point it at an existing vault.
use pre_drink::{agents::AgentRequest, session::Session};
use std::{fs, os::unix::fs::PermissionsExt};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::Builder::new()
        .prefix("pd-native-activity-")
        .tempdir_in("/tmp")?;
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700))?;
    let live = root.path().join("live");
    fs::create_dir(&live)?;
    fs::set_permissions(&live, fs::Permissions::from_mode(0o700))?;
    let mut s = Session::new(live.clone());
    s.unlock("fictional activity native garden", true, false)?;
    let v = s.access()?;
    let reader = v.create_client("Fictional shared reader", false)?;
    let curator = v.create_scoped_client("Fictional Grok curator", Some("grok"), true)?;
    for tool in ["connection_status", "get_facts", "search_conversations"] {
        let _ = v.agent_call(&AgentRequest {
            client_id: reader.client.id.clone(),
            token: reader.token.clone(),
            tool: tool.into(),
            query: "fictional garden".into(),
            limit: None,
            conversation_id: String::new(),
            curation: None,
        });
    }
    v.set_client_paused(&curator.client.id, true)?;
    v.verify()?;
    v.checkpoint()?;
    s.lock();
    let root = root.keep();
    println!(
        "{}",
        serde_json::json!({"fixture_root":root,"app_data_dir":live})
    );
    Ok(())
}
