//! Reviewed cleanup of inactive encrypted generations, never recursive deletion.
use crate::session::Session;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

const DATABASE: &str = "vault.enc.sqlite3";
const FILES: [&str; 4] = [
    "vault.enc.sqlite3",
    "vault.enc.sqlite3-wal",
    "vault.enc.sqlite3-shm",
    "vault.enc.sqlite3-journal",
];
pub const CONFIRMATION: &str = "REMOVE PREVIOUS VAULT";

pub(crate) struct BackupReceipt {
    path: PathBuf,
    digest: String,
    generation: u64,
    changes: u64,
}
#[derive(Serialize)]
pub struct RetainedVault {
    pub id: String,
    pub bytes: u64,
    pub modified_at: u64,
    pub files: Vec<String>,
    pub review_hash: String,
}
#[derive(Serialize)]
pub struct RetainedReview {
    pub copies: Vec<RetainedVault>,
    pub backup_ready: bool,
}

fn directory(path: &Path) -> Result<(), String> {
    if !fs::symlink_metadata(path)
        .map_err(|_| "Cannot inspect the vault folder.")?
        .file_type()
        .is_dir()
    {
        return Err("Vault folders must be real directories, not symbolic links.".into());
    }
    Ok(())
}
fn regular_digest(path: &Path) -> Result<(String, u64, u64), String> {
    let meta = fs::symlink_metadata(path)
        .map_err(|_| "Cannot inspect a retained file or backup. Refresh the review.")?;
    if !meta.file_type().is_file() {
        return Err("Cleanup refuses symbolic links and special files.".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if meta.nlink() != 1 {
            return Err("Cleanup refuses files with additional hard links.".into());
        }
    }
    if meta.len() > 8 * 1024 * 1024 * 1024 {
        return Err("This file exceeds the 8 GiB reviewed-cleanup limit. Keep it and use the recovery guide.".into());
    }
    let mut hash = Sha256::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        hash.update(meta.dev().to_be_bytes());
        hash.update(meta.ino().to_be_bytes());
        hash.update(meta.mtime_nsec().to_be_bytes());
        hash.update(meta.ctime().to_be_bytes());
        hash.update(meta.ctime_nsec().to_be_bytes());
    }
    let modified = meta
        .modified()
        .map_err(|_| "Cannot inspect a file date.")?
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    hash.update(modified.to_be_bytes());
    hash.update(meta.len().to_be_bytes());
    let mut file = fs::File::open(path).map_err(|_| "Cannot read a retained file or backup.")?;
    let mut buffer = [0u8; 65536];
    let mut read = 0;
    loop {
        let n = file
            .read(&mut buffer)
            .map_err(|_| "Could not finish checking a retained file or backup.")?;
        if n == 0 {
            break;
        }
        read += n as u64;
        if read > meta.len() {
            return Err("A file changed during review. Refresh the review.".into());
        }
        hash.update(&buffer[..n]);
    }
    if read != meta.len() {
        return Err("A file changed during review. Refresh the review.".into());
    }
    Ok((hex::encode(hash.finalize()), meta.len(), modified))
}
fn valid_generation(id: &str) -> bool {
    id.strip_prefix("vault-")
        .and_then(|v| uuid::Uuid::parse_str(v).ok().map(|u| u.to_string() == v))
        .unwrap_or(false)
}

