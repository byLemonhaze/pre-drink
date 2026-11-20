//! Every destructive operation below uses fictional data in a disposable directory.
use pre_drink::{db::Vault, memory::MemoryInput, retained_vaults::CONFIRMATION, session::Session};
use std::fs;
const OLD: &str = "synthetic original vault phrase";
const NEW: &str = "synthetic replacement vault phrase";
const BACKUP: &str = "synthetic external backup phrase";
fn rotated(root: &std::path::Path) -> Session {
    fs::create_dir(root).unwrap();
    let mut s = Session::new(root.into());
    s.unlock(OLD, true, false).unwrap();
    s.access()
        .unwrap()
        .save_memory(MemoryInput {
            id: None,
            kind: "fact".into(),
            content: "Fictional garden uses native plants.".into(),
            source_message_id: None,
            private: false,
        })
        .unwrap();
    let p = s.prepare_password_change(OLD, NEW).unwrap();
    s.apply_replacement(&p.review_id, NEW).unwrap();
    s.unlock(NEW, false, false).unwrap();
    s
}
#[test]
fn cleanup_requires_verified_backup_and_removes_only_reviewed_inactive_files() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("live");
    let mut s = rotated(&root);
    let original = root.join("vault.enc.sqlite3");
    let active = s.path().unwrap();
    let marker = fs::read(root.join("vault-layout-v1")).unwrap();
    let selector = fs::read(root.join("active-vault.json")).unwrap();
    let unrelated = root.join("keep.txt");
    fs::write(&unrelated, "fictional unrelated file").unwrap();
    for suffix in ["-wal", "-shm", "-journal"] {
        fs::write(
            root.join(format!("vault.enc.sqlite3{suffix}")),
            "fictional old journal",
        )
        .unwrap();
    }
    let preview = s.preview_retained_vaults().unwrap();
    assert!(!preview.backup_ready);
    assert_eq!(preview.copies.len(), 1);
    assert_eq!(preview.copies[0].files.len(), 4);
    assert!(s
        .remove_retained_vault("original", &preview.copies[0].review_hash, CONFIRMATION)
        .is_err());
    assert!(original.exists());
    let backup = temp.path().join("backup.sqlite3");
    s.backup_with_receipt(&backup, BACKUP).unwrap();
    let saved = fs::read(&backup).unwrap();
    let p = s.preview_retained_vaults().unwrap();
    assert!(p.backup_ready);
    assert!(s
        .remove_retained_vault("original", &p.copies[0].review_hash, "wrong")
        .is_err());
    assert!(s
        .remove_retained_vault("../backup.sqlite3", &p.copies[0].review_hash, CONFIRMATION)
        .is_err());
    s.sharing = true;
    s.remove_retained_vault("original", &p.copies[0].review_hash, CONFIRMATION)
        .unwrap();
    assert!(!s.sharing);
    assert!(!original.exists());
    assert!(active.exists());
    for suffix in ["-wal", "-shm", "-journal"] {
        assert!(!root.join(format!("vault.enc.sqlite3{suffix}")).exists());
    }
    assert_eq!(fs::read(root.join("active-vault.json")).unwrap(), selector);
    assert_eq!(fs::read(root.join("vault-layout-v1")).unwrap(), marker);
    assert_eq!(fs::read(&backup).unwrap(), saved);
    assert!(unrelated.exists());
    assert!(s.preview_retained_vaults().unwrap().copies.is_empty());
    s.lock();
    s.unlock(NEW, false, false).unwrap();
    assert_eq!(
        s.access().unwrap().memories(None, "", false).unwrap().len(),
        1
    );
    assert_eq!(
        Vault::open(&backup, BACKUP)
            .unwrap()
            .memories(None, "", false)
            .unwrap()
            .len(),
        1
    );
}
#[test]
fn cleanup_rejects_stale_reviews_modified_backups_and_new_sessions() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("live");
    let mut s = rotated(&root);
    let backup = temp.path().join("backup.sqlite3");
    s.backup_with_receipt(&backup, BACKUP).unwrap();
    let p = s.preview_retained_vaults().unwrap().copies.remove(0);
    fs::write(root.join("vault.enc.sqlite3-journal"), "fictional change").unwrap();
    assert!(s
        .remove_retained_vault(&p.id, &p.review_hash, CONFIRMATION)
        .is_err());
    let p = s.preview_retained_vaults().unwrap().copies.remove(0);
    fs::write(&backup, "damaged synthetic backup").unwrap();
    assert!(s
        .remove_retained_vault(&p.id, &p.review_hash, CONFIRMATION)
        .is_err());
    s.lock();
    assert!(s.preview_retained_vaults().is_err());
    s.unlock(NEW, false, false).unwrap();
    assert!(!s.preview_retained_vaults().unwrap().backup_ready);
    s.backup_with_receipt(&temp.path().join("fresh.sqlite3"), BACKUP)
        .unwrap();
    assert!(s
        .remove_retained_vault(&p.id, &p.review_hash, CONFIRMATION)
        .is_err());
    assert!(root.join("vault.enc.sqlite3").exists());
    let p = s.preview_retained_vaults().unwrap();
    s.access()
        .unwrap()
        .save_memory(MemoryInput {
            id: None,
            kind: "fact".into(),
            content: "Fictional update after backup.".into(),
            source_message_id: None,
            private: false,
        })
        .unwrap();
    assert!(s
        .remove_retained_vault("original", &p.copies[0].review_hash, CONFIRMATION)
        .is_err());
    assert!(!s.preview_retained_vaults().unwrap().backup_ready);
}
#[test]
fn generation_cleanup_excludes_active_and_pending_and_refuses_unknown_files() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("live");
    let mut s = rotated(&root);
    let old_generation = s.path().unwrap().parent().unwrap().to_owned();
    let p = s.prepare_password_change(NEW, OLD).unwrap();
    assert!(s.preview_retained_vaults().is_err());
    s.apply_replacement(&p.review_id, OLD).unwrap();
    s.unlock(OLD, false, false).unwrap();
    let active = s.path().unwrap();
    let id = old_generation.file_name().unwrap().to_str().unwrap();
    fs::write(old_generation.join("unexpected.txt"), "keep me").unwrap();
    assert!(s.preview_retained_vaults().is_err());
    fs::remove_file(old_generation.join("unexpected.txt")).unwrap();
    s.backup_with_receipt(&temp.path().join("verified.sqlite3"), BACKUP)
        .unwrap();
    let p = s.preview_retained_vaults().unwrap();
    assert_eq!(p.copies.len(), 2);
    let old = p.copies.iter().find(|p| p.id == id).unwrap();
    assert!(s
        .remove_retained_vault(
            active
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
            &old.review_hash,
            CONFIRMATION
        )
        .is_err());
    s.remove_retained_vault(id, &old.review_hash, CONFIRMATION)
        .unwrap();
    assert!(!old_generation.exists());
    assert!(active.exists());
    assert!(root.join("vault.enc.sqlite3").exists());
}
#[cfg(unix)]
#[test]
fn cleanup_refuses_symlinks_and_hardlinks_without_touching_targets() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("live");
    let mut s = rotated(&root);
    let external = temp.path().join("external");
    fs::write(&external, "must survive").unwrap();
    let journal = root.join("vault.enc.sqlite3-journal");
    std::os::unix::fs::symlink(&external, &journal).unwrap();
    assert!(s.preview_retained_vaults().is_err());
    fs::remove_file(&journal).unwrap();
    fs::hard_link(&external, &journal).unwrap();
    assert!(s.preview_retained_vaults().is_err());
    fs::remove_file(&journal).unwrap();
    let dir_link = root.join(format!("vault-{}", uuid::Uuid::new_v4()));
    std::os::unix::fs::symlink(temp.path(), &dir_link).unwrap();
    assert!(s.preview_retained_vaults().is_err());
    assert_eq!(fs::read_to_string(&external).unwrap(), "must survive");
}
