//! Verified replacement generations. Never rename a live SQLite file away from its WAL.
use crate::{crypto, db::Vault, session::Session};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

const POINTER: &str = "active-vault.json";
const LAYOUT_MARKER: &str = "vault-layout-v1";
const DATABASE: &str = "vault.enc.sqlite3";
const MAX_BACKUP_BYTES: u64 = 8 * 1024 * 1024 * 1024;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Pointer {
    version: u32,
    directory: String,
}

#[derive(Clone, Serialize)]
pub struct LifecyclePreview {
    pub review_id: String,
    pub operation: &'static str,
    pub conversations: i64,
    pub messages: i64,
    pub memories: i64,
    pub briefs: i64,
    pub clients_to_remove: i64,
    pub previous_vault: Option<String>,
}

pub struct PendingReplacement {
    preview: LifecyclePreview,
    directory: PathBuf,
    digest: String,
    active_path: PathBuf,
    generation: u64,
    changes: Option<u64>,
    keep: bool,
}
impl Drop for PendingReplacement {
    fn drop(&mut self) {
        if !self.keep {
            let _ = fs::remove_dir_all(&self.directory);
        }
    }
}

#[derive(Serialize)]
pub struct LifecycleResult {
    pub previous_vault: Option<String>,
    pub durability_warning: bool,
}

fn regular(path: &Path) -> Result<(), String> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| "The vault file is missing or unreadable.")?;
    if !metadata.file_type().is_file() {
        return Err("Choose a regular vault file, not a symbolic link.".into());
    }
    Ok(())
}

pub fn active_path(root: &Path) -> Result<PathBuf, String> {
    let pointer = root.join(POINTER);
    match fs::symlink_metadata(&pointer) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            if fs::symlink_metadata(root.join(LAYOUT_MARKER)).is_ok() {
                return Err("The active vault selection is missing. Preserve this folder and follow the recovery guide.".into());
            }
            let path = root.join(DATABASE);
            if fs::symlink_metadata(&path).is_ok() { regular(&path)?; }
            Ok(path)
        }
        Err(_) => Err("Cannot read the active vault selection. Preserve this folder and follow the recovery guide.".into()),
        Ok(metadata) => {
            if !metadata.file_type().is_file() || metadata.len() > 512 { return Err("Invalid active vault selection. Follow the recovery guide; nothing was replaced.".into()); }
            let pointer: Pointer = serde_json::from_slice(&fs::read(pointer).map_err(|_| "Cannot read the active vault selection.")?).map_err(|_| "Invalid active vault selection. Follow the recovery guide.")?;
            let suffix = pointer.directory.strip_prefix("vault-").ok_or("Invalid vault directory.")?;
            let id = uuid::Uuid::parse_str(suffix).map_err(|_| "Invalid vault directory.")?;
            if pointer.version != 1 || suffix != id.to_string() { return Err("Unsupported active vault selection.".into()); }
            let directory = root.join(pointer.directory);
            let metadata = fs::symlink_metadata(&directory).map_err(|_| "Selected vault directory is missing. Follow the recovery guide.")?;
            if !metadata.file_type().is_dir() { return Err("Selected vault directory must not be a symbolic link.".into()); }
            let path = directory.join(DATABASE); regular(&path)?; Ok(path)
        }
    }
}

fn create_directory(root: &Path) -> Result<PathBuf, String> {
    let path = root.join(format!("vault-{}", uuid::Uuid::new_v4()));
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(&path).map_err(|_| {
        "Cannot create a replacement vault. Check free space and folder permissions."
    })?;
    Ok(path)
}
fn sync_directory(path: &Path) -> Result<(), String> {
    fs::File::open(path)
        .and_then(|f| f.sync_all())
        .map_err(|_| "Could not flush the vault folder to disk.")
        .map_err(String::from)
}
fn digest(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|_| "Cannot read the prepared vault.")?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|_| "Cannot read the prepared vault.")?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(hex::encode(hash.finalize()))
}

