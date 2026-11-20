//! Repeatable synthetic scale check; never loads a personal export or vault.
use pre_drink::{
    db::Vault,
    model::{ImportReport, Message, ParsedConversation},
};
use std::time::Instant;

#[test]
#[ignore = "explicit 5,000-conversation encrypted-vault benchmark"]
fn encrypted_library_5000_conversations() {
    let data: Vec<_> = (0..5000).map(|i| {
        let id = format!("synthetic-{i}");
        ParsedConversation {
            id: id.clone(), title: format!("Synthetic project {i} garden planning"),
            created_at: Some(1_700_000_000.0), updated_at: Some(1_700_000_000.0 + i as f64),
            sensitive: false, raw_json: format!("{{\"synthetic\":true,\"id\":{i}}}"),
            messages: (0..20).map(|j| Message {
                id: format!("{id}-{j}"), conversation_id: id.clone(), role: if j % 2 == 0 { "user" } else { "assistant" }.into(),
                content: format!("Synthetic garden planning project {i}, note {j}. Choose native plants and make a seasonal schedule. This is fictional benchmark text."),
                timestamp: Some(1_700_000_000.0 + j as f64), branch_id: Some("main".into()), parent_id: if j == 0 { None } else { Some(format!("{id}-{}", j - 1)) }, is_active: true, sensitive: false, position: j,
            }).collect(),
        }
    }).collect();
    let dir = tempfile::tempdir().unwrap();
    let mut v = Vault::create(
        &dir.path().join("benchmark.enc.sqlite3"),
        "synthetic scale benchmark phrase",
    )
    .unwrap();
    let start = Instant::now();
    v.import(&data, ImportReport::default()).unwrap();
    println!(
        "synthetic_import_5000_conversations_100000_messages_ms={}",
        start.elapsed().as_millis()
    );
    assert_eq!(v.stats().unwrap().messages, 100_000);
    for query in ["", "garden", "garden planning"] {
        let start = Instant::now();
        let found = v.list(query, false, 0).unwrap();
        println!(
            "synthetic_list_query_{query:?}_ms={}",
            start.elapsed().as_millis()
        );
        assert_eq!(found.len(), 100);
    }
    let start = Instant::now();
    let pack = v.context_pack("garden", true).unwrap();
    println!("synthetic_chat_context_ms={}", start.elapsed().as_millis());
    assert!(pack.contains("Synthetic garden planning"));
    let start = Instant::now();
    v.import(&data, ImportReport::default()).unwrap();
    println!("synthetic_repeat_import_ms={}", start.elapsed().as_millis());
    assert_eq!(v.stats().unwrap().messages, 100_000);
    v.verify().unwrap();
}
