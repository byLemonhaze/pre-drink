use pre_drink::{
    db::Vault,
    import::{parse_conversation, read_archive},
    model::ImportReport,
    privacy::is_sensitive,
};
use serde_json::{json, Value};
use std::io::{Cursor, Write};
use zip::{write::SimpleFileOptions, ZipWriter};

fn fixture() -> Value {
    json!({
        "id":"conversation-1", "title":"A calmer workspace", "create_time":1700000000.0, "update_time":1700000100.5,
        "current_node":"answer-2", "mapping": {
            "root":{"parent":null,"message":null},
            "question":{"parent":"root","message":{"author":{"role":"user"},"content":{"parts":["Help me organize a reading nook."]},"create_time":1700000001}},
            "answer-1":{"parent":"question","message":{"author":{"role":"assistant"},"content":{"parts":["Use a mahogany bookshelf."]},"create_time":1700000002}},
            "answer-2":{"parent":"question","message":{"author":{"role":"assistant"},"content":{"parts":["Try a walnut shelf and warm lighting."]},"create_time":1700000003}}
        }
    })
}

fn archive(files: &[(&str, Vec<u8>)]) -> Cursor<Vec<u8>> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, content) in files {
        writer
            .start_file(*name, SimpleFileOptions::default())
            .unwrap();
        writer.write_all(content).unwrap();
    }
    let mut result = writer.finish().unwrap();
    result.set_position(0);
    result
}

fn import(vault: &mut Vault, data: Value) -> ImportReport {
    let zip = archive(&[(
        "conversations.json",
        serde_json::to_vec(&vec![data]).unwrap(),
    )]);
    let (conversations, report) = read_archive(zip).unwrap();
    vault.import(&conversations, report).unwrap()
}

fn vault() -> Vault {
    Vault::memory().unwrap()
}

#[test]
fn preserves_branches_parents_and_fractional_timestamps() {
    let c = parse_conversation(&fixture(), &mut ImportReport::default()).unwrap();
    assert_eq!(c.messages.len(), 3);
    assert_eq!(c.updated_at, Some(1700000100.5));
    let old = c
        .messages
        .iter()
        .find(|m| m.content.contains("mahogany"))
        .unwrap();
    let new = c
        .messages
        .iter()
        .find(|m| m.content.contains("walnut"))
        .unwrap();
    assert!(!old.is_active);
    assert!(old.branch_id.is_some());
    assert!(new.is_active);
    assert_eq!(old.parent_id, new.parent_id);
    assert_eq!(c.messages.iter().filter(|m| m.is_active).count(), 2);
}

#[test]
fn zip_to_database_searches_titles_and_inactive_branches() {
    let mut v = vault();
    let report = import(&mut v, fixture());
    assert_eq!(report.new_conversations, 1);
    assert_eq!(v.list("workspace", false, 0).unwrap().len(), 1);
    assert_eq!(v.list("mahogany", false, 0).unwrap().len(), 1);
    assert_eq!(v.list("walnut lighting", false, 0).unwrap().len(), 1);
    assert!(v.list("no-match-here", false, 0).unwrap().is_empty());
    for query in ["\"", "*", "OR", "(walnut)", "a NEAR b", "-", ":"] {
        assert!(
            v.list(query, false, 0).is_ok(),
            "literal query failed: {query}"
        );
    }
}

#[test]
fn repeat_import_is_idempotent_and_preserves_private_flag() {
    let mut v = vault();
    import(&mut v, fixture());
    let id = v.list("", false, 0).unwrap()[0].id.clone();
    v.mark_private(&id, true).unwrap();
    let report = import(&mut v, fixture());
    assert_eq!(report.new_conversations, 0);
    assert_eq!(report.updated_conversations, 1);
    assert_eq!(v.stats().unwrap().messages, 3);
    assert_eq!(v.stats().unwrap().private, 1);
    assert!(v.list("", false, 0).unwrap().is_empty());
    assert!(v.list("walnut", false, 0).unwrap().is_empty());
    assert!(v.messages(&id, false).is_err());
    assert_eq!(v.messages(&id, true).unwrap().len(), 3);
    v.mark_private(&id, false).unwrap();
    assert_eq!(v.list("", false, 0).unwrap().len(), 1);
}

#[test]
fn sensitive_titles_and_messages_are_hidden_by_default() {
    for sensitive_title in [false, true] {
        let mut data = fixture();
        if sensitive_title {
            data["title"] = json!("Contact person@example.org");
        } else {
            data["mapping"]["question"]["message"]["content"]["parts"] =
                json!(["Email me at person@example.org"]);
        }
        let mut v = vault();
        import(&mut v, data);
        assert_eq!(v.stats().unwrap().sensitive, 1);
        assert!(v.list("", false, 0).unwrap().is_empty());
        let shown = v.list("", true, 0).unwrap();
        assert_eq!(shown.len(), 1);
        assert!(v.messages(&shown[0].id, false).is_err());
        assert!(v.messages(&shown[0].id, true).is_ok());
    }
}