// Backup files must be standalone; never silently copy a live SQLite main file.
fn copy_backup(source: &Path, destination: &Path) -> Result<(), String> {
    regular(source)?;
    for suffix in ["-wal", "-journal"] {
        let sidecar = PathBuf::from(format!("{}{suffix}", source.to_string_lossy()));
        match fs::metadata(sidecar) {
            Ok(m) if m.len() != 0 => return Err("This file has an active journal. Use a verified encrypted backup made by Pre-Drink.".into()),
            Ok(_) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(_) => return Err("Cannot inspect backup journals.".into()),
        }
    }
    let mut source = fs::File::open(source).map_err(|_| "Cannot read this backup.")?;
    if source
        .metadata()
        .map_err(|_| "Cannot inspect this backup.")?
        .len()
        > MAX_BACKUP_BYTES
    {
        return Err("Backups over 8 GiB require manual recovery in this version.".into());
    }
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut out = options
        .open(destination)
        .map_err(|_| "Cannot create the prepared vault.")?;
    let copied = std::io::copy(
        &mut Read::by_ref(&mut source).take(MAX_BACKUP_BYTES + 1),
        &mut out,
    )
    .map_err(|_| "Could not copy the backup. Check free space.")?;
    if copied > MAX_BACKUP_BYTES {
        return Err("The backup exceeds the restore size limit.".into());
    }
    out.sync_all()
        .map_err(|_| "Could not flush the prepared backup.".into())
}

fn authenticate(path: &Path, password: &str) -> Result<(), String> {
    if password.len() > 1024 {
        return Err("Password is too long.".into());
    }
    let key = crypto::key(password, &crypto::salt(path)?)?;
    let connection =
        rusqlite::Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|_| "Cannot verify the current password.")?;
    crypto::configure(&connection, &key)
}

impl Session {
    pub fn cancel_prepared(&mut self, review_id: &str) {
        if self
            .pending_replacement
            .as_ref()
            .is_some_and(|p| p.preview.review_id == review_id)
        {
            self.cancel_replacement();
        }
    }

    pub fn cancel_replacement(&mut self) {
        self.pending_replacement.take();
    }

    pub fn prepare_restore(
        &mut self,
        source: &Path,
        password: &str,
    ) -> Result<LifecyclePreview, String> {
        self.expire();
        self.cancel_replacement();
        self.sharing = false;
        if password.len() > 1024 {
            return Err("Password is too long.".into());
        }
        let active = self.path()?;
        if active.exists() && fs::canonicalize(source).ok() == fs::canonicalize(&active).ok() {
            return Err("Select a separate verified backup, not your active vault.".into());
        }
        let mut pending = self.new_replacement("restore")?;
        let path = pending.directory.join(DATABASE);
        copy_backup(source, &path)?;
        self.finish_preparation(&mut pending, password)?;
        pending.changes = self.vault.as_ref().map(|v| v.connection.total_changes());
        let preview = pending.preview.clone();
        self.pending_replacement = Some(pending);
        self.touch();
        Ok(preview)
    }

    pub fn prepare_password_change(
        &mut self,
        current: &str,
        next: &str,
    ) -> Result<LifecyclePreview, String> {
        self.cancel_replacement();
        self.sharing = false;
        self.access()?;
        crypto::validate_password(next)?;
        if current == next {
            return Err("Choose a different new password.".into());
        }
        authenticate(&self.path()?, current)?;
        let mut pending = self.new_replacement("password")?;
        self.access()?
            .encrypted_backup(&pending.directory.join(DATABASE), next)?;
        self.finish_preparation(&mut pending, next)?;
        pending.changes = self.vault.as_ref().map(|v| v.connection.total_changes());
        let preview = pending.preview.clone();
        self.pending_replacement = Some(pending);
        self.touch();
        Ok(preview)
    }

    fn new_replacement(&self, operation: &'static str) -> Result<PendingReplacement, String> {
        let active = self.path()?;
        let directory = create_directory(&self.directory)?;
        Ok(PendingReplacement {
            preview: LifecyclePreview {
                review_id: uuid::Uuid::new_v4().to_string(),
                operation,
                conversations: 0,
                messages: 0,
                memories: 0,
                briefs: 0,
                clients_to_remove: 0,
                previous_vault: active
                    .exists()
                    .then(|| active.to_string_lossy().into_owned()),
            },
            directory,
            digest: String::new(),
            active_path: active,
            generation: self.generation,
            changes: self.vault.as_ref().map(|v| v.connection.total_changes()),
            keep: false,
        })
    }

