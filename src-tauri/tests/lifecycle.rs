use pre_drink::{
    db::Vault,
    memory::MemoryInput,
    session::{Session, IDLE_TIMEOUT},
};
use std::{fs, path::Path, time::SystemTime};
const OLD: &str = "synthetic original password phrase";
const NEW: &str = "synthetic replacement password phrase";
fn note(v: &Vault, text: &str) {
    v.save_memory(MemoryInput {
        id: None,
        kind: "fact".into(),
        content: text.into(),
        source_message_id: None,
        private: false,
    })
    .unwrap();
}
fn backup(path: &Path) {
    let v = Vault::memory().unwrap();
    note(&v, "Backup garden plan.");
    v.create_client("Old synthetic registration", false)
        .unwrap();
    v.encrypted_backup(path, NEW).unwrap();
}
fn live(root: &Path) -> Session {
    let mut s = Session::new(root.into());
    s.unlock(OLD, true, false).unwrap();
    note(s.access().unwrap(), "Original garden plan.");
    s
}

#[test]
fn restore_preserves_original_and_source_removes_clients_and_reopens_locked() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("backup");
    backup(&source);
    let before = fs::read(&source).unwrap();
    let mut s = live(dir.path());
    let original = s.path().unwrap();
    s.sharing = true;
    let p = s.prepare_restore(&source, NEW).unwrap();
    assert!(!s.sharing);
    assert_eq!(p.memories, 1);
    assert_eq!(p.clients_to_remove, 1);
    assert_eq!(s.path().unwrap(), original);
    let result = s.apply_replacement(&p.review_id, NEW).unwrap();
    assert!(!result.durability_warning);
    assert!(s.vault.is_none());
    assert!(!s.sharing);
    assert_eq!(result.previous_vault.unwrap(), original.to_string_lossy());
    assert_eq!(
        fs::read(source).unwrap(),
        before,
        "Inspection never modifies backup bytes"
    );
    let old = Vault::open(&original, OLD).unwrap();
    assert_eq!(
        old.memories(None, "", false).unwrap()[0].content,
        "Original garden plan."
    );
    drop(old);
    drop(s);
    let mut s = Session::new(dir.path().into());
    assert!(s.unlock(OLD, false, false).is_err());
    s.unlock(NEW, false, false).unwrap();
    assert_eq!(
        s.access().unwrap().memories(None, "", false).unwrap()[0].content,
        "Backup garden plan."
    );
    assert!(s.access().unwrap().clients().unwrap().is_empty());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(s.path().unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        assert_eq!(
            fs::metadata(s.path().unwrap().parent().unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
    }
}

#[test]
fn password_rotation_requires_current_password_preserves_data_and_old_recovery_copy() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = live(dir.path());
    let old_path = s.path().unwrap();
    s.access()
        .unwrap()
        .create_client("Synthetic client", false)
        .unwrap();
    assert!(s.prepare_password_change("incorrect current", NEW).is_err());
    assert!(s.prepare_password_change(OLD, "short").is_err());
    assert!(s.prepare_password_change(OLD, OLD).is_err());
    let p = s.prepare_password_change(OLD, NEW).unwrap();
    assert_eq!(p.clients_to_remove, 1);
    s.apply_replacement(&p.review_id, NEW).unwrap();
    assert!(s.unlock(OLD, false, false).is_err());
    s.unlock(NEW, false, false).unwrap();
    assert_eq!(
        s.access().unwrap().memories(None, "", false).unwrap()[0].content,
        "Original garden plan."
    );
    assert!(s.access().unwrap().clients().unwrap().is_empty());
    assert!(Vault::open(&old_path, OLD).is_ok());
    let next = s.prepare_password_change(NEW, OLD).unwrap();
    s.apply_replacement(&next.review_id, OLD).unwrap();
    assert!(s.unlock(NEW, false, false).is_err());
    s.unlock(OLD, false, false).unwrap();
}

#[test]
fn restores_into_empty_or_locked_storage_without_knowing_old_password() {
    for existing in [true, false] {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("backup");
        backup(&source);
        let mut s = if existing {
            let mut s = live(dir.path());
            s.lock();
            s
        } else {
            Session::new(dir.path().into())
        };
        let p = s.prepare_restore(&source, NEW).unwrap();
        assert_eq!(p.previous_vault.is_some(), existing);
        s.apply_replacement(&p.review_id, NEW).unwrap();
        s.unlock(NEW, false, false).unwrap();
        assert_eq!(
            s.access().unwrap().memories(None, "", false).unwrap().len(),
            1
        );
    }
}

#[test]
fn stale_edit_lock_expiry_cancel_and_wrong_commit_password_preserve_original() {
    for action in ["edit", "lock", "expire", "cancel", "password", "token"] {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("backup");
        backup(&source);
        let mut s = live(dir.path());
        let original = s.path().unwrap();
        let p = s.prepare_restore(&source, NEW).unwrap();
        match action {
            "edit" => note(s.access().unwrap(), "Newer unsaved-to-backup knowledge."),
            "lock" => s.lock(),
            "expire" => s.last_activity = SystemTime::now() - IDLE_TIMEOUT,
            "cancel" => s.cancel_replacement(),
            _ => (),
        }
        assert!(
            s.apply_replacement(
                if action == "token" {
                    "stale"
                } else {
                    &p.review_id
                },
                if action == "password" { "wrong" } else { NEW }
            )
            .is_err(),
            "{action}"
        );
        assert_eq!(s.path().unwrap(), original);
        s.lock();
        s.unlock(OLD, false, false).unwrap();
        assert!(!s.sharing);
        assert!(s
            .access()
            .unwrap()
            .memories(None, "", false)
            .unwrap()
            .iter()
            .any(|m| m.content == "Original garden plan."));
        assert_eq!(
            fs::read_dir(dir.path())
                .unwrap()
                .filter_map(Result::ok)
                .filter(|f| f.file_name().to_string_lossy().starts_with("vault-"))
                .count(),
            0,
            "Cancelled preparations are removed"
        );
    }
}

