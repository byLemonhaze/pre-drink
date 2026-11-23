//! Owner-only, session-bound delivery of newly generated client settings.
use crate::{agents::AgentClient, client_config::ClientSetup, session::Session};
use serde::{Deserialize, Serialize};
use std::{io::Write, path::Path};
use zeroize::Zeroizing;

pub(crate) struct PendingClientSetup {
    id: String,
    client_id: String,
    extension: &'static str,
    configuration: Zeroizing<String>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct SetupRequest {
    pub setup_id: String,
    pub session_generation: u64,
}

#[derive(Serialize)]
pub struct PreparedClientSetup {
    pub client: AgentClient,
    pub request: SetupRequest,
}

impl Session {
    pub fn prepare_client_settings(
        &mut self,
        setup: ClientSetup,
        extension: &'static str,
    ) -> PreparedClientSetup {
        let id = uuid::Uuid::new_v4().to_string();
        self.pending_client_setup = Some(PendingClientSetup {
            id: id.clone(),
            client_id: setup.client.id.clone(),
            extension,
            configuration: Zeroizing::new(setup.configuration),
        });
        self.touch();
        PreparedClientSetup {
            client: setup.client,
            request: SetupRequest {
                setup_id: id,
                session_generation: self.generation,
            },
        }
    }

    pub fn discard_client_settings(&mut self, id: &str) {
        if self
            .pending_client_setup
            .as_ref()
            .is_some_and(|p| p.id == id)
        {
            self.pending_client_setup = None;
        }
    }

    /// The owner command holds the session mutex through delivery. No caller-
    /// supplied token/configuration can be written, and delayed dialogs must
    /// recheck expiry, lock/replacement, setup identity and registration.
    pub fn with_client_settings<T>(
        &mut self,
        request: &SetupRequest,
        deliver: impl FnOnce(&str, &str) -> Result<T, String>,
    ) -> Result<T, String> {
        self.access()?;
        if request.session_generation != self.generation {
            return Err("The vault session changed. Create new private settings.".into());
        }
        let pending = self.pending_client_setup.as_ref().filter(|p| p.id == request.setup_id)
            .ok_or("These private settings are no longer available. Revoke the old connection before creating another.")?;
        if !self
            .vault
            .as_ref()
            .ok_or("Your vault is locked.")?
            .clients()?
            .iter()
            .any(|c| c.id == pending.client_id)
        {
            self.pending_client_setup = None;
            return Err(
                "This connection was revoked. Its settings cannot be saved or revealed.".into(),
            );
        }
        let result = deliver(&pending.configuration, pending.extension);
        self.touch();
        result
    }
}

pub fn save_settings(path: &Path, configuration: &str, extension: &str) -> Result<(), String> {
    if !path.is_absolute() || path.extension().is_none_or(|e| e != extension) {
        return Err(format!(
            "Choose a new absolute file path ending in .{extension}."
        ));
    }
    let parent = path
        .parent()
        .and_then(|p| p.canonicalize().ok())
        .ok_or("Choose an existing local destination folder.")?;
    if parent
        .ancestors()
        .any(|p| p.join(".git").exists() || p.file_name().is_some_and(|n| n == ".git"))
    {
        return Err(
            "These settings contain a secret. Choose a folder outside a Git repository.".into(),
        );
    }
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    // Use the resolved parent and refuse existing files, including symlinks.
    let destination = parent.join(path.file_name().ok_or("Choose a filename.")?);
    let mut file = options
        .open(&destination)
        .map_err(|_| "The destination exists or cannot be created. Choose a new filename.")?;
    if file
        .write_all(configuration.as_bytes())
        .and_then(|_| file.sync_all())
        .is_err()
    {
        drop(file);
        let _ = std::fs::remove_file(destination);
        return Err(
            "Could not finish saving private settings. Check available space and try again.".into(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        client_config::{create_setup, ClientFormat},
        db::Vault,
    };

    fn prepare(s: &mut Session) -> PreparedClientSetup {
        let setup = create_setup(
            s.access().unwrap(),
            "Fictional setup",
            false,
            false,
            ClientFormat::Codex,
            Path::new("/Applications/Fictional.app/pre-drink"),
            Path::new("/tmp/fictional/agents.sock"),
        )
        .unwrap();
        s.prepare_client_settings(setup, "toml")
    }
    fn session() -> Session {
        let mut s = Session::new(std::path::PathBuf::new());
        s.vault = Some(Vault::memory().unwrap());
        s
    }

    #[test]
    fn settings_stay_in_backend_until_explicit_delivery_and_cannot_cross_sessions() {
        let mut s = session();
        let first = prepare(&mut s);
        let metadata = serde_json::to_string(&first).unwrap();
        assert!(
            !metadata.contains("configuration") && !metadata.contains("PRE_DRINK_CLIENT_TOKEN")
        );
        let config = s
            .with_client_settings(&first.request, |text, _| Ok(text.to_owned()))
            .unwrap();
        let parsed: toml::Value = toml::from_str(&config).unwrap();
        assert_eq!(
            parsed["mcp_servers"]["pre-drink"]["env"]["PRE_DRINK_CLIENT_ID"].as_str(),
            Some(first.client.id.as_str())
        );
        let second = prepare(&mut s);
        s.discard_client_settings(&first.request.setup_id); // old modal cleanup cannot erase the new one
        assert!(s
            .with_client_settings(&first.request, |_, _| -> Result<(), String> {
                panic!("stale setup delivered")
            })
            .is_err());
        assert!(s
            .with_client_settings(&second.request, |_, _| Ok(()))
            .is_ok());
        s.lock();
        assert!(s.pending_client_setup.is_none());
        s.vault = Some(Vault::memory().unwrap());
        assert!(s
            .with_client_settings(&second.request, |_, _| -> Result<(), String> {
                panic!("old session delivered")
            })
            .is_err());
    }

    #[test]
    fn revoked_closed_and_expired_settings_never_reach_the_writer() {
        for mode in ["revoke", "close", "expire"] {
            let mut s = session();
            let prepared = prepare(&mut s);
            match mode {
                "revoke" => s
                    .access()
                    .unwrap()
                    .revoke_client(&prepared.client.id)
                    .unwrap(),
                "close" => s.discard_client_settings(&prepared.request.setup_id),
                _ => s.last_activity = std::time::SystemTime::now() - crate::session::IDLE_TIMEOUT,
            }
            assert!(s
                .with_client_settings(&prepared.request, |_, _| -> Result<(), String> {
                    panic!("invalid setup delivered")
                })
                .is_err());
            assert!(s.pending_client_setup.is_none());
        }
    }

    #[test]
    fn saved_settings_are_private_exact_and_never_overwrite_or_enter_a_repo() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.toml");
        save_settings(&path, "fictional settings", "toml").unwrap();
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "fictional settings"
        );
        assert!(save_settings(&path, "replacement", "toml").is_err());
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "fictional settings"
        );
        assert!(save_settings(&dir.path().join("bad.json"), "fictional", "toml").is_err());
        assert!(save_settings(Path::new("relative.toml"), "fictional", "toml").is_err());
        #[cfg(unix)]
        {
            use std::os::unix::fs::{symlink, PermissionsExt};
            assert_eq!(
                std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
            let link = dir.path().join("link.toml");
            symlink(&path, &link).unwrap();
            assert!(save_settings(&link, "replacement", "toml").is_err());
        }
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        assert!(save_settings(&dir.path().join("secret.toml"), "fictional", "toml").is_err());
        assert!(!dir.path().join("secret.toml").exists());
    }
}
