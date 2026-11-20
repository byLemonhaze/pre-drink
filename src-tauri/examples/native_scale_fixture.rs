//! Disposable fictional library for native pagination/search/keyboard acceptance.
//! Never reads personal exports or opens the default app storage directory.
use pre_drink::{
    model::{ImportReport, Message, ParsedConversation},
    session::Session,
};
use serde_json::json;
use std::{fs, os::unix::fs::PermissionsExt, time::Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::Builder::new()
        .prefix("pd-native-scale-")
        .tempdir_in("/tmp")?
        .keep();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700))?;
    let live = root.join("live");
    fs::create_dir(&live)?;
    let mut session = Session::new(live.clone());
    const PASSWORD: &str = "fictional scale garden phrase";
    session.unlock(PASSWORD, true, false)?;
    let conversations: Vec<_> = (0..5000)
        .map(|i| {
            let id = format!("fictional-scale-{i}");
            ParsedConversation {
                id: id.clone(),
                title: format!("Fictional garden project {i:04}"),
                created_at: Some(1_700_000_000.0),
                updated_at: Some(1_700_000_000.0 + i as f64),
                sensitive: false,
                raw_json: json!({"fictional":true,"fixture_id":i}).to_string(),
                messages: (0..20)
                    .map(|j| Message {
                        id: format!("{id}-{j}"),
                        conversation_id: id.clone(),
                        role: if j % 2 == 0 { "user" } else { "assistant" }.into(),
                        content: format!("Fictional garden project {i:04}, note {j}. Choose native plants and a seasonal schedule. Search marker gardenmarker{i:04}."),
                        timestamp: Some(1_700_000_000.0 + j as f64),
                        branch_id: Some("main".into()),
                        parent_id: (j > 0).then(|| format!("{id}-{}", j - 1)),
                        is_active: true,
                        sensitive: false,
                        position: j,
                    })
                    .collect(),
            }
        })
        .collect();
    let vault = session.access()?;
    let started = Instant::now();
    vault.import(&conversations, ImportReport::default())?;
    let import_ms = started.elapsed().as_millis();
    vault.verify()?;
    let stats = vault.stats()?;
    assert_eq!(stats.conversations, 5000);
    assert_eq!(stats.messages, 100_000);
    assert_eq!(vault.list("gardenmarker0042", false, 0)?.len(), 1);
    session.lock();
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "fixture_root":root,"app_data_dir":live,"fictional_password":PASSWORD,
            "conversations":5000,"messages":100000,"normalized_import_ms":import_ms,
            "unique_search":"gardenmarker0042"
        }))?
    );
    Ok(())
}