    fn finish_preparation(
        &self,
        pending: &mut PendingReplacement,
        password: &str,
    ) -> Result<(), String> {
        let path = pending.directory.join(DATABASE);
        let v = Vault::open(&path, password)?;
        v.verify()?;
        let stats = v.stats()?;
        pending.preview.conversations = stats.conversations;
        pending.preview.messages = stats.messages;
        for (table, field) in [
            ("memory", &mut pending.preview.memories),
            ("briefs", &mut pending.preview.briefs),
            ("agent_clients", &mut pending.preview.clients_to_remove),
        ] {
            *field = v
                .connection
                .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
                .map_err(|_| "Cannot inspect prepared vault.")?;
        }
        // Old credentials never regain access through restore or password rotation.
        v.connection
            .execute("DELETE FROM agent_clients", [])
            .map_err(|_| "Cannot reset restored agent access.")?;
        v.connection
            .execute(
                "UPDATE curation_policy SET publish_archive=0,publish_session=0",
                [],
            )
            .map_err(|_| "Cannot reset restored publication policies.")?;
        v.verify()?;
        v.checkpoint()?;
        drop(v);
        fs::File::open(&path)
            .and_then(|f| f.sync_all())
            .map_err(|_| "Cannot flush the verified replacement.")?;
        sync_directory(&pending.directory)?;
        pending.digest = digest(&path)?;
        Ok(())
    }

