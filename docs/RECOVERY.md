# Backup, restore and password changes

## Encrypted backup

In **Vault settings → Encrypted backup**, choose a strong password and a new filename on APFS or another local filesystem supporting hard links. The password can differ from the live vault's password. Existing backups are never overwritten; each copy is reopened and verified before success is reported.

Keep backups on a separate drive and passwords in a password manager. Backups include archive snapshots, search indexes, memories, briefs, sources, tags and client registrations. Separate original-export media attachments are not imported or included. A recovery copy on the same disk is not a substitute for an independent backup.

## In-app restore

Restore is available in **Vault settings → Restore or change password**, and under **Restore from an encrypted backup** on the lock screen. The old vault password is not required to restore a separately passworded backup.

1. Choose a standalone encrypted Pre-Drink backup and enter its password locally.
2. Select **Inspect backup**. Agent sharing stops. The app copies the backup into a new private local folder, validates encryption, supported schema, integrity and relationships, and displays its conversation/message/memory/brief counts. The selected backup is never opened for modification.
3. Review the replacement and the previous vault location. Restore replaces the active library; it does not merge libraries. Registrations from the backup are removed so old credentials cannot regain access.
4. Confirm the review, then choose **Restore verified backup**. The app verifies the prepared bytes and session again and switches the active selection. Any intervening edit, lock, expiry or changed preparation requires a new review.
5. Unlock with the **backup password**. Inspect the archive, memory and briefs. Sharing stays off; authorize current clients again only when ready.
6. Create an independent encrypted backup before considering removal of old recovery copies.

Cancelling discards the prepared copy where filesystem permissions permit. Wrong passwords, invalid/corrupt/newer-schema files, changed preparations and failures before publication leave the original selected. A completed switch followed by a folder-flush failure reports a warning instead of claiming nothing changed: preserve all folders and verify after restarting.

Restore accepts standalone files up to 8 GiB. Non-empty WAL/rollback journals are refused: use an app-created backup, not a copy of a running SQLite main file. Symbolic-link inputs and invalid active-selection paths are refused. Do not run external database writers against app storage.

## Password change

While unlocked, choose **Vault settings → Restore or change password → Change vault password**. Enter the current password, a different new password of at least 14 characters and its confirmation. The app verifies the current password, builds and verifies a separately encrypted replacement, and shows a review before switching and locking.

Unlock with the new password. Agent registrations are removed and sharing stays off. Existing backups **and the preserved previous vault still use their previous passwords**. Password change does not retroactively re-encrypt or erase those copies. If the old password was exposed, those retained copies remain exposed to someone holding that password. Keep them only as long as necessary for recovery and follow the cleanup guidance below after verification.

No password, hidden recovery key or bypass is stored. Password entry and final submission should be performed by the owner when exercising the native credential-change flow.

## Storage generations and interrupted operations

Before the first restore/password change, the database is at `~/Library/Application Support/com.predrink.vault/vault.enc.sqlite3`. After a switch, a small `active-vault.json` file selects a `vault-<UUID>/vault.enc.sqlite3` generation under that same app folder. A `vault-layout-v1` marker records successful use of this layout so a missing selector will not silently reopen the old root copy. The selector contains only a format version and a generated folder name; conversation data and credentials remain encrypted. Build details display the actual active database path.

A new generation is encrypted, verified, checkpointed and flushed before a same-directory atomic selector rename. The old database and its journals stay at their original location. There is no window where the application intentionally removes the old database before publishing the new selection. An interruption before publication keeps the previous selection; an unpublished encrypted preparation can remain after an abrupt process termination. An interruption after publication selects the new generation on restart. Hardware/filesystem failures and actual power cuts are not fully simulated by automated tests; keep independent backups.

An unreadable, malformed or missing-target selector fails closed; a missing selector also fails closed once the layout marker exists. The app must not silently fall back to an older vault or create an empty one. Older app builds do not understand this selector and may open the old root copy: **do not downgrade after a lifecycle operation**. Install a current build or follow manual recovery.

## Manual recovery and retained-copy cleanup

1. Quit Pre-Drink and its MCP bridges completely. Preserve the **entire application storage folder**, including `active-vault.json`, all generation folders and every journal. Do not replace or move live database files.
2. Keep that complete preserved folder while investigating. If recovery from a verified standalone backup is needed, move the old application folder aside, create a fresh `com.predrink.vault` folder, and copy the backup into it as `vault.enc.sqlite3`. Do not copy the old selector, layout marker or unrelated journals into this fresh folder.
3. Open a current app build and unlock with the backup password. The app applies private directory permissions. Verify the data, revoke restored registrations before enabling sharing (manual copying does not perform in-app registration cleanup), and make an independent backup.
4. If selecting a retained generation manually, preserve its database **and matching journals together**; do not copy only a potentially incomplete main file. Prefer opening an isolated copy with a current build and making a verified standalone backup before restoring it.
5. For cleanup, first record the active path from Build details and verify the current vault and an independent backup. Quit the app. Remove only a specifically identified inactive generation folder, or an identified old root database with its matching journals. Never remove the active folder, active selector, entire app folder or unknown files as a space-saving guess. There is no automatic retained-generation purge in this version.

Deletion is not secure erasure on SSDs. Original ZIPs, copied Markdown, clipboard histories, backups, filesystem snapshots and Time Machine can retain data. Password changes and archive deletion cannot revoke those copies.

## Forgotten password

There is no reset, server-held key or backdoor. A backup requires its own password. Original ChatGPT ZIPs can rebuild an archive in a fresh vault but cannot recreate manually curated memory and briefs. In-app restore preserves the inaccessible old vault rather than overwriting it.

## Legacy plaintext migration

First-run migration detects v0.1 `vault.sqlite3`, excludes concurrent writers, creates and verifies an encrypted copy, then optionally removes old database/journal files. The cleanup checkbox is on by default; retaining old files leaves them plaintext. Failure to clean up preserves the verified encrypted copy. Do not delete encrypted data to retry migration.

## Evidence and remaining limits

Automated coverage includes a complete disposable wipe-and-restore exercise, verified replacement/restart, original backup byte preservation, old/new password checks, agent reset, stale/cancel/expiry/tamper rejection, unsupported schema and active-journal refusal, permissions, an unwritable publication folder and an unpublished-stage interruption model. These use fictional data only. Actual power-loss, disk-full-at-every-write-point, full native password-change submission, broader filesystem support and personal-vault recovery require their own evidence. See [VALIDATION](VALIDATION.md).
