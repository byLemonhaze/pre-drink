# Security design

This records implemented controls, not a guarantee of invulnerability. The primary threat addressed is disclosure of a copied vault or backup while locked. A compromised Mac, unlocked app, or authorized agent is a separate boundary.

## Storage format 1

- SQLCipher 4.20 / SQLite 3.54.1, built into the app with vendored OpenSSL. User vaults never fall back to plaintext SQLite.
- SQLCipher page encryption: AES-256-CBC, fresh random page IVs, HMAC-SHA512 authentication. Database and journal page contents include encrypted search indexes, raw JSON, memories, briefs, and client records.
- Argon2id v19: 65,536 KiB memory, 3 passes, 1 lane, 32-byte key. Each database has a random 16-byte salt in SQLCipher's normal public header position.
- Pre-Drink passes the derived key and salt through SQLCipher's documented raw-key API. It does not use SQLCipher's default PBKDF2 password derivation for the vault password. No key file, saved password, or recovery secret is created.
- Format/KDF parameters are fixed in `crypto.rs`; future changes require explicit migration, not silent parameter changes.
- New passwords require at least 14 characters and at most 1,024 UTF-8 bytes. Length is not entropy: choose a long unique passphrase. App-side delays cannot prevent offline guesses against a copied database.
- SQLCipher memory-security mode and secure-delete enabled; temporary database storage stays in memory. Rust password/key buffers use best-effort zeroization. The WebView, OS, imported strings, and all other allocations cannot be promised perfectly erased.
- Directory mode 0700; new database/backup/socket mode 0600. SQLite shared-memory bookkeeping may reveal page numbers and operational metadata, not plaintext journal page content.

