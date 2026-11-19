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
    let mut session = Session::new(dir.path().into());
    session
        .unlock("a unique temporary password", true, false)
        .unwrap();
    let v = session.access().unwrap();
    v.save_memory(MemoryInput {
        id: None,
        kind: "fact".into(),
        content: "Pre-Drink is local.".into(),
        source_message_id: None,
        approved: true,
        private: false,
    })
    .unwrap();
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
    assert_eq!(receive()["result"]["tools"].as_array().unwrap().len(), 6);
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
    send(
        json!({"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"search_conversations","arguments":{"query":"Pre-Drink"}}}),
    );
    assert_eq!(receive()["result"]["isError"], true);
    store
        .lock()
        .unwrap()
        .access()
        .unwrap()
        .revoke_client(&client.client.id)
        .unwrap();
    send(
        json!({"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"get_facts","arguments":{}}}),
    );
    assert_eq!(receive()["result"]["isError"], true);
    store.lock().unwrap().lock();
    send(
        json!({"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"get_facts","arguments":{}}}),
    );
    assert_eq!(receive()["result"]["isError"], true);
}
