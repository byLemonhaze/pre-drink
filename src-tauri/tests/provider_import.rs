//! Fictional official-export shape contracts, including source access after import.
use pre_drink::{
    agents::AgentRequest,
    curation::{CurationOperation, SourceRef, WriteMemory},
    db::Vault,
    import::read_archive,
};
use serde_json::{json, Value};
use std::io::{Cursor, Write};
fn zip(files: &[(&str, Value)]) -> Cursor<Vec<u8>> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, data) in files {
        zip.start_file(*name, zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(&serde_json::to_vec(data).unwrap()).unwrap();
    }
    let mut c = zip.finish().unwrap();
    c.set_position(0);
    c
}
fn grok() -> Value {
    serde_json::from_str(include_str!("fixtures/grok/account-v1/garden.input.json")).unwrap()
}
fn claude() -> Value {
    serde_json::from_str(include_str!(
        "fixtures/claude/messages-v1/garden.input.json"
    ))
    .unwrap()
}
fn request(client: &pre_drink::agents::NewClient, tool: &str) -> AgentRequest {
    AgentRequest {
        client_id: client.client.id.clone(),
        token: client.token.clone(),
        tool: tool.into(),
        query: "garden".into(),
        limit: None,
        conversation_id: String::new(),
        curation: None,
    }
}
#[test]
fn contracts_preserve_text_dates_graphs_and_raw_without_promoting_reasoning() {
    let (items, report) = read_archive(zip(&[
        ("nested/prod-grok-backend.json", grok()),
        ("other/conversations.json", claude()),
    ]))
    .unwrap();
    assert_eq!(
        (
            report.conversations,
            report.messages,
            report.skipped_messages
        ),
        (2, 5, 2)
    );
    let g = &items[0];
    let c = &items[1];
    assert_eq!((&*g.platform, &*c.platform), ("grok", "claude"));
    assert_ne!(g.id, c.id);
    assert_eq!(g.created_at, Some(1704067200.0));
    assert_eq!(g.updated_at, c.updated_at);
    assert_eq!(
        g.messages
            .iter()
            .map(|m| m.content.as_str())
            .collect::<Vec<_>>(),
        vec![
            "Garden needs a shaded corner.",
            "Use limestone for the garden.",
            "Use sandstone for the garden."
        ]
    );
    assert!(!g.messages[1].is_active);
    assert!(g.messages[2].is_active);
    assert_eq!(g.messages[1].parent_id, Some(g.messages[0].id.clone()));
    assert_eq!(
        c.messages[1].content,
        "Use cedar for the garden.\nKeep a reading bench."
    );
    assert!(c.messages.iter().all(|m| m.is_active));
    assert_eq!(
        serde_json::from_str::<Value>(&g.raw_json).unwrap(),
        grok()["conversations"][0]
    );
    assert_eq!(
        serde_json::from_str::<Value>(&c.raw_json).unwrap(),
        claude()[0]
    );
    for item in &items {
        assert!(item
            .messages
            .iter()
            .all(|m| !m.content.contains("hidden-thought") && !m.content.contains("tool-only")));
    }
    let mut v = Vault::memory().unwrap();
    v.import(&items, report).unwrap();
    let (_, report) = read_archive(zip(&[
        ("prod-grok-backend.json", grok()),
        ("conversations-000.json", claude()),
    ]))
    .unwrap();
    let r = v.import(&items, report).unwrap();
    assert_eq!((r.new_conversations, r.updated_conversations), (0, 2));
    assert_eq!(v.stats().unwrap().messages, 5);
    assert_eq!(v.list("sandstone", false, 0).unwrap()[0].platform, "grok");
    assert_eq!(v.list("cedar", false, 0).unwrap()[0].platform, "claude");
}
#[test]
fn all_three_curators_are_isolated_after_real_parser_paths_and_privacy_propagates() {
    let mut v = Vault::memory().unwrap();
    let mut files = vec![
        ("prod-grok-backend.json", grok()),
        ("conversations-001.json", claude()),
    ];
    files.push(("conversations-002.json",json!([{"id":"fictional-shared-id","title":"ChatGPT garden","current_node":"q","mapping":{"q":{"message":{"author":{"role":"user"},"content":{"parts":["Garden uses clay."]}}}}}])));
    let (items, report) = read_archive(zip(&files)).unwrap();
    v.import(&items, report).unwrap();
    let clients: Vec<_> = ["grok", "claude", "chatgpt"]
        .iter()
        .map(|p| {
            v.create_scoped_client("Fictional curator", Some(p), true)
                .unwrap()
        })
        .collect();
    for (i, client) in clients.iter().enumerate() {
        let matches = v
            .agent_call(&request(client, "search_conversations"))
            .unwrap();
        assert_eq!(matches.as_array().unwrap().len(), 1);
        assert_eq!(matches[0]["platform"], items[i].platform);
        for (j, item) in items.iter().enumerate() {
            let mut r = request(client, "curate_memory");
            r.curation = Some(CurationOperation::ReadSource {
                message_id: item.messages[0].id.clone(),
                offset: None,
            });
            assert_eq!(v.agent_call(&r).is_ok(), i == j);
            r.curation = Some(CurationOperation::Write {
                input: WriteMemory {
                    request_key: format!("source-{j}"),
                    id: None,
                    expected_revision: 0,
                    kind: "project".into(),
                    content: "Garden uses a reading corner.".into(),
                    origin: "archive".into(),
                    session_label: String::new(),
                    sources: vec![SourceRef {
                        message_id: item.messages[0].id.clone(),
                        version: 1,
                    }],
                    conflict: false,
                },
            });
            let result = v.agent_call(&r);
            assert_eq!(result.is_ok(), i == j);
            if i == j {
                let note = result.unwrap()["id"].as_str().unwrap().to_string();
                let review = v.curation_review(std::slice::from_ref(&note)).unwrap();
                v.publish_curation_review(&[note], &review.review_hash)
                    .unwrap();
            }
        }
    }
    let reader = v
        .create_scoped_client("Fictional shared reader", None, false)
        .unwrap();
    assert_eq!(
        v.agent_call(&request(&reader, "get_projects"))
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        3
    );
    v.mark_private(&items[0].id, true).unwrap();
    assert_eq!(
        v.agent_call(&request(&reader, "get_projects"))
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let (fresh, report) = read_archive(zip(&files)).unwrap();
    v.import(&fresh, report).unwrap();
    assert!(v.list("sandstone", false, 0).unwrap().is_empty());
}
#[test]
fn malformed_duplicate_ambiguous_or_cyclic_exports_fail_atomically() {
    let mut bad = grok();
    bad["conversations"][0]["responses"][1]["response"]["_id"] = json!("q");
    let mut cycle = grok();
    cycle["conversations"][0]["responses"][0]["response"]["parent_response_id"] = json!("new");
    let mut missing = grok();
    missing["conversations"][0]["conversation"]
        .as_object_mut()
        .unwrap()
        .remove("id");
    for data in [
        bad,
        cycle,
        missing,
        json!({"conversations":[{}]}),
        json!([]),
    ] {
        assert!(read_archive(zip(&[
            ("conversations.json", claude()),
            ("prod-grok-backend.json", data)
        ]))
        .is_err());
    }
    let mut bad = claude();
    bad[0]["chat_messages"][1]["uuid"] = json!("q");
    assert!(read_archive(zip(&[("conversations.json", bad)])).is_err());
    let mut bad = claude();
    bad[0]["mapping"] = json!({});
    assert!(read_archive(zip(&[("conversations.json", bad)])).is_err());
    assert!(read_archive(zip(&[("arbitrary-live-capture.json", grok())])).is_err());
}
#[test]
fn restricted_retained_branch_and_no_text_are_not_silently_lost() {
    let mut g = grok();
    g["conversations"][0]["responses"][1]["response"]["message"] =
        json!("Fictional contact gardener@example.org");
    let mut c = claude();
    c[0]["chat_messages"][0]["text"] = json!("Fictional contact gardener@example.org");
    let (items, report) = read_archive(zip(&[
        ("..\\nested\\prod-grok-backend.json", g),
        ("nested/conversations.json", c),
    ]))
    .unwrap();
    assert!(items.iter().all(|i| i.sensitive));
    let mut v = Vault::memory().unwrap();
    v.import(&items, report).unwrap();
    assert!(v.list("garden", false, 0).unwrap().is_empty());
    assert_eq!(v.list("garden", true, 0).unwrap().len(), 2);
    let mut c = claude();
    c[0]["chat_messages"] = json!([]);
    let (items, r) = read_archive(zip(&[("conversations.json", c)])).unwrap();
    assert_eq!(r.conversations, 1);
    assert!(items[0].messages.is_empty());
}

#[test]
fn flat_response_records_and_legacy_claude_text_remain_readable() {
    let mut g = grok();
    for item in g["conversations"][0]["responses"].as_array_mut().unwrap() {
        *item = item["response"].take();
    }
    let mut c = claude();
    c[0]["chat_messages"][0]["content"] = json!([]);
    c[0]["chat_messages"][1]["content"] = json!([{"type":"thinking","thinking":"unshared"}]);
    c[0]["chat_messages"][1]["text"] =
        json!("must not fall back to this for reasoning-only blocks");
    let (items, _) = read_archive(zip(&[
        ("prod-grok-backend.json", g),
        ("conversations.json", c),
    ]))
    .unwrap();
    assert_eq!(items[0].messages.len(), 3);
    assert_eq!(items[1].messages.len(), 1);
    assert_eq!(
        items[1].messages[0].content,
        "Garden needs a shaded corner."
    );
}
