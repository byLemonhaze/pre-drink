use crate::db::Vault;
use serde::Serialize;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};

pub type Store = Arc<Mutex<Session>>;
pub const IDLE_TIMEOUT: Duration = Duration::from_secs(15 * 60);

#[derive(Serialize)]
pub struct VaultStatus {
    pub unlocked: bool,
    pub exists: bool,
    pub legacy_exists: bool,
    pub sharing: bool,
    pub database_path: String,
    pub app_version: &'static str,
    pub build_revision: &'static str,
    pub built_at: &'static str,
    pub build_profile: &'static str,
    pub isolated_storage: bool,
}

pub struct Session {
    pub vault: Option<Vault>,
    pub(crate) pending_replacement: Option<crate::lifecycle::PendingReplacement>,
    pub directory: PathBuf,
    pub isolated_storage: bool,
    pub sharing: bool,
    pub generation: u64,
    pub last_activity: SystemTime,
    pub on_lock: Option<Box<dyn Fn() + Send + Sync>>,
}

impl Session {
    pub fn new(directory: PathBuf) -> Self {
        Self {
            vault: None,
            pending_replacement: None,
            directory,
            isolated_storage: false,
            sharing: false,
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
        let prepared = self.pending_replacement.is_some();
        self.cancel_replacement();
        if self.vault.is_none() && !self.sharing && !prepared {
            return;
        }
        if let Some(callback) = &self.on_lock {
            callback();
        }
        self.sharing = false;
        self.generation = self.generation.wrapping_add(1);
        self.vault.take();
    }
    pub fn expire(&mut self) {
        if self.last_activity.elapsed().unwrap_or(IDLE_TIMEOUT) >= IDLE_TIMEOUT {
            self.lock();
        }
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
    ) -> Result<crate::privacy_recheck::PrivacyRecheck, String> {
        self.sharing = false;
        let result = self.access()?.apply_privacy_recheck(review_hash);
        self.touch();
        result
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
        self.vault = Some(vault);
        self.touch();
        self.generation = self.generation.wrapping_add(1);
        self.sharing = false;
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
