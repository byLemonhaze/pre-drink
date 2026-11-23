//! Encrypted, bounded owner-visible metadata. Never store request arguments,
//! response/error text, source IDs, curation bodies, or credentials here.
use crate::{db::Vault, session::Session};
use rusqlite::{params, Connection};
use serde::Serialize;

pub const ACTIVITY_LIMIT: i64 = 1000;
#[derive(Serialize)]
pub struct AgentActivity {
    pub id: i64,
    pub client_name: String,
    pub occurred_at: f64,
    pub operation: String,
    pub outcome: String,
}
#[derive(Serialize)]
pub struct ActivityPage {
    pub items: Vec<AgentActivity>,
    pub next_before: Option<i64>,
    pub session_generation: u64,
}

pub(crate) fn record(
    db: &Connection,
    client_id: Option<&str>,
    client_name: &str,
    operation: &str,
    outcome: &str,
) -> Result<i64, String> {
    db.execute("INSERT INTO agent_activity(client_id,client_name,occurred_at,operation,outcome) VALUES(?1,?2,unixepoch('subsec'),?3,?4)", params![client_id,client_name,operation,outcome])
        .map_err(|_| "Could not record agent activity. No new request was started.")?;
    let id = db.last_insert_rowid();
    db.execute("DELETE FROM agent_activity WHERE id IN (SELECT id FROM agent_activity ORDER BY id DESC LIMIT -1 OFFSET ?1)", [ACTIVITY_LIMIT])
        .map_err(|_| "Could not maintain agent activity history.")?;
    Ok(id)
}

