//! Fictional, versioned compatibility contracts. No personal exports are read.
use pre_drink::{db::Vault, import::read_archive, model::ParsedConversation};
use serde_json::{json, Value};
use std::{
    io::{Cursor, Write},
    path::PathBuf,
};
use zip::{write::SimpleFileOptions, ZipWriter};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/chatgpt/mapping-v1")
}
fn fixture(name: &str) -> Value {
    serde_json::from_slice(&std::fs::read(fixture_root().join(name)).unwrap()).unwrap()
}
fn zip(files: &[(&str, Vec<u8>)]) -> Cursor<Vec<u8>> {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, bytes) in files {
        zip.start_file(*name, SimpleFileOptions::default()).unwrap();
        zip.write_all(bytes).unwrap();
    }
    let mut result = zip.finish().unwrap();
    result.set_position(0);
    result
}
fn bytes(value: &Value) -> Vec<u8> {
    serde_json::to_vec(value).unwrap()
}
fn normalized(c: &ParsedConversation) -> Value {
    json!({"id":c.id,"title":c.title,"created_at":c.created_at,"updated_at":c.updated_at,"sensitive":c.sensitive,"messages":c.messages})
}

#[test]
fn versioned_chatgpt_contracts_match_reviewed_normalization_and_omissions() {
    let manifest = fixture("manifest.json");
    assert_eq!(manifest["provider"], "chatgpt");
    assert_eq!(manifest["contract_revision"], 1);
    assert!(manifest["official_format_version"].is_null());
    for case in manifest["cases"].as_array().unwrap() {
        let input = fixture(case["input"].as_str().unwrap());
        let expected = fixture(case["expected"].as_str().unwrap());
        let (parsed, report) = read_archive(zip(&[("conversations.json", bytes(&input))])).unwrap();
        let actual = json!({"conversations":parsed.iter().map(normalized).collect::<Vec<_>>(),"messages":report.messages,"skipped_messages":report.skipped_messages,"warnings":report.warnings});
        assert_eq!(actual, expected, "contract {}", case["name"]);
        assert_eq!(report.conversations, parsed.len());
        for (parsed, original) in parsed.iter().zip(input.as_array().unwrap()) {
            // Unknown fields and omitted reader messages remain archived as JSON values.
            assert_eq!(
                serde_json::from_str::<Value>(&parsed.raw_json).unwrap(),
                *original
            );
        }
    }
}

#[test]
fn archive_layouts_and_repeat_import_keep_contract_identity_and_search() {
    let branched = fixture("branched.input.json");
    let mixed = fixture("mixed-content.input.json");
    let mut vault = Vault::memory().unwrap();
    for entry in [
        "conversations.json",
        "nested/conversations.json",
        "conversations-000.json",
        "..\\fictional\\conversations-001.json",
    ] {
        let (parsed, report) = read_archive(zip(&[(entry, bytes(&branched))])).unwrap();
        vault.import(&parsed, report).unwrap();
        assert_eq!(vault.stats().unwrap().conversations, 1);
        assert_eq!(vault.stats().unwrap().messages, 4);
        assert_eq!(vault.list("mahogany", false, 0).unwrap().len(), 1);
    }
    let (parsed, report) = read_archive(zip(&[
        ("nested/conversations-000.json", bytes(&branched)),
        ("nested/conversations-001.json", bytes(&mixed)),
        ("unrelated.json", b"not conversation data".to_vec()),
    ]))
    .unwrap();
    let report = vault.import(&parsed, report).unwrap();
    assert_eq!(
        (report.new_conversations, report.updated_conversations),
        (1, 1)
    );
    assert_eq!(
        (
            vault.stats().unwrap().conversations,
            vault.stats().unwrap().messages
        ),
        (2, 6)
    );
    assert_eq!(vault.list("seedling", false, 0).unwrap().len(), 1);
    assert!(vault.list("intermediate", false, 0).unwrap().is_empty());
}

#[test]
fn inactive_fixture_branch_still_restricts_the_whole_conversation() {
    let mut input = fixture("branched.input.json");
    input[0]["mapping"]["old"]["message"]["content"]["parts"] =
        json!(["Fictional contact: gardener@example.org"]);
    let (parsed, report) = read_archive(zip(&[("conversations.json", bytes(&input))])).unwrap();
    assert!(parsed[0].sensitive);
    let mut vault = Vault::memory().unwrap();
    vault.import(&parsed, report).unwrap();
    assert!(vault.list("walnut", false, 0).unwrap().is_empty());
    assert_eq!(vault.list("walnut", true, 0).unwrap().len(), 1);
}

#[test]
fn later_malformed_or_duplicate_records_never_produce_a_partial_import() {
    let input = fixture("branched.input.json");
    let mut vault = Vault::memory().unwrap();
    let (parsed, report) = read_archive(zip(&[("conversations.json", bytes(&input))])).unwrap();
    vault.import(&parsed, report).unwrap();
    let mut malformed = input.clone();
    malformed[0]["conversation_id"] = json!("fictional-new-record");
    malformed[0]["mapping"] = json!({"broken":42});
    for second in [
        bytes(&malformed),
        bytes(&input),
        b"invalid JSON".to_vec(),
        b"{}".to_vec(),
    ] {
        let result = read_archive(zip(&[
            ("conversations-000.json", bytes(&input)),
            ("conversations-001.json", second),
        ]))
        .and_then(|(parsed, report)| vault.import(&parsed, report));
        assert!(result.is_err());
        assert_eq!(
            (
                vault.stats().unwrap().conversations,
                vault.stats().unwrap().messages
            ),
            (1, 4)
        );
        assert_eq!(vault.list("mahogany", false, 0).unwrap().len(), 1);
    }
}

#[test]
fn oversized_declared_json_is_rejected_before_decompression() {
    let mut data = zip(&[("conversations.json", b"[]".to_vec())]).into_inner();
    let header = data.windows(4).position(|w| w == b"PK\x01\x02").unwrap();
    // Advertise excessive uncompressed JSON without allocating a huge test payload.
    data[header + 24..header + 28].copy_from_slice(&(256_u32 * 1024 * 1024 + 1).to_le_bytes());
    let error = read_archive(Cursor::new(data)).unwrap_err();
    assert!(error.contains("256 MB import limit"));
}