    pub fn apply_replacement(
        &mut self,
        review_id: &str,
        password: &str,
    ) -> Result<LifecycleResult, String> {
        self.expire();
        self.sharing = false;
        let mut pending = self
            .pending_replacement
            .take()
            .ok_or("Inspect a backup or prepare a password change first.")?;
        if pending.preview.review_id != review_id
            || pending.generation != self.generation
            || pending.active_path != self.path()?
            || pending.changes != self.vault.as_ref().map(|v| v.connection.total_changes())
        {
            return Err(
                "Your vault or session changed. Prepare and review the operation again.".into(),
            );
        }
        let path = pending.directory.join(DATABASE);
        if digest(&path)? != pending.digest {
            return Err("The prepared file changed. Nothing was replaced.".into());
        }
        if password.len() > 1024 {
            return Err("Password is too long.".into());
        }
        let v = Vault::open(&path, password)?;
        v.verify()?;
        v.checkpoint()?;
        drop(v);
        fs::File::open(&path)
            .and_then(|f| f.sync_all())
            .map_err(|_| "Cannot flush replacement vault.")?;
        sync_directory(&pending.directory)?;
        // Keep every old file and journal in its original directory. The only
        // commit point is the same-folder atomic rename of a small pointer file.
        let pointer = Pointer {
            version: 1,
            directory: pending
                .directory
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        };
        let temporary = self
            .directory
            .join(format!(".active-{}.partial", uuid::Uuid::new_v4()));
        let marker = self.directory.join(LAYOUT_MARKER);
        let mut new_marker = false;
        let publication = (|| -> Result<bool, String> {
            let mut options = fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options
                .open(&temporary)
                .map_err(|_| "Cannot prepare the vault switch. Original vault preserved.")?;
            file.write_all(
                &serde_json::to_vec(&pointer).map_err(|_| "Cannot encode vault selection.")?,
            )
            .and_then(|_| file.sync_all())
            .map_err(|_| "Cannot flush the vault switch. Original vault preserved.")?;
            sync_directory(&self.directory)?;
            // Persist the no-fallback guard BEFORE publishing the selector.
            // A crash before the first selector now requires explicit recovery;
            // it can never silently make an old root vault current again.
            match fs::symlink_metadata(&marker) {
                Ok(metadata) => {
                    if !metadata.file_type().is_file()
                        || metadata.len() != 2
                        || fs::read(&marker).map_err(|_| "Cannot read vault recovery guard.")?
                            != b"1\n"
                    {
                        return Err("Invalid vault recovery guard. Preserve this folder and follow the recovery guide.".into());
                    }
                    fs::File::open(&marker)
                        .and_then(|f| f.sync_all())
                        .map_err(|_| "Cannot flush vault recovery guard.")?;
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    let mut guard = options.open(&marker).map_err(|_| {
                        "Cannot prepare vault recovery guard. Original vault preserved."
                    })?;
                    new_marker = true;
                    guard
                        .write_all(b"1\n")
                        .and_then(|_| guard.sync_all())
                        .map_err(|_| {
                            "Cannot flush vault recovery guard. Original vault preserved."
                        })?;
                }
                Err(_) => return Err("Cannot inspect vault recovery guard.".into()),
            }
            sync_directory(&self.directory)?;
            let generation = self.generation;
            self.lock();
            if generation == self.generation {
                self.generation = self.generation.wrapping_add(1);
            }
            #[cfg(test)]
            publication_checkpoint(&self.directory, "before");
            fs::rename(&temporary, self.directory.join(POINTER))
                .map_err(|_| "Could not switch vaults. Unlock the original vault to continue.")?;
            // Never remove the active generation after commit.
            pending.keep = true;
            #[cfg(test)]
            publication_checkpoint(&self.directory, "after");
            Ok(sync_directory(&self.directory).is_err())
        })();
        // Ordinary precommit failures can undo a newly created guard. If that
        // cleanup fails, leave recovery explicit rather than guessing a vault.
        if publication.is_err() && new_marker && !pending.keep {
            let _ = fs::remove_file(marker);
            let _ = sync_directory(&self.directory);
        }
        let _ = fs::remove_file(temporary);
        let durability_warning = publication?;
        Ok(LifecycleResult {
            previous_vault: pending.preview.previous_vault.clone(),
            durability_warning,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const OLD: &str = "synthetic lifecycle initial phrase";
    const NEW: &str = "synthetic lifecycle replacement phrase";
    #[test]
    fn tampered_preparation_and_orphaned_stage_never_replace_original() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = Session::new(dir.path().into());
        s.unlock(OLD, true, false).unwrap();
        let original = s.path().unwrap();
        let p = s.prepare_password_change(OLD, NEW).unwrap();
        let path = s
            .pending_replacement
            .as_ref()
            .unwrap()
            .directory
            .join(DATABASE);
        let mut contents = fs::read(&path).unwrap();
        contents[120] ^= 1;
        fs::write(path, contents).unwrap();
        assert!(s.apply_replacement(&p.review_id, NEW).is_err());
        assert_eq!(s.path().unwrap(), original);
        s.prepare_password_change(OLD, NEW).unwrap();
        // Model a process interruption leaving a complete but unpublished stage.
        let mut pending = s.pending_replacement.take().unwrap();
        pending.keep = true;
        drop(pending);
        drop(s);
        let mut reopened = Session::new(dir.path().into());
        assert_eq!(reopened.path().unwrap(), original);
        reopened.unlock(OLD, false, false).unwrap();
        assert!(!dir.path().join(POINTER).exists());
    }
    #[cfg(unix)]
    #[test]
    fn unwritable_commit_folder_preserves_original_and_removes_prepared_vault() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let mut s = Session::new(dir.path().into());
        s.unlock(OLD, true, false).unwrap();
        let original = s.path().unwrap();
        let p = s.prepare_password_change(OLD, NEW).unwrap();
        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o500)).unwrap();
        let result = s.apply_replacement(&p.review_id, NEW);
        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700)).unwrap();
        assert!(result.is_err());
        assert_eq!(s.path().unwrap(), original);
        s.lock();
        s.unlock(OLD, false, false).unwrap();
    }
}

#[cfg(test)]
mod selector_tests {
    use super::*;
    #[test]
    fn missing_selector_after_rotation_does_not_reopen_the_old_root_vault() {
        let root = tempfile::tempdir().unwrap();
        let mut s = Session::new(root.path().into());
        let old = "synthetic original selector phrase";
        let new = "synthetic changed selector phrase";
        s.unlock(old, true, false).unwrap();
        let original = s.path().unwrap();
        let p = s.prepare_password_change(old, new).unwrap();
        s.apply_replacement(&p.review_id, new).unwrap();
        fs::remove_file(root.path().join(POINTER)).unwrap();
        assert!(s.status().is_err());
        assert!(s.unlock(old, false, false).is_err());
        assert!(Vault::open(&original, old).is_ok());
    }
}