#[test]
fn invalid_backups_active_files_and_pointer_corruption_fail_closed() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = live(dir.path());
    let original = s.path().unwrap();
    assert!(s.prepare_restore(&original, OLD).is_err());
    let source = dir.path().join("bad-backup");
    fs::write(&source, b"truncated").unwrap();
    assert!(s.prepare_restore(&source, NEW).is_err());
    fs::remove_file(&source).unwrap();
    backup(&source);
    assert!(s.prepare_restore(&source, OLD).is_err());
    fs::write(dir.path().join("bad-backup-wal"), b"journal present").unwrap();
    assert!(s.prepare_restore(&source, NEW).is_err());
    fs::remove_file(dir.path().join("bad-backup-wal")).unwrap();
    // Unknown schema is rejected without modifying the source or active vault.
    let key = pre_drink::crypto::key(NEW, &pre_drink::crypto::salt(&source).unwrap()).unwrap();
    let conn = rusqlite::Connection::open(&source).unwrap();
    pre_drink::crypto::configure(&conn, &key).unwrap();
    conn.execute_batch("PRAGMA user_version=999;").unwrap();
    drop(conn);
    let unchanged = fs::read(&source).unwrap();
    assert!(s.prepare_restore(&source, NEW).is_err());
    assert_eq!(fs::read(&source).unwrap(), unchanged);
    assert_eq!(s.path().unwrap(), original);
    s.lock();
    fs::write(dir.path().join("active-vault.json"), b"invalid pointer").unwrap();
    assert!(s.status().is_err());
    assert!(s.unlock(OLD, true, false).is_err());
    assert!(
        original.exists(),
        "Corrupt selection must not overwrite or create a replacement"
    );
}

#[test]
fn encrypted_page_damage_and_truncation_leave_live_vault_and_backup_usable() {
    let root = tempfile::tempdir().unwrap();
    let storage = root.path().join("live");
    fs::create_dir(&storage).unwrap();
    let mut s = live(&storage);
    let original = s.path().unwrap();
    let good = root.path().join("verified-backup");
    backup(&good);
    let bytes = fs::read(&good).unwrap();
    assert!(bytes.len() > 8192);
    let damaged = root.path().join("damaged-backup");
    for failure in ["header-page", "later-page", "partial-page"] {
        let mut broken = bytes.clone();
        match failure {
            "header-page" => broken[128] ^= 1,
            "later-page" => broken[4096 + 128] ^= 1,
            _ => {
                broken.truncate(broken.len() - 17);
            }
        }
        fs::write(&damaged, &broken).unwrap();
        s.sharing = true;
        assert!(s.prepare_restore(&damaged, NEW).is_err(), "{failure}");
        assert!(!s.sharing);
        assert_eq!(s.path().unwrap(), original);
        assert_eq!(
            fs::read(&damaged).unwrap(),
            broken,
            "Inspection must not repair or overwrite its input"
        );
        assert!(!storage.join("active-vault.json").exists());
        assert_eq!(
            fs::read_dir(&storage)
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| entry.file_type().unwrap().is_dir())
                .count(),
            0,
            "Failed preparations must be discarded"
        );
        s.lock();
        s.unlock(OLD, false, false).unwrap();
        assert_eq!(
            s.access().unwrap().memories(None, "", false).unwrap()[0].content,
            "Original garden plan."
        );
    }
    assert_eq!(fs::read(&good).unwrap(), bytes);
    // A failed attempt must not poison the next valid restore.
    let review = s.prepare_restore(&good, NEW).unwrap();
    s.apply_replacement(&review.review_id, NEW).unwrap();
    s.unlock(NEW, false, false).unwrap();
    assert_eq!(
        s.access().unwrap().memories(None, "", false).unwrap()[0].content,
        "Backup garden plan."
    );
}

#[test]
fn reviewed_restore_uses_private_copy_even_if_original_backup_disappears() {
    let root = tempfile::tempdir().unwrap();
    let mut s = live(root.path());
    let source = root.path().join("removable-backup");
    backup(&source);
    let review = s.prepare_restore(&source, NEW).unwrap();
    // Models disconnecting the backup drive after inspection has completed.
    fs::remove_file(&source).unwrap();
    s.apply_replacement(&review.review_id, NEW).unwrap();
    drop(s);
    let mut reopened = Session::new(root.path().into());
    reopened.unlock(NEW, false, false).unwrap();
    assert!(!reopened.sharing);
    let vault = reopened.access().unwrap();
    assert_eq!(
        vault.memories(None, "", false).unwrap()[0].content,
        "Backup garden plan."
    );
    assert!(vault.clients().unwrap().is_empty());
    assert!(
        reopened.apply_replacement(&review.review_id, NEW).is_err(),
        "An applied review cannot be replayed after reopening"
    );
}