References: [SQLCipher design](https://www.zetetic.net/sqlcipher/design/), [raw-key API](https://www.zetetic.net/sqlcipher/sqlcipher-api/), [RustCrypto Argon2](https://docs.rs/argon2/0.5.3/argon2/).

## Lock boundary

Startup is locked. Lock closes the database connection, disables sharing, invalidates pending imports, and emits a frontend event to unmount visible data; polling backs up the event. Passwords never go to MCP clients.

Owner inactivity locks after 15 minutes. The watchdog locks after a pause over 30 seconds. Native observers handle `NSWorkspaceWillSleepNotification`, `NSWorkspaceScreensDidSleepNotification`, and `NSWorkspaceSessionDidResignActiveNotification`. These are sleep/display/session events, not instantaneous detection of every macOS lock-screen configuration. Use Lock when stepping away. Agent calls do not extend the owner's idle deadline.

Operations serialize on a session mutex. An in-progress transaction or bounded socket response can finish before a waiting lock acquires it. Import parsing outside the mutex cannot commit after the session generation changes.

## Approval and provenance

Saving a new or existing note/brief always creates a draft; save commands reject approval fields. A separate owner-only review shows saved content and current supporting notes. Approval compares a SHA-256 snapshot of that record and its source states, and rejects changes since review. The hash binds a review version; it is not an authentication credential. The session mutex covers comparison and approval.

Schema-v3 migration rescans older vault content and clears old approvals once; subsequent unlocks retain new approvals. Edits preserve original message provenance. Agent access requires approval and excludes private/sensitive content and restricted or missing source messages/conversations. Editing a source memory blocks its linked briefs until reviewed. A changed imported message text/role revokes its linked memories and advances their version, blocking dependent briefs; identical reimports preserve approvals. Deleted source memories leave orphan markers blocking the old brief. Existing links cannot be removed to bypass privacy.

Owners may write independent notes/briefs without sources. The app cannot infer where manually pasted text came from. Credential-shaped patterns and a conservative high-entropy heuristic supplement the personal-data rules. Detected credentials are restricted with no sharing override. Detection has false positives and false negatives; approval requires human review. MCP arguments cannot override private/sensitive exclusions. Local restricted-view toggles affect the owner's viewer only.

## Sensitive classification and recheck

Bare digit strings (including epoch timestamps) and technical mentions of passwords/API keys are not sufficient by themselves to flag content. Structured or explicitly labeled telephone numbers (including “call me at”), personal-data terms, natural-language credential assignments, short PIN/passcodes, recognizable secrets and high-entropy candidates remain restricted. Rules are heuristic; unformatted/unlabeled personal numbers and unfamiliar secret formats may be missed. This change does not add an owner override for detected credentials.

Old restrictions are not cleared just by opening the app or importing again. Opening applies stronger rules before returning the unlocked vault: new restrictions propagate from all messages, including inactive branches, to their conversation, and revoke memory/brief approvals when anything becomes newly restricted. Unchanged classifications do not reset approvals.

The owner may preview a recheck in Settings. Its hash binds saved contents, prior flags and the connection change counter. Clearing requires individually selected complete items; no selection clears nothing. Up to 100 candidates of at most 16,000 UTF-8 bytes each are shown per batch. Larger items remain restricted, never silently truncated for approval. Unshown, duplicate or currently detected-sensitive IDs are rejected. A conversation cannot clear while any of its messages remains flagged. Applying stops agent sharing and updates selected clearances/new restrictions atomically. Any actual classification change clears all memory/brief approvals. Manual private flags, source links and raw contents remain unchanged. Cleared conversations can become available to existing chat-enabled clients if the owner later enables sharing; the confirmation explains this.

Previous encrypted generations are retained until the owner reviews deletion. Cleanup requires a fresh verified backup outside the vault folder in the current unlocked session, unchanged backup bytes/identity and unchanged vault data, typed confirmation and an integrity check of the active vault. It excludes the active generation and pending replacements; accepts only canonical generation names or the original root database group; rejects links, special files and unexpected directory contents; and removes only exact known files. It is not recursive deletion or SSD erasure. Unrelated original exports, backups and system snapshots are outside cleanup. External same-user writers/malware are outside this app's protection boundary; do not modify vault files concurrently. See [RECOVERY](RECOVERY.md) for workflow and partial-failure behavior.

## Agent boundary

The executable's `--mcp` mode uses the official Rust SDK over stdio and forwards calls through a Unix socket in the private app directory. No HTTP/TCP listener exists. The socket cannot unlock or independently open the vault.

Clients get random 256-bit bearer tokens. Only SHA-256 hashes are stored in the encrypted vault; comparison is constant-time. Explicit owner consent issues configurations. Agent memory lists contain only id, kind, content and updated_at; memory-only packs omit archive source IDs. Default grants cover approved memory and briefs; raw chat search requires additional permission. Sharing resets on lock/restart.

Limits: 64 KiB requests, 256,000-byte responses, two-second socket inactivity/write timeouts, a three-second total request-read deadline, four active connections. The broker checks a private parent directory before binding. A repeated same-account denial of service is still possible. Unknown tools and invalid arguments fail. Agents cannot edit/approve notes, unmark private conversations, change permissions, or obtain the vault key. Chat-enabled clients may mark conversations private.

Client configuration tokens are secrets. Generation returns JSON or TOML for the chosen local client, keeps tokens in environment entries rather than process arguments, and never edits client settings automatically. Registration and serialization are one transaction: unsupported paths or formatting failure leave no newly authorized client. Unsupported control/non-Unicode paths and Grok environment-expansion expressions fail closed. Same-account malware may read them; permissions do not isolate malicious software running as the owner. A compromised authorized client can read its entire scope while sharing is on. Grants cover all eligible memories/briefs, not one topic or brief. Revocation cannot recall returned context.

Pre-Drink does not send the vault to the internet. A cloud-backed client may send received context to its provider; authorization explains this. History is labeled untrusted reference data, but that alone cannot eliminate prompt injection in the receiving model.

## Import and backup

ZIPs are read without filesystem extraction. JSON size and entry counts are bounded, trees validated, and imports atomic per ZIP. JSON is data, never code. Production WebView policy blocks arbitrary network destinations and remote scripts/media; raw HTML rendering is disabled.

Backups get a fresh salt and password-derived key. An encrypted staging file is exported, reopened, checked for integrity/relationships and table counts, checkpointed, flushed, and published without overwrite. Publication requires a local filesystem supporting hard links (such as APFS). The live vault remains intact on failure.

Backups retain client registrations. In-app restore and password change delete registrations from the prepared replacement before activation and leave sharing off. Manual restoration can restore old permissions; revoke those registrations before enabling sharing. Migration exclusively locks the legacy database and verifies its encrypted copy before optionally deleting legacy files. SSD deletion is not secure erasure. Original ZIPs, snapshots, Time Machine history and old copies can remain plaintext.

## Limits and maintenance

Encryption cannot protect an unlocked screen, screenshots, stolen passwords, privileged memory inspection, clipboard contents, approved exports, or another agent's retained context. Page authentication detects tampering but not substitution of a complete older valid database (rollback). File sizes and access timing remain visible. Keep macOS/FileVault and dependencies current and retain encrypted backups separately.

No independent security or cryptographic audit of this application has been completed. Automated verification uses synthetic data. Review dependency advisories every release, including transitive/platform warnings. Signing/notarization establishes distribution identity and integrity, not certification of the privacy model.

## Selected export boundary

Owner-only selective sharing accepts saved item identifiers, never a frontend-provided payload to write. Assembly excludes private, sensitive, unapproved and source-blocked content, checks current detector rules, omits raw source metadata and caps the final payload at 128,000 bytes without silent truncation. A content/source snapshot hash and unlock generation bind the preview. Copy/save assemble again and reject stale versions or sessions before emitting bytes. The session mutex covers validation and output, so a competing lock/edit executes before or after the complete bounded operation. It cannot recall output already delivered.

Exports are deliberately plaintext. New Markdown files are created with mode 0600 and `create_new`; existing files and final-path symlinks are refused. Partial write failures attempt to remove the new file; OS/filesystem failure can prevent cleanup. Clipboard delivery replaces the clipboard with current-host-only content and writes the confidential marker before the text. Cooperating history managers can honor that marker; other apps can still retain text. No automatic clipboard erasure, remote recall or protection for synced destinations is promised. This screen does not reduce live MCP grant scope. No private exclusion text is included in the outgoing payload. Reference quoting is a presentation aid, not a prompt-injection guarantee.

## Lifecycle publication and deletion

Restore/password changes operate on a separately encrypted generation. Preparation contains no stored password; apply requires the backup/new password again from the local form, validates the prepared bytes and binds the operation to the active session and connection changes. A fresh directory and file use 0700/0600. The selector contains only a generated UUID directory name, is mode 0600 and is published by atomic rename after flushing. Strict resolution refuses malformed selectors, traversal, symlink generations and missing targets. Old files/journals remain intact for recovery. This preserves recoverability but also preserves copies decryptable with old passwords. Their removal is a separate deliberate action; password rotation does not erase SSD/filesystem snapshots or backups.

The app serializes lifecycle operations with all other owner/MCP operations. Pending imports and sharing previews are invalidated by the session generation switch. Failed preparation leaves the original selected. A post-publication directory-flush error is reported as a completed switch with a warning; it cannot truthfully be reported as an unchanged vault. Tests model interrupted/unpublished state and no-write failure, not every hardware power-loss outcome. Same-user external database writers and malicious same-account file replacement remain outside the app's isolation guarantees.

Archive deletion has a session/version-bound review and typed confirmation. Its transaction removes raw snapshots, messages, FTS and conversation/tag links while leaving derived records blocked by missing evidence. Snapshot pruning removes older raw JSON only. Neither path is a secure-erasure promise or an implicit purge of other encrypted generations. No lifecycle/deletion commands are exposed over MCP.

## Interface recovery

Main-page navigation/reload locks the backend session and disables sharing. A React render-error boundary hides failed content and requires a successful backend lock before enabling its reload button. If locking fails, it instructs the owner to quit and reopen. The bootstrap fallback runs before the application module graph and does not display exception contents. The native startup timeout uses an app-attached dialog and ignores a delayed response after startup has succeeded. Debug timing is opt-in and contains fixed phase labels and elapsed milliseconds only; release builds do not emit it. These recovery controls are not protection against a compromised renderer or operating system.