impl Session {
    /// Existing backup operation verifies decryption and integrity before recording
    /// a session-scoped receipt. No password is retained.
    pub fn backup_with_receipt(&mut self, path: &Path, password: &str) -> Result<(), String> {
        self.backup_receipt = None;
        self.access()?.encrypted_backup(path, password)?;
        let path = fs::canonicalize(path)
            .map_err(|_| "Backup saved, but its location could not be verified for cleanup.")?;
        let root = fs::canonicalize(&self.directory)
            .map_err(|_| "Backup saved, but the vault folder could not be verified for cleanup.")?;
        if !path.starts_with(root) {
            let (digest, _, _) = regular_digest(&path)?;
            self.backup_receipt = Some(BackupReceipt {
                path,
                digest,
                generation: self.generation,
                changes: self.access()?.connection.total_changes(),
            });
        }
        self.touch();
        Ok(())
    }
    fn backup_ready(&self) -> bool {
        self.backup_receipt.as_ref().is_some_and(|b| {
            b.generation == self.generation
                && self
                    .vault
                    .as_ref()
                    .is_some_and(|v| v.connection.total_changes() == b.changes)
                && regular_digest(&b.path).is_ok_and(|(digest, _, _)| digest == b.digest)
        })
    }
    fn retained_plan(&mut self) -> Result<RetainedReview, String> {
        self.access()?;
        if self.pending_replacement.is_some() {
            return Err(
                "Finish or cancel the pending restore/password change before cleanup.".into(),
            );
        }
        directory(&self.directory)?;
        let active = self.path()?;
        if Path::new(&self.access()?.path) != active {
            return Err("The active vault changed. Lock and unlock it before cleanup.".into());
        }
        let changes = self.access()?.connection.total_changes();
        let mut ids = vec!["original".to_owned()];
        for entry in fs::read_dir(&self.directory).map_err(|_| "Cannot list retained vaults.")? {
            let name = entry
                .map_err(|_| "Cannot read the vault folder.")?
                .file_name()
                .to_string_lossy()
                .into_owned();
            if valid_generation(&name) {
                ids.push(name);
            }
        }
        ids.sort();
        let mut copies = Vec::new();
        for id in ids {
            let dir = if id == "original" {
                self.directory.clone()
            } else {
                self.directory.join(&id)
            };
            if dir.join(DATABASE) == active {
                continue;
            }
            directory(&dir)?;
            if id != "original" {
                for entry in fs::read_dir(&dir).map_err(|_| "Cannot inspect a retained folder.")? {
                    let name = entry
                        .map_err(|_| "Cannot inspect a retained folder.")?
                        .file_name();
                    if !FILES.iter().any(|v| name == *v) {
                        return Err("A retained vault folder contains unrecognized files. Cleanup is blocked; inspect that folder using the recovery guide.".into());
                    }
                }
            }
            let mut p = RetainedVault {
                id,
                bytes: 0,
                modified_at: 0,
                files: Vec::new(),
                review_hash: String::new(),
            };
            let mut hash = Sha256::new();
            hash.update(
                serde_json::to_vec(&(&p.id, &active, self.generation, changes))
                    .map_err(|e| e.to_string())?,
            );
            for name in FILES {
                let path = dir.join(name);
                match fs::symlink_metadata(&path) {
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                    Err(_) => return Err("Cannot inspect a retained file.".into()),
                    Ok(_) => (),
                }
                let (digest, bytes, modified) = regular_digest(&path)?;
                hash.update(name.as_bytes());
                hash.update(digest.as_bytes());
                p.bytes += bytes;
                p.modified_at = p.modified_at.max(modified);
                p.files.push(name.into());
            }
            if p.id == "original" && p.files.is_empty() {
                continue;
            }
            p.review_hash = hex::encode(hash.finalize());
            copies.push(p);
        }
        Ok(RetainedReview {
            copies,
            backup_ready: self.backup_ready(),
        })
    }
    pub fn preview_retained_vaults(&mut self) -> Result<RetainedReview, String> {
        let review = self.retained_plan()?;
        self.touch();
        Ok(review)
    }
    pub fn remove_retained_vault(
        &mut self,
        id: &str,
        review_hash: &str,
        confirmation: &str,
    ) -> Result<(), String> {
        self.sharing = false;
        if confirmation != CONFIRMATION {
            return Err(format!("Type {CONFIRMATION} to confirm this deletion."));
        }
        let review = self.retained_plan()?;
        if !review.backup_ready {
            return Err("First save a verified encrypted backup outside the vault folder, in this unlocked session. If the vault or backup changed, make a fresh backup.".into());
        }
        let selected = review
            .copies
            .into_iter()
            .find(|p| p.id == id && p.review_hash == review_hash)
            .ok_or("The retained copy or vault session changed. Refresh the review.")?;
        self.access()?.verify()?;
        let dir = if id == "original" {
            self.directory.clone()
        } else {
            self.directory.join(id)
        };
        // Only enumerated regular files; never recursive removal or user-supplied paths.
        // Any partial filesystem failure is reported honestly and can be reviewed again.
        for name in selected.files {
            fs::remove_file(dir.join(name)).map_err(|_| "Cleanup stopped. Some selected files may already be removed; refresh the review. The active vault was not targeted.")?;
        }
        if id != "original" {
            fs::remove_dir(&dir).map_err(|_| "Selected files were removed, but the folder could not be removed. Refresh the review.")?;
        }
        fs::File::open(&self.directory).and_then(|f| f.sync_all()).map_err(|_| "Files were removed, but the folder could not be flushed. Check the review again after restarting.")?;
        self.touch();
        Ok(())
    }
}
