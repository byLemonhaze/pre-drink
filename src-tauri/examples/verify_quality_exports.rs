//! Validate the packaged fictional corpus through each production import path.
use pre_drink::{db::Vault, import::read_export_zip};
use std::path::Path;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let corpus: serde_json::Value = serde_json::from_str(include_str!(
        "../tests/fixtures/curation/quality-v1/cases.json"
    ))?;
    let cases = corpus["cases"].as_array().unwrap();
    for provider in ["chatgpt", "grok", "claude"] {
        let path = format!("artifacts/curation-quality-v1/fictional-{provider}-quality.zip");
        let (conversations, report) = read_export_zip(Path::new(&path))?;
        assert_eq!(conversations.len(), cases.len());
        assert_eq!(report.skipped_messages, 0);
        for (conversation, case) in conversations.iter().zip(cases) {
            assert_eq!(conversation.platform, provider);
            let messages = case["messages"].as_array().unwrap();
            assert_eq!(conversation.messages.len(), messages.len());
            for (actual, expected) in conversation.messages.iter().zip(messages) {
                assert_eq!(actual.role, expected[0].as_str().unwrap());
                assert_eq!(actual.content, expected[1].as_str().unwrap());
            }
            if case["id"] == "sensitive-source" {
                assert!(conversation.sensitive);
            }
        }
        let mut v = Vault::memory()?;
        v.import(&conversations, report)?;
        let client = v.create_scoped_client("Fictional corpus check", Some(provider), true)?;
        let visible = v.agent_call(&pre_drink::agents::AgentRequest {
            client_id: client.client.id,
            token: client.token,
            tool: "curate_memory".into(),
            query: String::new(),
            limit: None,
            conversation_id: String::new(),
            curation: Some(pre_drink::curation::CurationOperation::ListSources {
                after: None,
                limit: Some(50),
            }),
        })?;
        let text = visible.to_string();
        assert!(!text.contains("sensitive-source"));
        assert!(text.contains("changed-decision"));
        println!("{provider}: all fictional source texts preserved; contact source withheld; no model called");
    }
    Ok(())
}