// Only the isolated child-test executable can stop at publication boundaries.
// No fault-injection environment variable is read by production builds.
#[cfg(test)]
fn publication_checkpoint(root: &Path, stage: &str) {
    if std::env::var_os("PD_SWITCH_CHILD_ROOT").as_deref() == Some(root.as_os_str())
        && std::env::var("PD_SWITCH_CHILD_STAGE").as_deref() == Ok(stage)
    {
        fs::write(root.join("child-ready"), stage).unwrap();
        loop {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }
}

#[cfg(test)]
mod interrupted_switch_tests {
    use super::*;
    use std::{
        process::{Child, Command, Stdio},
        time::{Duration, Instant},
    };
    const OLD: &str = "fictional switch original phrase";
    const NEW: &str = "fictional switch replacement phrase";
    struct ChildGuard(Child);
    impl Drop for ChildGuard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    fn check(stage: &str, test_name: &str) {
        if let Some(root) = std::env::var_os("PD_SWITCH_CHILD_ROOT") {
            let mut s = Session::new(root.into());
            s.unlock(OLD, false, false).unwrap();
            let preview = s.prepare_password_change(OLD, NEW).unwrap();
            s.apply_replacement(&preview.review_id, NEW).unwrap();
            panic!("Missed publication checkpoint");
        }
        let root = tempfile::tempdir().unwrap();
        let original = root.path().join(DATABASE);
        let mut s = Session::new(root.path().into());
        s.unlock(OLD, true, false).unwrap();
        let v = s.access().unwrap();
        v.save_memory(crate::memory::MemoryInput {
            id: None,
            kind: "project".into(),
            content: "Fictional cedar planters".into(),
            source_message_id: None,
            private: false,
        })
        .unwrap();
        v.create_scoped_client("Fictional prior reader", None, false)
            .unwrap();
        v.checkpoint().unwrap();
        s.lock();
        let mut child = ChildGuard(
            Command::new(std::env::current_exe().unwrap())
                .args(["--exact", test_name, "--nocapture"])
                .env("PD_SWITCH_CHILD_ROOT", root.path())
                .env("PD_SWITCH_CHILD_STAGE", stage)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap(),
        );
        let deadline = Instant::now() + Duration::from_secs(60);
        while !root.path().join("child-ready").exists() {
            assert!(
                child.0.try_wait().unwrap().is_none(),
                "Child exited before switch boundary"
            );
            assert!(Instant::now() < deadline, "Child missed switch boundary");
            std::thread::sleep(Duration::from_millis(20));
        }
        child.0.kill().unwrap();
        child.0.wait().unwrap();
        assert!(
            root.path().join(LAYOUT_MARKER).exists(),
            "Recovery guard must precede commit"
        );
        let old = Vault::open(&original, OLD).unwrap();
        old.verify().unwrap();
        assert_eq!(old.memories(None, "", false).unwrap().len(), 1);
        if stage == "before" {
            let staged = fs::read_dir(root.path())
                .unwrap()
                .map(|e| e.unwrap().path())
                .find(|p| {
                    p.is_dir()
                        && p.file_name()
                            .unwrap()
                            .to_string_lossy()
                            .starts_with("vault-")
                })
                .unwrap();
            let replacement = Vault::open(&staged.join(DATABASE), NEW).unwrap();
            replacement.verify().unwrap();
            assert_eq!(replacement.memories(None, "", false).unwrap().len(), 1);
            // The first switch is interrupted: preserve both generations and
            // require explicit recovery, never guess which one is current.
            assert!(!root.path().join(POINTER).exists());
            assert!(active_path(root.path()).is_err());
        } else {
            let selected = active_path(root.path()).unwrap();
            assert_ne!(selected, original);
            let restored = Vault::open(&selected, NEW).unwrap();
            restored.verify().unwrap();
            assert_eq!(restored.memories(None, "", false).unwrap().len(), 1);
            let clients: i64 = restored
                .connection
                .query_row("SELECT count(*) FROM agent_clients", [], |r| r.get(0))
                .unwrap();
            assert_eq!(clients, 0);
            drop(restored);
            fs::remove_file(root.path().join(POINTER)).unwrap();
            assert!(
                active_path(root.path()).is_err(),
                "Never silently reopen the old root"
            );
        }
    }
    #[test]
    fn killed_before_selector_commit_preserves_recoverable_generations() {
        check("before", "lifecycle::interrupted_switch_tests::killed_before_selector_commit_preserves_recoverable_generations");
    }
    #[test]
    fn killed_after_selector_commit_never_falls_back_to_old_vault() {
        check("after", "lifecycle::interrupted_switch_tests::killed_after_selector_commit_never_falls_back_to_old_vault");
    }
}
