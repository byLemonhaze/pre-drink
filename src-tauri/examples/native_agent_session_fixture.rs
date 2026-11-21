//! Disposable fictional owner-hold/session walkthrough. Never opens personal storage.
use pre_drink::{
    agents::AgentRequest,
    curation::{CurationOperation, WriteMemory},
    session::Session,
};
use std::{fs, os::unix::fs::PermissionsExt};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::Builder::new()
        .prefix("pd-native-agent-session-")
        .tempdir_in("/tmp")?
        .keep();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700))?;
    let mut session = Session::new(root.clone());
    const PASSWORD: &str = "fictional session garden phrase";
    session.unlock(PASSWORD, true, false)?;
    let vault = session.access()?;
    let client = vault.create_scoped_client("Fictional fixture curator", Some("chatgpt"), true)?;
    let result = vault.agent_call(&AgentRequest {
        client_id: client.client.id.clone(),
        token: client.token,
        tool: "curate_memory".into(),
        query: String::new(),
        limit: None,
        conversation_id: String::new(),
        curation: Some(CurationOperation::Write {
            input: WriteMemory {
                request_key: "fictional-held-garden".into(),
                id: None,
                expected_revision: 0,
                kind: "project".into(),
                content: "Garden uses ceramic planters.".into(),
                origin: "session".into(),
                session_label: "Fictional garden discussion".into(),
                sources: vec![],
                conflict: false,
            },
        }),
    })?;
    vault.edit_curation(
        result["id"].as_str().unwrap(),
        1,
        "Garden uses ceramic planters.",
        true,
    )?;
    vault.revoke_client(&client.client.id)?;
    session.lock();
    println!(
        "{}",
        serde_json::json!({"app_data_dir":root,"fictional_password":PASSWORD})
    );
    Ok(())
}
