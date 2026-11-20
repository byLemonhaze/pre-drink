# Backup and recovery

## Backup

In **Settings → Save encrypted backup**, choose a strong password and a new filename on APFS or another local filesystem supporting hard links. The password can differ from the live vault's password. Existing backups are never overwritten; each copy is reopened and verified before success is reported.

Keep backups on a separate drive and passwords in a password manager. Backups include archive snapshots, search indexes, memories, briefs, sources, tags, and client registrations. Separate original-export media attachments are not imported or included.

## Restore (manual in this version)

1. Quit Pre-Drink completely and stop MCP bridges. Never replace live database files.
2. In Finder's **Go → Go to Folder**, open `~/Library/Application Support/`.
3. Move the entire existing `com.predrink.vault` folder to a safe local location. Preserve its database, `-wal`, and `-shm` files together. Do not delete it; a copied live database without its journal can be incomplete.
4. Create a fresh `com.predrink.vault` folder in Application Support.
5. **Copy** the verified backup into it as `vault.enc.sqlite3`. Retain the original backup. Never mix journal files from different databases.
6. Open Pre-Drink, which applies owner-only directory permissions and starts locked. Unlock with the **backup's** password.
7. Verify archive, memory and briefs. Sharing remains off. Revoke restored client registrations before authorizing current clients and enabling sharing.
8. Keep the previous folder until satisfied. If recovery fails, quit and preserve both folders before investigating.

Restoring an older backup restores older data and permissions. This replaces a vault; it does not merge libraries. In-app restore/password rotation are not implemented yet.

## Forgotten password

There is no reset, server-held key or backdoor. A backup requires its own password. Original ChatGPT ZIPs can rebuild the archive in a fresh vault but cannot recreate manually curated memory and briefs.

## Legacy plaintext migration

First-run migration detects v0.1 `vault.sqlite3`, excludes concurrent writers, creates and verifies an encrypted copy, then optionally removes old database/journal files. The cleanup checkbox is on by default; retaining old files leaves them plaintext.

Deletion does not erase snapshots, Time Machine history, original ZIPs or earlier copies. If encryption succeeds but cleanup fails, preserve the encrypted file, verify its contents, quit, and remove only the known legacy files. Never delete the encrypted vault to retry migration without preserving it first.

## Recovery evidence

A destructive synthetic acceptance test now creates an encrypted vault containing branches, source-linked approved notes/briefs, private/sensitive records and a client registration. It verifies a separately passworded backup, closes and removes the entire disposable live directory, copies the backup into a fresh directory and unlocks a new session. It compares all logical tables including snapshots, tags and FTS; checks privacy/provenance filtering, correct password, file permissions, sharing-off and restored-client revocation. The backup remains intact. This verifies the documented manual restoration path with fictional data, not an in-app restore interface or recovery of a personal vault.
