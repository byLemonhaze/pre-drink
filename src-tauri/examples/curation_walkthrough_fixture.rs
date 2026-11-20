//! Fictional isolated UI acceptance data. Never opens the owner's vault.
use pre_drink::{
    agents::AgentRequest,
    curation::{CurationOperation, WriteMemory},
    session::Session,
};
use std::{fs, os::unix::fs::PermissionsExt};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::Builder::new()
        .prefix("pd-curation-preview-")
        .tempdir_in("/tmp")?
        .keep();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700))?;
    let mut session = Session::new(root.clone());
    session.unlock("fictional curation garden phrase", true, false)?;
    let v = session.access()?;
    let client = v.create_scoped_client("Fictional Codex curator", Some("chatgpt"), true)?;
    let grok = v.create_scoped_client("Fictional Grok curator", Some("grok"), true)?;
    for (key, kind, content, conflict) in [
        (
            "project",
            "project",
            "Garden project: a small courtyard with ceramic planters and a shaded reading corner.",
            false,
        ),
        (
            "fact",
            "fact",
            "The fictional gardener prefers terracotta colors.",
            false,
        ),
        (
            "link",
            "decision",
            "Garden reference: https://example.test/planters — review before sharing.",
            false,
        ),
        (
            "conflict",
            "decision",
            "Two garden sketches disagree about planter placement.",
            true,
        ),
    ] {
        v.agent_call(&AgentRequest {
            client_id: client.client.id.clone(),
            token: client.token.clone(),
            tool: "curate_memory".into(),
            query: String::new(),
            limit: None,
            conversation_id: String::new(),
            curation: Some(CurationOperation::Write {
                input: WriteMemory {
                    request_key: key.into(),
                    id: None,
                    expected_revision: 0,
                    kind: kind.into(),
                    content: content.into(),
                    origin: "session".into(),
                    session_label: "Fictional garden planning".into(),
                    sources: vec![],
                    conflict,
                },
            }),
        })?;
    }
    v.agent_call(&AgentRequest {
        client_id: grok.client.id,
        token: grok.token,
        tool: "curate_memory".into(),
        query: String::new(),
        limit: None,
        conversation_id: String::new(),
        curation: Some(CurationOperation::Write {
            input: WriteMemory {
                request_key: "grok-session".into(),
                id: None,
                expected_revision: 0,
                kind: "project".into(),
                content: "Grok garden draft: consider a sandstone path.".into(),
                origin: "session".into(),
                session_label: "Fictional Grok garden".into(),
                sources: vec![],
                conflict: false,
            },
        }),
    })?;
    session.lock();
    println!("{}", root.display());
    Ok(())
}
