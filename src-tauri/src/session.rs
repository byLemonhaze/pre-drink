use crate::db::Vault;
use serde::Serialize;
use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime},
};

pub type Store = Arc<Mutex<Session>>;
pub const IDLE_TIMEOUT: Duration = Duration::from_secs(15 * 60);

pub struct AgentWorkSession {
    started: Instant,
    duration: Option<Duration>,
}

#[derive(Serialize)]
pub struct AgentSessionStatus {
    /// None means the owner's explicit "until Mac locks" choice.
    pub remaining_seconds: Option<u64>,
}

#[derive(Serialize)]
pub struct VaultStatus {
    pub unlocked: bool,
    pub exists: bool,
    pub legacy_exists: bool,
    pub sharing: bool,
    pub agent_session: Option<AgentSessionStatus>,
    pub privacy_on_open: crate::privacy_recheck::PrivacyUpdate,
    pub database_path: String,
    pub app_version: &'static str,
    pub build_revision: &'static str,
    pub built_at: &'static str,
    pub build_profile: &'static str,
    pub isolated_storage: bool,
}

#[derive(Serialize)]
pub struct ClientConnection {
    pub request_seen: bool,
    pub sharing: bool,
}

pub struct Session {
    pub vault: Option<Vault>,
    pub(crate) successful_clients: HashSet<String>,
    pub(crate) pending_replacement: Option<crate::lifecycle::PendingReplacement>,
    pub(crate) backup_receipt: Option<crate::retained_vaults::BackupReceipt>,
    pub directory: PathBuf,
    pub isolated_storage: bool,
    pub sharing: bool,
    agent_session: Option<AgentWorkSession>,
    pub generation: u64,
    pub last_activity: SystemTime,
    pub on_lock: Option<Box<dyn Fn() + Send + Sync>>,
}

impl Session {
    pub fn new(directory: PathBuf) -> Self {
        Self {
            vault: None,
            successful_clients: HashSet::new(),
            pending_replacement: None,
            backup_receipt: None,
            directory,
            isolated_storage: false,
            sharing: false,
            agent_session: None,
            generation: 0,
            last_activity: SystemTime::now(),
            on_lock: None,
        }
    }
    pub fn path(&self) -> Result<PathBuf, String> {
        crate::lifecycle::active_path(&self.directory)
    }
    pub fn legacy_path(&self) -> PathBuf {
        self.directory.join("vault.sqlite3")
    }
    pub fn status(&mut self) -> Result<VaultStatus, String> {
        self.expire();
        let path = self.path()?;
        Ok(VaultStatus {
            unlocked: self.vault.is_some(),
            exists: path.exists(),
            legacy_exists: self.legacy_path().exists(),
            sharing: self.sharing,
            agent_session: self
                .agent_session
                .as_ref()
                .map(|session| AgentSessionStatus {
                    remaining_seconds: session
                        .duration
                        .map(|d| d.saturating_sub(session.started.elapsed()).as_secs()),
                }),
            privacy_on_open: self
                .vault
                .as_ref()
                .map(|v| v.privacy_on_open.clone())
                .unwrap_or_default(),
            database_path: path.to_string_lossy().into_owned(),
            app_version: env!("CARGO_PKG_VERSION"),
            build_revision: env!("PRE_DRINK_BUILD_REVISION"),
            built_at: env!("PRE_DRINK_BUILT_AT"),
            build_profile: if cfg!(debug_assertions) {
                "Development build"
            } else {
                "Release build"
            },
            isolated_storage: self.isolated_storage,
        })
    }
    pub fn lock(&mut self) {
        self.agent_session = None;
        self.successful_clients.clear();
        self.backup_receipt = None;
        let prepared = self.pending_replacement.is_some();
        self.cancel_replacement();
        if self.vault.is_none() && !self.sharing && !prepared {
            return;
        }
        if let Some(callback) = &self.on_lock {
            callback();
        }
        self.stop_sharing();
        self.generation = self.generation.wrapping_add(1);
        self.vault.take();
    }
    pub fn expire(&mut self) {
        if !self.sharing {
            self.agent_session = None;
        }
        if let Some(work) = &self.agent_session {
            if work.duration.is_some_and(|d| work.started.elapsed() >= d) {
                self.lock();
            }
            return;
        }
        if self.last_activity.elapsed().unwrap_or(IDLE_TIMEOUT) >= IDLE_TIMEOUT {
            self.lock();
        }
    }
    /// Owner-only command. Agents cannot start, extend, or restore this grant.
    pub fn start_agent_session(&mut self, mode: &str) -> Result<(), String> {
        let duration = match mode {
            "one_hour" => Some(Duration::from_secs(3600)),
            "four_hours" => Some(Duration::from_secs(4 * 3600)),
            "until_lock" => None,
            _ => return Err("Choose a supported agent session duration.".into()),
        };
        self.access()?;
        if self.pending_replacement.is_some() {
            return Err(
                "Finish or cancel vault maintenance before starting an agent session.".into(),
            );
        }
        self.agent_session = Some(AgentWorkSession {
            started: Instant::now(),
            duration,
        });
        self.sharing = true;
        self.touch();
        Ok(())
    }
    pub fn stop_sharing(&mut self) {
        self.sharing = false;
        self.agent_session = None;
    }
    pub fn access(&mut self) -> Result<&mut Vault, String> {
        self.expire();
        self.vault
            .as_mut()
            .ok_or_else(|| "Your vault is locked. Unlock it in Pre-Drink.".into())
    }
    pub fn apply_privacy_recheck(
        &mut self,
        review_hash: &str,
        clear_ids: &[String],
    ) -> Result<crate::privacy_recheck::PrivacyRecheck, String> {
        self.stop_sharing();
        let result = self.access()?.apply_privacy_recheck(review_hash, clear_ids);
        self.touch();
        result
    }
    // Owner-only setup evidence, not a live connection or model-identity claim.
    // No tool arguments, returned content, timestamps or tokens are retained.
    pub fn client_connection(&mut self, id: &str) -> Result<ClientConnection, String> {
        if !self
            .access()?
            .clients()?
            .iter()
            .any(|client| client.id == id)
        {
            return Err("This client was removed. Create a new connection if needed.".into());
        }
        Ok(ClientConnection {
            request_seen: self.successful_clients.contains(id),
            sharing: self.sharing,
        })
    }

