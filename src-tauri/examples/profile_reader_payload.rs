//! Reproducible fictional payload comparison, not a native RAM/latency benchmark.
use pre_drink::{
    db::Vault,
    model::{ImportReport, Message, ParsedConversation},
};
use std::time::Instant;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vault = Vault::memory()?;
    let messages = (0..10_000)
        .map(|i| Message {
            id: format!("fictional-{i}"),
            conversation_id: "fictional-profile".into(),
            role: "user".into(),
            content: format!(
                "Fictional garden entry {i}. {}",
                "cedar planters ".repeat(150)
            ),
            timestamp: None,
            branch_id: None,
            parent_id: None,
            is_active: true,
            sensitive: false,
            position: i,
        })
        .collect();
    vault.import(
        &[ParsedConversation {
            id: "fictional-profile".into(),
            platform: "chatgpt".into(),
            title: "Fictional payload profile".into(),
            created_at: None,
            updated_at: None,
            sensitive: false,
            raw_json: "{}".into(),
            messages,
        }],
        ImportReport::default(),
    )?;
    let started = Instant::now();
    let old = vault.messages("fictional-profile", false)?;
    let old_bytes = serde_json::to_vec(&old)?.len();
    let old_ms = started.elapsed().as_millis();
    drop(old);
    let started = Instant::now();
    let page = vault.message_page("fictional-profile", false, false, "", 0)?;
    let page_bytes = serde_json::to_vec(&page)?.len();
    let page_ms = started.elapsed().as_millis();
    assert_eq!(page.messages.len(), 20);
    assert_eq!(page.total, 10_000);
    println!("fictional_messages=10000 full_response_bytes={old_bytes} page_response_bytes={page_bytes} full_read_serialize_ms={old_ms} page_read_serialize_ms={page_ms}");
    Ok(())
}
