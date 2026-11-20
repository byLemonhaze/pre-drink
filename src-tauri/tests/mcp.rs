#![cfg(feature = "desktop")]
use pre_drink::{
    broker,
    memory::MemoryInput,
    session::{Session, Store},
};
use serde_json::{json, Value};
use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, Command, Stdio},
    sync::{mpsc, Arc, Mutex},
    time::Duration,
};
struct Process(Child);
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn actual_stdio_client_negotiates_lists_tools_and_obeys_lock_and_revocation() {
    let dir = tempfile::Builder::new()
        .prefix("pd-mcp-")
        .tempdir_in("/tmp")
        .unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut session = Session::new(dir.path().into());
    session
        .unlock("a unique temporary password", true, false)
        .unwrap();
    let v = session.access().unwrap();
    let note = v
        .save_memory(MemoryInput {
            id: None,
            kind: "fact".into(),
            content: "Pre-Drink is local.".into(),
            source_message_id: None,
            private: false,
        })
        .unwrap();
    let review = v.approval_review("memory", &note).unwrap();
    v.approve_review("memory", &note, &review.review_hash)
        .unwrap();
    for (content, private) in [
        ("Pre-Drink fictional draft marker", false),
        ("Pre-Drink fictional private marker", true),
        ("Pre-Drink fictional sensitive person@example.org", false),
    ] {
        v.save_memory(MemoryInput {
            id: None,
            kind: "fact".into(),
            content: content.into(),
            source_message_id: None,
            private,
        })
        .unwrap();
    }
    let client = v.create_client("Isolated MCP check", false).unwrap();
    let store: Store = Arc::new(Mutex::new(session));
    let socket = dir.path().join("agents.sock");
    broker::serve(&socket, store.clone()).unwrap();
    let mut process = Process(
        Command::new(env!("CARGO_BIN_EXE_pre-drink"))
            .arg("--mcp")
            .env("PRE_DRINK_SOCKET", &socket)
            .env("PRE_DRINK_CLIENT_ID", &client.client.id)
            .env("PRE_DRINK_CLIENT_TOKEN", &client.token)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let stdout = process.0.stdout.take().unwrap();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            if tx.send(line.unwrap()).is_err() {
                break;
            }
        }
    });
    let input = process.0.stdin.as_mut().unwrap();
    let mut send = |message: Value| {
        writeln!(input, "{message}").unwrap();
        input.flush().unwrap();
    };
    let receive = || -> Value {
        serde_json::from_str(
            &rx.recv_timeout(Duration::from_secs(8))
                .expect("MCP response timeout"),
        )
        .unwrap()
    };
    send(
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"local-test","version":"1"}}}),
    );
    let initialized = receive();
    assert_eq!(initialized["result"]["serverInfo"]["name"], "pre-drink");
    send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
    send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}));
    assert_eq!(receive()["result"]["tools"].as_array().unwrap().len(), 7);
    send(
        json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_facts","arguments":{}}}),
    );
    assert_eq!(receive()["result"]["isError"], true);
    store.lock().unwrap().sharing = true;
    send(
        json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"get_facts","arguments":{}}}),
    );
    let facts = receive();
    assert_ne!(facts["result"]["isError"], true);
    assert!(facts.to_string().contains("Pre-Drink is local"));
    assert!(!facts.to_string().contains("fictional"));
    send(
        json!({"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"search_conversations","arguments":{"query":"Pre-Drink"}}}),
    );
    assert_eq!(receive()["result"]["isError"], true);
    // Lock while the credential is still valid: revocation must not mask a lock failure.
    store.lock().unwrap().lock();
    send(
        json!({"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"get_facts","arguments":{}}}),
    );
    assert_eq!(receive()["result"]["isError"], true);
    store
        .lock()
        .unwrap()
        .unlock("a unique temporary password", false, false)
        .unwrap();
    send(
        json!({"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"get_facts","arguments":{}}}),
    );
    assert_eq!(receive()["result"]["isError"], true);
    store.lock().unwrap().sharing = true;
    send(
        json!({"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"get_context_pack","arguments":{"topic":"Pre-Drink"}}}),
    );
    let pack = receive();
    assert_ne!(pack["result"]["isError"], true);
    assert!(pack.to_string().contains("Pre-Drink is local"));
    assert!(!pack.to_string().contains("fictional"));
    store
        .lock()
        .unwrap()
        .access()
        .unwrap()
        .save_memory(MemoryInput {
            id: Some(note),
            kind: "fact".into(),
            content: "Pre-Drink fictional edited marker".into(),
            source_message_id: None,
            private: false,
        })
        .unwrap();
    send(
        json!({"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"get_facts","arguments":{}}}),
    );
    let edited = receive();
    assert_ne!(edited["result"]["isError"], true);
    let payload: Value =
        serde_json::from_str(edited["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(payload, json!([]));
    store
        .lock()
        .unwrap()
        .access()
        .unwrap()
        .revoke_client(&client.client.id)
        .unwrap();
    send(
        json!({"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"get_facts","arguments":{}}}),
    );
    assert_eq!(receive()["result"]["isError"], true);
    store.lock().unwrap().lock();
    store
        .lock()
        .unwrap()
        .unlock("a unique temporary password", false, false)
        .unwrap();
    // Re-enable sharing deliberately so this denial proves persisted revocation.
    store.lock().unwrap().sharing = true;
    send(
        json!({"jsonrpc":"2.0","id":11,"method":"tools/call","params":{"name":"get_facts","arguments":{}}}),
    );
    assert_eq!(receive()["result"]["isError"], true);
}

#[test]
fn actual_stdio_curator_writes_retries_and_stops_on_lock() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::Builder::new()
        .prefix("pd-curation-mcp-")
        .tempdir_in("/tmp")
        .unwrap();
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut session = Session::new(dir.path().into());
    session
        .unlock("fictional curation transport phrase", true, false)
        .unwrap();
    let client = session
        .access()
        .unwrap()
        .create_scoped_client("Fictional Codex", Some("chatgpt"), true)
        .unwrap();
    session.sharing = true;
    let store: Store = Arc::new(Mutex::new(session));
    let socket = dir.path().join("agents.sock");
    broker::serve(&socket, store.clone()).unwrap();
    let mut process = Process(
        Command::new(env!("CARGO_BIN_EXE_pre-drink"))
            .arg("--mcp")
            .env("PRE_DRINK_SOCKET", &socket)
            .env("PRE_DRINK_CLIENT_ID", &client.client.id)
            .env("PRE_DRINK_CLIENT_TOKEN", &client.token)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let stdout = process.0.stdout.take().unwrap();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            if tx.send(line.unwrap()).is_err() {
                break;
            }
        }
    });
    let input = process.0.stdin.as_mut().unwrap();
    let mut send = |message: Value| {
        writeln!(input, "{message}").unwrap();
        input.flush().unwrap();
    };
    let receive = || -> Value {
        serde_json::from_str(
            &rx.recv_timeout(Duration::from_secs(8))
                .expect("MCP response timeout"),
        )
        .unwrap()
    };
    send(
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"fictional-curator","version":"1"}}}),
    );
    assert!(receive()["error"].is_null());
    send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
    send(
        json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"curate_memory","arguments":{"operation":{"action":"status"}}}}),
    );
    let response = receive();
    assert_ne!(response["result"]["isError"], true);
    assert!(response.to_string().contains("chatgpt"));
    let input = json!({"request_key":"fictional-session-one","id":null,"expected_revision":0,"kind":"project","content":"Fictional garden uses ceramic planters.","origin":"session","session_label":"Fictional garden","sources":[],"conflict":false});
    let write = json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"curate_memory","arguments":{"operation":{"action":"write","input":input}}}});
    send(write.clone());
    let first = receive();
    assert_ne!(first["result"]["isError"], true);
    send(write);
    assert_eq!(first, receive());
    let result: Value =
        serde_json::from_str(first["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(result["revision"], 1);
    assert_eq!(result["published"], false);
    let note_id = result["id"].as_str().unwrap().to_string();
    {
        let mut locked = store.lock().unwrap();
        let vault = locked.access().unwrap();
        let review = vault
            .curation_review(std::slice::from_ref(&note_id))
            .unwrap();
        vault
            .publish_curation_review(&[note_id], &review.review_hash)
            .unwrap();
    }
    send(
        json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"get_projects","arguments":{}}}),
    );
    let shared = receive();
    assert!(shared.to_string().contains("owner_review"));
    assert!(shared.to_string().contains("ceramic"));
    // Lock must block both reads and writes using the otherwise-valid credential.
    store.lock().unwrap().lock();
    send(
        json!({"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"curate_memory","arguments":{"operation":{"action":"write","input":input}}}}),
    );
    assert_eq!(receive()["result"]["isError"], true);
}