#[test]
fn retained_branches_still_protect_reimported_conversation() {
    let mut data = fixture();
    data["mapping"]["answer-1"]["message"]["content"]["parts"] = json!(["My bank account details"]);
    let mut v = vault();
    import(&mut v, data.clone());
    data["mapping"].as_object_mut().unwrap().remove("answer-1");
    import(&mut v, data);
    assert!(v.list("", false, 0).unwrap().is_empty());
    assert_eq!(v.stats().unwrap().messages, 3);
}

#[test]
fn nested_and_split_archives_work_without_extracting_paths() {
    let mut second = fixture();
    second["id"] = json!("conversation-2");
    let files = [
        (
            "nested/conversations-000.json",
            serde_json::to_vec(&vec![fixture()]).unwrap(),
        ),
        (
            "../../conversations-001.json",
            serde_json::to_vec(&vec![second]).unwrap(),
        ),
        ("accounts.json", b"unrelated".to_vec()),
    ];
    let (items, report) = read_archive(archive(&files)).unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(report.messages, 6);
}

#[test]
fn unsupported_and_malformed_archives_fail_before_database_writes() {
    for files in [
        vec![("chat.html", b"<html/>".to_vec())],
        vec![("conversations.json", b"invalid".to_vec())],
        vec![("conversations.json", b"{}".to_vec())],
        vec![(
            "conversations.json",
            serde_json::to_vec(&vec![fixture(), json!({"id":"broken"})]).unwrap(),
        )],
        vec![(
            "conversations.json",
            serde_json::to_vec(&vec![fixture(), fixture()]).unwrap(),
        )],
    ] {
        assert!(read_archive(archive(&files)).is_err());
    }
    assert!(read_archive(Cursor::new(b"not a zip")).is_err());
}

#[test]
fn cyclic_trees_are_rejected_even_outside_the_current_branch() {
    let mut data = fixture();
    data["mapping"]["cycle-a"] = json!({"parent":"cycle-b","message":null});
    data["mapping"]["cycle-b"] = json!({"parent":"cycle-a","message":null});
    assert!(parse_conversation(&data, &mut ImportReport::default()).is_err());
    data["current_node"] = json!("cycle-a");
    assert!(parse_conversation(&data, &mut ImportReport::default()).is_err());
}

#[test]
fn mixed_content_does_not_stringify_objects_or_expose_hidden_messages() {
    let mut data = fixture();
    data["mapping"]["question"]["message"]["content"]["parts"] =
        json!(["Describe this", {"asset_pointer":"file-service://private"}, {"text":"caption"}]);
    data["mapping"]["answer-1"]["message"]["metadata"] =
        json!({"is_visually_hidden_from_conversation":true});
    let mut report = ImportReport::default();
    let c = parse_conversation(&data, &mut report).unwrap();
    assert_eq!(c.messages.len(), 2);
    assert_eq!(c.messages[0].content, "Describe this\ncaption");
    assert_eq!(report.skipped_messages, 1);
    assert!(c.raw_json.contains("asset_pointer"));
}

#[test]
fn missing_current_node_uses_latest_leaf_and_reports_it() {
    let mut data = fixture();
    data["current_node"] = Value::Null;
    let mut report = ImportReport::default();
    let c = parse_conversation(&data, &mut report).unwrap();
    assert!(!report.warnings.is_empty());
    assert_eq!(c.messages.iter().filter(|m| m.is_active).count(), 2);
    assert!(c
        .messages
        .iter()
        .any(|m| m.is_active && m.content.contains("walnut")));
}

#[test]
fn database_survives_reopening() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("vault.sqlite3");
    {
        let mut v = Vault::create(&path, "a unique test passphrase").unwrap();
        import(&mut v, fixture());
    }
    let reopened = Vault::open(&path, "a unique test passphrase").unwrap();
    assert_eq!(reopened.stats().unwrap().messages, 3);
    assert_eq!(reopened.list("mahogany", false, 0).unwrap().len(), 1);
}

#[test]
fn transaction_failure_rolls_back_every_conversation() {
    let mut v = vault();
    let good = parse_conversation(&fixture(), &mut ImportReport::default()).unwrap();
    let mut other = fixture();
    other["id"] = json!("other");
    let mut bad = parse_conversation(&other, &mut ImportReport::default()).unwrap();
    bad.messages[0].role = "invalid-role".into();
    assert!(v.import(&[good, bad], ImportReport::default()).is_err());
    assert_eq!(v.stats().unwrap().conversations, 0);
    assert!(v.list("walnut", true, 0).unwrap().is_empty());
}

#[test]
fn privacy_rules_cover_basic_sensitive_patterns() {
    for text in [
        "person@example.org",
        "514-555-1234",
        "123 Maple Street",
        "prescription refill",
        "credit card",
        "api_key",
    ] {
        assert!(is_sensitive(text), "not detected: {text}");
    }
    assert!(!is_sensitive("A walnut bookshelf and a reading lamp."));
}

#[test]
fn deep_message_trees_do_not_recurse() {
    let mut mapping = serde_json::Map::new();
    for n in 0..10_000 {
        mapping.insert(
            n.to_string(),
            json!({"parent": if n == 0 { None } else { Some((n-1).to_string()) }, "message":null}),
        );
    }
    let data = json!({"id":"deep","current_node":"9999","mapping":mapping});
    assert!(parse_conversation(&data, &mut ImportReport::default()).is_ok());
}
