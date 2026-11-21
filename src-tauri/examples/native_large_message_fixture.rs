//! One invented oversized message; never reads the user's app storage.
use pre_drink::{
    model::{ImportReport, Message, ParsedConversation},
    session::Session,
};
use serde_json::json;
use std::{fs, os::unix::fs::PermissionsExt};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::Builder::new()
        .prefix("pd-native-large-")
        .tempdir_in("/tmp")?
        .keep();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700))?;
    let mut s = Session::new(root.clone());
    const PASSWORD: &str = "fictional reading garden phrase";
    s.unlock(PASSWORD, true, false)?;
    let content = format!("# Fictional oversized garden plan\n\n{}\nlargefinalorchid: all fictional entries remain readable. 🌿\n<script>fictional inert text</script>",
        "Choose cedar planters. **Keep the walkway clear.** 🌿\n\n".repeat(10000));
    let characters = content.chars().count();
    let v = s.access()?;
    v.import(
        &[ParsedConversation {
            platform: "chatgpt".into(),
            id: "fictional-large".into(),
            title: "Fictional oversized garden message".into(),
            created_at: None,
            updated_at: None,
            sensitive: false,
            raw_json: json!({"fictional":true}).to_string(),
            messages: vec![Message {
                id: "fictional-large-01".into(),
                conversation_id: "fictional-large".into(),
                role: "user".into(),
                content,
                timestamp: None,
                branch_id: None,
                parent_id: None,
                is_active: true,
                sensitive: false,
                position: 0,
            }],
        }],
        ImportReport::default(),
    )?;
    v.verify()?;
    assert_eq!(v.list("largefinalorchid", false, 0)?.len(), 1);
    s.lock();
    println!(
        "{}",
        json!({"app_data_dir":root,"fictional_password":PASSWORD,"characters":characters})
    );
    Ok(())
}