impl Vault {
    pub fn set_client_paused(&self, id: &str, paused: bool) -> Result<(), String> {
        let tx = self
            .connection
            .unchecked_transaction()
            .map_err(|e| e.to_string())?;
        let (name, previous): (String,bool) = tx.query_row("SELECT a.name,COALESCE(c.paused,0) FROM agent_clients a LEFT JOIN agent_controls c ON c.client_id=a.id WHERE a.id=?1", [id], |r| Ok((r.get(0)?,r.get(1)?)))
            .map_err(|_| "This connection no longer exists.")?;
        if previous == paused {
            return Ok(());
        }
        tx.execute("INSERT INTO agent_controls(client_id,paused) VALUES(?1,?2) ON CONFLICT(client_id) DO UPDATE SET paused=excluded.paused", params![id,paused]).map_err(|e| e.to_string())?;
        record(
            &tx,
            Some(id),
            &name,
            if paused {
                "client_paused"
            } else {
                "client_resumed"
            },
            "completed",
        )?;
        tx.commit().map_err(|e| e.to_string())
    }
    pub(crate) fn begin_agent_activity(&self, client_id: &str, tool: &str) -> Result<i64, String> {
        let operation = match tool {
            "connection_status"
            | "get_facts"
            | "get_decisions"
            | "get_projects"
            | "get_context_pack"
            | "search_conversations"
            | "mark_private"
            | "curate_memory" => tool,
            _ => "unknown_tool",
        };
        let tx = self
            .connection
            .unchecked_transaction()
            .map_err(|e| e.to_string())?;
        let name: String = tx
            .query_row(
                "SELECT name FROM agent_clients WHERE id=?1",
                [client_id],
                |r| r.get(0),
            )
            .map_err(|_| "Client access was revoked.")?;
        let event = record(&tx, Some(client_id), &name, operation, "started")?;
        tx.execute("INSERT INTO agent_controls(client_id,last_request_at) VALUES(?1,unixepoch('subsec')) ON CONFLICT(client_id) DO UPDATE SET last_request_at=excluded.last_request_at", [client_id]).map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(event)
    }
    pub(crate) fn finish_agent_activity(&self, event: i64, outcome: &str) -> Result<(), String> {
        self.connection.execute("UPDATE agent_activity SET outcome=?1 WHERE id=?2", params![outcome,event])
            .map_err(|_| "The request may have run, but its outcome could not be recorded. Check activity before retrying.")?;
        Ok(())
    }
    pub fn agent_activity(&self, before: Option<i64>) -> Result<Vec<AgentActivity>, String> {
        if before.is_some_and(|id| id <= 0) {
            return Err("Invalid activity page.".into());
        }
        let mut stmt = self.connection.prepare("SELECT id,client_name,occurred_at,operation,outcome FROM agent_activity WHERE (?1 IS NULL OR id<?1) ORDER BY id DESC LIMIT 51").map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([before], |r| {
                Ok(AgentActivity {
                    id: r.get(0)?,
                    client_name: r.get(1)?,
                    occurred_at: r.get(2)?,
                    operation: r.get(3)?,
                    outcome: r.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string());
        rows
    }
    pub fn clear_agent_activity(&self) -> Result<(), String> {
        let tx = self
            .connection
            .unchecked_transaction()
            .map_err(|e| e.to_string())?;
        tx.execute_batch(
            "DELETE FROM agent_activity; UPDATE agent_controls SET last_request_at=NULL;",
        )
        .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())
    }
    pub fn revoke_all_clients(&self, expected_ids: &[String]) -> Result<(), String> {
        let tx = self
            .connection
            .unchecked_transaction()
            .map_err(|e| e.to_string())?;
        let current = self.clients()?;
        let mut actual: Vec<_> = current.iter().map(|c| c.id.clone()).collect();
        actual.sort();
        let mut expected = expected_ids.to_vec();
        expected.sort();
        if actual != expected {
            return Err(
                "Connections changed. Review the current list before revoking all access.".into(),
            );
        }
        if actual.is_empty() {
            return Err("There are no connections to revoke.".into());
        }
        record(&tx, None, "All connections", "revoke_all", "completed")?;
        tx.execute("DELETE FROM agent_clients", [])
            .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())
    }
}

impl Session {
    pub fn activity_page(&mut self, before: Option<i64>) -> Result<ActivityPage, String> {
        let mut items = self.access()?.agent_activity(before)?;
        let more = items.len() > 50;
        items.truncate(50);
        let next_before = if more {
            items.last().map(|i| i.id)
        } else {
            None
        };
        self.touch();
        Ok(ActivityPage {
            items,
            next_before,
            session_generation: self.generation,
        })
    }
    pub fn pause_client(&mut self, id: &str, paused: bool) -> Result<(), String> {
        if let Err(error) = self.access()?.set_client_paused(id, paused) {
            if paused {
                self.stop_sharing();
                return Err(format!("Could not save this pause. Agent sharing was stopped for all connections. {error}"));
            }
            return Err(error);
        }
        self.successful_clients.remove(id);
        self.touch();
        Ok(())
    }
    pub fn clear_activity(&mut self, generation: u64) -> Result<(), String> {
        self.access()?;
        if self.generation != generation {
            return Err("The vault session changed. Review its activity again.".into());
        }
        self.access()?.clear_agent_activity()?;
        self.successful_clients.clear();
        self.touch();
        Ok(())
    }
    pub fn revoke_registered_client(&mut self, id: &str) -> Result<(), String> {
        if let Err(error) = self.access()?.revoke_client(id) {
            self.stop_sharing();
            return Err(format!("Could not revoke this connection. Agent sharing was stopped for all connections. {error}"));
        }
        self.pending_client_setup = None;
        self.successful_clients.remove(id);
        self.touch();
        Ok(())
    }
    pub fn revoke_all(&mut self, expected_ids: &[String]) -> Result<(), String> {
        self.access()?;
        // Count the verified owner action before ending an extended work session.
        // Otherwise the next access() can expire on the old idle timestamp and
        // leave every credential registered. Expired sessions still fail above.
        self.touch();
        self.stop_sharing();
        self.pending_client_setup = None;
        self.access()?
            .revoke_all_clients(expected_ids)
            .map_err(|e| format!("Agent sharing is off. No connections were revoked: {e}"))?;
        self.pending_client_setup = None;
        self.successful_clients.clear();
        self.stop_sharing();
        self.touch();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        agents::{AgentRequest, NewClient},
        client_config::{create_setup, ClientFormat},
    };
    fn request(client: &NewClient, tool: &str) -> AgentRequest {
        AgentRequest {
            client_id: client.client.id.clone(),
            token: client.token.clone(),
            tool: tool.into(),
            query: "fictional-private-query-marker".into(),
            limit: None,
            conversation_id: "fictional-source-marker".into(),
            curation: None,
        }
    }
    #[test]
    fn pause_blocks_every_tool_and_resume_preserves_scope_and_other_clients() {
        let v = Vault::memory().unwrap();
        let c = v
            .create_scoped_client("Fictional Grok", Some("grok"), true)
            .unwrap();
        let other = v.create_client("Fictional shared reader", false).unwrap();
        v.set_client_paused(&c.client.id, true).unwrap();
        let n = v.agent_activity(None).unwrap().len();
        v.set_client_paused(&c.client.id, true).unwrap();
        assert_eq!(v.agent_activity(None).unwrap().len(), n);
        for tool in [
            "connection_status",
            "get_facts",
            "get_decisions",
            "get_projects",
            "get_context_pack",
            "search_conversations",
            "mark_private",
            "curate_memory",
        ] {
            assert!(v
                .agent_call(&request(&c, tool))
                .unwrap_err()
                .contains("paused"));
        }
        assert!(v.agent_call(&request(&other, "get_facts")).is_ok());
        v.set_client_paused(&c.client.id, false).unwrap();
        let status = v.agent_call(&request(&c, "connection_status")).unwrap();
        assert_eq!(status["provider"], "grok");
        assert_eq!(status["can_curate"], true);
        let row = v
            .clients()
            .unwrap()
            .into_iter()
            .find(|x| x.id == c.client.id)
            .unwrap();
        assert!(!row.paused && row.last_request_at.is_some());
        assert!(v.set_client_paused("missing", false).is_err());
        v.verify().unwrap();
    }
    #[test]
    fn history_never_stores_arguments_credentials_or_unknown_tool_text() {
        let v = Vault::memory().unwrap();
        let c = v.create_client("Fictional reader", false).unwrap();
        let before = v.agent_activity(None).unwrap().len();
        let mut bad = request(&c, "get_facts");
        bad.token = "x".repeat(64);
        assert!(v.agent_call(&bad).is_err());
        assert_eq!(v.agent_activity(None).unwrap().len(), before);
        assert!(v
            .agent_call(&request(&c, "fictional-unknown-tool-marker"))
            .is_err());
        assert!(v.agent_call(&request(&c, "get_context_pack")).is_ok());
        let text = serde_json::to_string(&v.agent_activity(None).unwrap()).unwrap();
        for secret in [
            &c.token,
            c.client.id.as_str(),
            "fictional-private-query-marker",
            "fictional-source-marker",
            "fictional-unknown-tool-marker",
        ] {
            assert!(!text.contains(secret));
        }
        assert!(
            text.contains("unknown_tool") && text.contains("failed") && text.contains("completed")
        );
        let columns: String = v
            .connection
            .query_row(
                "SELECT sql FROM sqlite_master WHERE name='agent_activity'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        for forbidden in ["token", "query", "source_id", "content", "error"] {
            assert!(!columns.contains(forbidden));
        }
    }
    #[test]
    fn failed_recording_never_runs_a_request_or_changes_pause_and_revocation() {
        let v = Vault::memory().unwrap();
        let c = v.create_client("Fictional reader", false).unwrap();
        v.connection.execute_batch("CREATE TRIGGER fail_activity BEFORE INSERT ON agent_activity BEGIN SELECT RAISE(ABORT,'fictional failure'); END;").unwrap();
        assert!(v
            .agent_call(&request(&c, "get_facts"))
            .unwrap_err()
            .contains("No new request"));
        assert!(v.set_client_paused(&c.client.id, true).is_err());
        assert!(!v.clients().unwrap()[0].paused);
        assert!(v.revoke_client(&c.client.id).is_err());
        assert_eq!(v.clients().unwrap().len(), 1);
        v.connection.execute_batch("DROP TRIGGER fail_activity; CREATE TRIGGER fail_outcome BEFORE UPDATE ON agent_activity BEGIN SELECT RAISE(ABORT,'fictional failure'); END;").unwrap();
        assert!(v
            .agent_call(&request(&c, "get_facts"))
            .unwrap_err()
            .contains("may have run"));
        assert_eq!(v.agent_activity(None).unwrap()[0].outcome, "started");
        v.connection
            .execute_batch("DROP TRIGGER fail_outcome;")
            .unwrap();
        assert!(v.agent_call(&request(&c, "get_facts")).is_ok());
    }
    #[test]
    fn bounded_history_paginates_and_clear_preserves_credentials_and_pause() {
        let v = Vault::memory().unwrap();
        let c = v.create_client("Fictional reader", false).unwrap();
        let tx = v.connection.unchecked_transaction().unwrap();
        for _ in 0..1050 {
            record(
                &tx,
                Some(&c.client.id),
                "Fictional reader",
                "get_facts",
                "completed",
            )
            .unwrap();
        }
        tx.commit().unwrap();
        assert_eq!(
            v.connection
                .query_row("SELECT count(*) FROM agent_activity", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            1000
        );
        v.set_client_paused(&c.client.id, true).unwrap();
        assert!(v.agent_call(&request(&c, "get_facts")).is_err());
        let mut s = Session::new(std::path::PathBuf::new());
        s.vault = Some(v);
        let mut before = None;
        let mut ids = Vec::new();
        loop {
            let page = s.activity_page(before).unwrap();
            ids.extend(page.items.iter().map(|x| x.id));
            before = page.next_before;
            if before.is_none() {
                break;
            }
        }
        assert_eq!(ids.len(), 1000);
        assert!(ids.windows(2).all(|p| p[0] > p[1]));
        assert!(s.clear_activity(s.generation + 1).is_err());
        s.clear_activity(s.generation).unwrap();
        assert!(s.activity_page(None).unwrap().items.is_empty());
        let row = s.access().unwrap().clients().unwrap().remove(0);
        assert!(row.paused);
        assert!(row.last_request_at.is_none());
        s.pause_client(&c.client.id, false).unwrap();
        assert!(s
            .access()
            .unwrap()
            .agent_call(&request(&c, "get_facts"))
            .is_ok());
    }
    #[test]
    fn revoke_all_rejects_changed_lists_and_stops_sharing_without_erasing_history() {
        let v = Vault::memory().unwrap();
        let c = v.create_client("Fictional reader", false).unwrap();
        let mut s = Session::new(std::path::PathBuf::new());
        s.vault = Some(v);
        s.start_agent_session("until_lock").unwrap();
        let setup = create_setup(
            s.access().unwrap(),
            "Fictional pending setup",
            false,
            false,
            ClientFormat::Codex,
            std::path::Path::new("/app/pre-drink"),
            std::path::Path::new("/tmp/fictional.sock"),
        )
        .unwrap();
        let second = setup.client.id.clone();
        let prepared = s.prepare_client_settings(setup, "toml");
        assert!(s.revoke_all(std::slice::from_ref(&c.client.id)).is_err());
        assert!(!s.sharing);
        s.start_agent_session("until_lock").unwrap();
        s.successful_clients.insert(c.client.id.clone());
        s.revoke_all(&[c.client.id.clone(), second]).unwrap();
        assert!(!s.sharing && s.successful_clients.is_empty() && s.pending_client_setup.is_none());
        assert!(s.access().unwrap().clients().unwrap().is_empty());
        assert!(s
            .access()
            .unwrap()
            .agent_call(&request(&c, "get_facts"))
            .is_err());
        assert!(s
            .with_client_settings(&prepared.request, |_, _| -> Result<(), String> {
                panic!("revoked settings delivered")
            })
            .is_err());
        assert!(s
            .activity_page(None)
            .unwrap()
            .items
            .iter()
            .any(|x| x.operation == "revoke_all"));
        s.access().unwrap().verify().unwrap();
    }
    #[test]
    fn encrypted_reopen_backup_and_restore_preserve_activity_without_restoring_grants() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("fictional.sqlite3");
        let backup = dir.path().join("fictional-backup.sqlite3");
        let password = "fictional activity garden phrase";
        let v = Vault::create(&path, password).unwrap();
        let c = v
            .create_client("Fictional private activity label", false)
            .unwrap();
        v.agent_call(&request(&c, "get_facts")).unwrap();
        v.set_client_paused(&c.client.id, true).unwrap();
        v.encrypted_backup(&backup, password).unwrap();
        v.checkpoint().unwrap();
        drop(v);
        for p in [&path, &backup] {
            assert!(!std::fs::read(p)
                .unwrap()
                .windows(b"Fictional private activity label".len())
                .any(|x| x == b"Fictional private activity label"));
            let v = Vault::open(p, password).unwrap();
            assert!(v.clients().unwrap()[0].paused);
            assert!(v.clients().unwrap()[0].last_request_at.is_some());
            assert_eq!(v.agent_activity(None).unwrap().len(), 3);
        }
        let restored_dir = dir.path().join("restored");
        std::fs::create_dir(&restored_dir).unwrap();
        let mut s = Session::new(restored_dir);
        let prepared = s.prepare_restore(&backup, password).unwrap();
        s.apply_replacement(&prepared.review_id, password).unwrap();
        s.unlock(password, false, false).unwrap();
        assert!(s.access().unwrap().clients().unwrap().is_empty());
        assert!(!s.sharing);
        assert_eq!(s.activity_page(None).unwrap().items.len(), 3);
    }
}

#[cfg(test)]
mod fail_safe_tests {
    use super::*;
    #[test]
    fn revoke_all_finishes_after_a_long_idle_work_session_without_reviving_expired_sessions() {
        for work_session in [true, false] {
            let mut s = Session::new(std::path::PathBuf::new());
            s.vault = Some(Vault::memory().unwrap());
            let client = s
                .access()
                .unwrap()
                .create_client("Fictional reader", false)
                .unwrap();
            if work_session {
                s.start_agent_session("until_lock").unwrap();
            }
            s.last_activity = std::time::SystemTime::now() - crate::session::IDLE_TIMEOUT;
            let result = s.revoke_all(std::slice::from_ref(&client.client.id));
            assert!(!s.sharing);
            if work_session {
                result.expect(
                    "An authorized owner revocation must finish before normal idle expiry resumes",
                );
                assert!(s.access().unwrap().clients().unwrap().is_empty());
                assert!(s.status().unwrap().agent_session.is_none());
            } else {
                assert!(result.is_err());
                assert!(
                    s.vault.is_none(),
                    "An expired session must not be revived by revocation"
                );
            }
        }
    }

    #[test]
    fn unsaved_pause_and_revocation_stop_global_sharing() {
        for operation in ["pause", "revoke", "all"] {
            let v = Vault::memory().unwrap();
            let client = v.create_client("Fictional reader", false).unwrap();
            v.connection.execute_batch("CREATE TRIGGER fail_activity BEFORE INSERT ON agent_activity BEGIN SELECT RAISE(ABORT,'fictional full disk'); END;").unwrap();
            let mut s = Session::new(std::path::PathBuf::new());
            s.vault = Some(v);
            s.start_agent_session("until_lock").unwrap();
            let result = match operation {
                "pause" => s.pause_client(&client.client.id, true),
                "revoke" => s.revoke_registered_client(&client.client.id),
                _ => s.revoke_all(std::slice::from_ref(&client.client.id)),
            };
            assert!(result.is_err());
            assert!(!s.sharing);
            let clients = s.access().unwrap().clients().unwrap();
            assert_eq!(clients.len(), 1);
            assert!(!clients[0].paused);
        }
    }
}