    pub fn touch(&mut self) {
        self.last_activity = SystemTime::now();
    }
    pub fn unlock(
        &mut self,
        password: &str,
        create: bool,
        remove_legacy: bool,
    ) -> Result<(), String> {
        if self.vault.is_some() {
            return Err("The vault is already unlocked.".into());
        }
        if password.len() > 1024 {
            return Err("Password is too long.".into());
        }
        let path = self.path()?;
        let vault = if create {
            if path.exists() {
                return Err("A vault already exists. Unlock it with its password.".into());
            }
            if self.legacy_path().exists() {
                Vault::migrate_plaintext(&self.legacy_path(), &path, password)?
            } else {
                Vault::create(&path, password)?
            }
        } else {
            Vault::open(&path, password)?
        };
        vault.verify()?;
        if remove_legacy && self.legacy_path().exists() {
            // Migration verifies the encrypted copy before removing the old duplicate.
            // SSD snapshots/backups may retain old plaintext; deletion is not secure erasure.
            if !create {
                return Err("Legacy cleanup must be completed as part of migration.".into());
            }
            std::fs::remove_file(self.legacy_path()).map_err(|_| "The encrypted vault is ready, but the old plaintext file couldn't be removed. Remove it manually after verifying your library.")?;
            for suffix in ["-wal", "-shm"] {
                let old = self.directory.join(format!("vault.sqlite3{suffix}"));
                if old.exists() {
                    std::fs::remove_file(old)
                        .map_err(|_| "An old plaintext journal couldn't be removed.")?;
                }
            }
        }
        self.successful_clients.clear();
        self.vault = Some(vault);
        self.touch();
        self.generation = self.generation.wrapping_add(1);
        self.stop_sharing();
        Ok(())
    }
}

pub fn watchdog(store: Store) {
    std::thread::spawn(move || {
        let mut last_tick = SystemTime::now();
        loop {
            std::thread::sleep(Duration::from_secs(3));
            if let Ok(mut session) = store.lock() {
                // A sleep/resume gap locks even if no frontend event runs while asleep.
                if last_tick.elapsed().unwrap_or(Duration::MAX) > Duration::from_secs(30) {
                    session.lock();
                } else {
                    session.expire();
                }
            }
            last_tick = SystemTime::now();
        }
    });
}

#[cfg(test)]
mod agent_session_tests {
    use super::*;
    fn unlocked() -> Session {
        let mut s = Session::new(PathBuf::new());
        s.vault = Some(Vault::memory().unwrap());
        s
    }
    #[test]
    fn longer_session_is_explicit_bounded_and_not_extended_by_activity() {
        let mut s = unlocked();
        assert!(s.start_agent_session("forever").is_err());
        assert!(!s.sharing);
        s.start_agent_session("one_hour").unwrap();
        s.last_activity = SystemTime::now() - Duration::from_secs(1800);
        assert!(s.access().is_ok());
        s.agent_session.as_mut().unwrap().started = Instant::now() - Duration::from_secs(3601);
        s.touch(); // Owner activity also cannot silently extend the explicit deadline.
        assert!(s.access().is_err());
        assert!(!s.sharing);
        assert!(s.agent_session.is_none());
        assert!(s.start_agent_session("four_hours").is_err());
    }
    #[test]
    fn lock_and_stop_sharing_cancel_until_lock_session() {
        let mut s = unlocked();
        s.start_agent_session("until_lock").unwrap();
        s.last_activity = SystemTime::now() - Duration::from_secs(86400);
        assert!(s.access().is_ok());
        s.stop_sharing();
        assert!(s.agent_session.is_none());
        assert!(s.access().is_err()); // Normal idle protection is restored immediately.
        let mut s = unlocked();
        s.start_agent_session("until_lock").unwrap();
        s.lock(); // Also used by macOS sleep and screen-lock observers.
        assert!(s.agent_session.is_none());
        assert!(s.vault.is_none());
        assert!(!s.sharing);
    }
    #[test]
    fn ordinary_sharing_keeps_default_idle_limit_and_status_is_not_activity() {
        let mut s = unlocked();
        s.sharing = true;
        s.last_activity = SystemTime::now() - IDLE_TIMEOUT;
        assert!(s.access().is_err());
        let mut s = unlocked();
        s.start_agent_session("four_hours").unwrap();
        let before = s.last_activity;
        let status = s.status().unwrap();
        assert!(status.agent_session.unwrap().remaining_seconds.unwrap() <= 14400);
        assert_eq!(before, s.last_activity);
    }
}
