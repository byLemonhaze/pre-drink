//! Owner-only Unix socket. It never receives a vault password or opens the database.
use crate::{agents::AgentRequest, session::Store};
use std::{
    io::{BufRead, BufReader, Write},
    os::unix::{
        fs::PermissionsExt,
        net::{UnixListener, UnixStream},
    },
    path::Path,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};
use zeroize::Zeroize;

pub fn serve(path: &Path, store: Store) -> Result<(), String> {
    // Caller holds the application's exclusive instance lock before replacing a stale socket.
    if path.exists() {
        std::fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    let listener =
        UnixListener::bind(path).map_err(|_| "Couldn't start the private agent connection.")?;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
        .map_err(|e| e.to_string())?;
    std::thread::spawn(move || {
        let active = Arc::new(AtomicUsize::new(0));
        for stream in listener.incoming().flatten() {
            if active.load(Ordering::SeqCst) >= 4 {
                continue;
            }
            active.fetch_add(1, Ordering::SeqCst);
            let active = active.clone();
            let store = store.clone();
            std::thread::spawn(move || {
                let _ = handle(stream, store);
                active.fetch_sub(1, Ordering::SeqCst);
            });
        }
    });
    Ok(())
}

fn handle(mut stream: UnixStream, store: Store) -> Result<(), String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    use std::io::Read;
    BufReader::new(&stream)
        .take(65_537)
        .read_until(b'\n', &mut bytes)
        .map_err(|_| "Invalid client request.")?;
    if bytes.len() > 65_536 || bytes.last() != Some(&b'\n') {
        return Err("Request is too large or incomplete.".into());
    }
    let parsed = serde_json::from_slice::<AgentRequest>(&bytes);
    bytes.zeroize();
    let mut request = parsed.map_err(|_| "Invalid client request.")?;
    let mut session = store.lock().map_err(|_| "Vault unavailable.")?;
    session.expire();
    let result = if !session.sharing {
        Err("Agent sharing is off. Enable it in the unlocked Pre-Drink app.".into())
    } else {
        session.access().and_then(|v| v.agent_call(&request))
    };
    request.token.zeroize();
    let reply = match result {
        Ok(data) => serde_json::json!({"ok":true,"data":data}),
        Err(error) => serde_json::json!({"ok":false,"error":error}),
    };
    let mut encoded = serde_json::to_vec(&reply).map_err(|_| "Couldn't encode reply.")?;
    if encoded.len() > 256_000 {
        encoded = br#"{"ok":false,"error":"Result is too large. Use a narrower context topic."}"#
            .to_vec();
    }
    // Keep the session guard through the write so locking/revocation cannot race a queued response.
    encoded.push(b'\n');
    let sent = stream
        .write_all(&encoded)
        .map_err(|_| "Client disconnected.".to_string());
    encoded.zeroize();
    sent
}
