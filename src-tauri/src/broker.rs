//! Owner-only Unix socket. It never receives a vault password or opens the database.
use crate::{agents::AgentRequest, session::Store};
use std::{
    io::{Read, Write},
    os::unix::{
        fs::{FileTypeExt, MetadataExt, PermissionsExt},
        net::{UnixListener, UnixStream},
    },
    path::Path,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use zeroize::Zeroize;

pub fn serve(path: &Path, store: Store) -> Result<(), String> {
    // The parent protects the socket even during bind-before-chmod. Do not
    // temporarily change the process-wide umask in a multithreaded app.
    let parent = path.parent().ok_or("Missing private socket directory.")?;
    let metadata = std::fs::symlink_metadata(parent).map_err(|e| e.to_string())?;
    if !metadata.is_dir()
        || metadata.uid() != crate::peer::current_uid()
        || metadata.permissions().mode() & 0o077 != 0
    {
        return Err("The agent socket requires a private directory (0700).".into());
    }
    // Caller holds the application's exclusive instance lock before replacing a stale socket.
    if let Ok(existing) = std::fs::symlink_metadata(path) {
        if !existing.file_type().is_socket() || existing.uid() != crate::peer::current_uid() {
            return Err(
                "The agent socket path contains an unexpected file. Nothing was removed.".into(),
            );
        }
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

fn read_request(stream: &mut UnixStream) -> Result<zeroize::Zeroizing<Vec<u8>>, String> {
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut bytes = zeroize::Zeroizing::new(Vec::new());
    let mut chunk = zeroize::Zeroizing::new([0u8; 4096]);
    loop {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .filter(|d| !d.is_zero())
            .ok_or("Client request deadline exceeded.")?;
        stream
            .set_read_timeout(Some(remaining.min(Duration::from_secs(2))))
            .map_err(|e| e.to_string())?;
        let n = stream
            .read(&mut *chunk)
            .map_err(|_| "Invalid or timed-out client request.")?;
        if n == 0 {
            return Err("Incomplete client request.".into());
        }
        let end = chunk[..n].iter().position(|b| *b == b'\n').map(|i| i + 1);
        bytes.extend_from_slice(&chunk[..end.unwrap_or(n)]);
        if bytes.len() > 65_536 {
            return Err("Request is too large.".into());
        }
        if end.is_some() {
            return Ok(bytes);
        }
    }
}

fn handle(mut stream: UnixStream, store: Store) -> Result<(), String> {
    crate::peer::require_same_account(&stream)?;
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .map_err(|e| e.to_string())?;
    let mut bytes = read_request(&mut stream)?;
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
    let succeeded = result.is_ok();
    request.token.zeroize();
    let reply = match result {
        Ok(data) => serde_json::json!({"ok":true,"data":data}),
        Err(error) => serde_json::json!({"ok":false,"error":error}),
    };
    let mut encoded = serde_json::to_vec(&reply).map_err(|_| "Couldn't encode reply.")?;
    let within_limit = encoded.len() <= 256_000;
    if !within_limit {
        encoded.zeroize();
        encoded = br#"{"ok":false,"error":"Result is too large. Use a narrower context topic."}"#
            .to_vec();
    }
    // Keep the session guard through the write so locking/revocation cannot race a queued response.
    encoded.push(b'\n');
    let sent = stream
        .write_all(&encoded)
        .map_err(|_| "Client disconnected.".to_string());
    encoded.zeroize();
    if sent.is_ok() && succeeded && within_limit {
        session.successful_clients.insert(request.client_id);
    }
    sent
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_socket_cleanup_never_removes_regular_files_or_symlinks() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let path = directory.path().join("broker.sock");
        let target = directory.path().join("fictional.txt");
        std::fs::write(&target, "fictional content").unwrap();
        let store = Arc::new(std::sync::Mutex::new(crate::session::Session::new(
            directory.path().into(),
        )));
        std::fs::write(&path, "fictional content").unwrap();
        assert!(serve(&path, store.clone()).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "fictional content");
        std::fs::remove_file(&path).unwrap();
        std::os::unix::fs::symlink(&target, &path).unwrap();
        assert!(serve(&path, store).is_err());
        assert!(path.is_symlink());
        assert_eq!(
            std::fs::read_to_string(&target).unwrap(),
            "fictional content"
        );
    }

    #[test]
    fn setup_evidence_requires_authenticated_success_and_retains_no_content() {
        use std::io::{BufRead, BufReader};
        let vault = crate::db::Vault::memory().unwrap();
        let client = vault
            .create_scoped_client("Fictional Grok", Some("grok"), true)
            .unwrap();
        let mut session = crate::session::Session::new(std::path::PathBuf::new());
        session.vault = Some(vault);
        let store = Arc::new(std::sync::Mutex::new(session));
        let exchange = |token: &str, tool: &str| {
            let (server, mut socket) = UnixStream::pair().unwrap();
            let state = store.clone();
            let worker = std::thread::spawn(move || handle(server, state));
            writeln!(
                socket,
                "{}",
                serde_json::json!({"client_id":client.client.id,"token":token,"tool":tool})
            )
            .unwrap();
            let mut output = String::new();
            BufReader::new(socket).read_line(&mut output).unwrap();
            worker.join().unwrap().unwrap();
            serde_json::from_str::<serde_json::Value>(&output).unwrap()
        };
        assert_eq!(exchange(&client.token, "connection_status")["ok"], false);
        assert_eq!(
            store
                .lock()
                .unwrap()
                .access()
                .unwrap()
                .agent_activity(None)
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            store
                .lock()
                .unwrap()
                .access()
                .unwrap()
                .activity_totals()
                .unwrap()
                .denied,
            0
        );
        assert_eq!(exchange(&"x".repeat(64), "connection_status")["ok"], false);
        assert_eq!(
            store
                .lock()
                .unwrap()
                .access()
                .unwrap()
                .activity_totals()
                .unwrap()
                .denied,
            0
        );
        store.lock().unwrap().sharing = true;
        assert_eq!(exchange(&"x".repeat(64), "connection_status")["ok"], false);
        assert_eq!(
            store
                .lock()
                .unwrap()
                .access()
                .unwrap()
                .activity_totals()
                .unwrap()
                .denied,
            1
        );
        assert_eq!(
            store
                .lock()
                .unwrap()
                .access()
                .unwrap()
                .agent_activity(None)
                .unwrap()
                .len(),
            1
        );
        assert_eq!(exchange(&client.token, "nonexistent_tool")["ok"], false);
        assert!(
            !store
                .lock()
                .unwrap()
                .client_connection(&client.client.id)
                .unwrap()
                .request_seen
        );
        assert_eq!(
            exchange(&client.token, "connection_status"),
            serde_json::json!({"ok":true,"data":{
                "connected":true,"provider":"grok","can_curate":true,"allow_history":true
            }})
        );
        let mut s = store.lock().unwrap();
        assert!(s.client_connection(&client.client.id).unwrap().request_seen);
        assert_eq!(s.successful_clients.len(), 1);
        assert_eq!(s.access().unwrap().agent_activity(None).unwrap().len(), 3);
        s.pause_client(&client.client.id, true).unwrap();
        drop(s);
        assert_eq!(exchange(&client.token, "get_facts")["ok"], false);
        let mut s = store.lock().unwrap();
        assert!(!s.client_connection(&client.client.id).unwrap().request_seen);
        assert_eq!(
            s.access().unwrap().agent_activity(None).unwrap()[0].outcome,
            "paused"
        );
        s.pause_client(&client.client.id, false).unwrap();
        drop(s);
        assert_eq!(exchange(&client.token, "connection_status")["ok"], true);
        let mut s = store.lock().unwrap();
        s.sharing = false;
        let evidence = s.client_connection(&client.client.id).unwrap();
        assert!(evidence.request_seen && !evidence.sharing);
        s.access()
            .unwrap()
            .revoke_client(&client.client.id)
            .unwrap();
        assert!(s.client_connection(&client.client.id).is_err());
        s.lock();
        assert!(s.successful_clients.is_empty());
    }

    #[test]
    fn trickled_requests_cannot_extend_the_read_deadline() {
        let (mut server, mut client) = UnixStream::pair().unwrap();
        let writer = std::thread::spawn(move || {
            for _ in 0..12 {
                if client.write_all(b" ").is_err() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(400));
            }
        });
        let started = Instant::now();
        assert!(read_request(&mut server).is_err());
        assert!(started.elapsed() < Duration::from_secs(4));
        drop(server);
        writer.join().unwrap();
    }

    #[test]
    fn incomplete_and_oversized_requests_fail() {
        for payload in [b"{}".to_vec(), vec![b' '; 65_537]] {
            let (mut server, mut client) = UnixStream::pair().unwrap();
            let writer = std::thread::spawn(move || {
                let _ = client.write_all(&payload);
            });
            assert!(read_request(&mut server).is_err());
            drop(server);
            writer.join().unwrap();
        }
    }

    #[test]
    fn public_parent_directory_is_rejected_before_binding() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
        let store = Arc::new(std::sync::Mutex::new(crate::session::Session::new(
            dir.path().into(),
        )));
        let path = dir.path().join("agents.sock");
        assert!(serve(&path, store).is_err());
        assert!(!path.exists());
    }
}
