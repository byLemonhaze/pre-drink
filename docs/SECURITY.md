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

Schema-v3 migration rescans older vault content and clears old approvals once; subsequent unlocks retain new approvals. Edits preserve original message provenance. Agent access requires approval and excludes private/sensitive content and restricted or missing source messages/conversations. Editing a source memory blocks its linked briefs until reviewed. Deleted source memories leave orphan markers blocking the old brief. Existing links cannot be removed to bypass privacy.

Owners may write independent notes/briefs without sources. The app cannot infer where manually pasted text came from. Credential-shaped patterns and a conservative high-entropy heuristic supplement the personal-data rules. Detected credentials are restricted with no sharing override. Detection has false positives and false negatives; approval requires human review. MCP arguments cannot override private/sensitive exclusions. Local restricted-view toggles affect the owner's viewer only.

## Agent boundary

The executable's `--mcp` mode uses the official Rust SDK over stdio and forwards calls through a Unix socket in the private app directory. No HTTP/TCP listener exists. The socket cannot unlock or independently open the vault.

Clients get random 256-bit bearer tokens. Only SHA-256 hashes are stored in the encrypted vault; comparison is constant-time. Explicit owner consent issues configurations. Agent memory lists contain only id, kind, content and updated_at; memory-only packs omit archive source IDs. Default grants cover approved memory and briefs; raw chat search requires additional permission. Sharing resets on lock/restart.

Limits: 64 KiB requests, 256,000-byte responses, two-second socket inactivity/write timeouts, a three-second total request-read deadline, four active connections. The broker checks a private parent directory before binding. A repeated same-account denial of service is still possible. Unknown tools and invalid arguments fail. Agents cannot edit/approve notes, unmark private conversations, change permissions, or obtain the vault key. Chat-enabled clients may mark conversations private.

Client configuration tokens are secrets. Same-account malware may read them; permissions do not isolate malicious software running as the owner. A compromised authorized client can read its entire scope while sharing is on. Grants cover all eligible memories/briefs, not one topic or brief. Revocation cannot recall returned context.

Pre-Drink does not send the vault to the internet. A cloud-backed client may send received context to its provider; authorization explains this. History is labeled untrusted reference data, but that alone cannot eliminate prompt injection in the receiving model.

## Import and backup

ZIPs are read without filesystem extraction. JSON size and entry counts are bounded, trees validated, and imports atomic per ZIP. JSON is data, never code. Production WebView policy blocks arbitrary network destinations and remote scripts/media; raw HTML rendering is disabled.

Backups get a fresh salt and password-derived key. An encrypted staging file is exported, reopened, checked for integrity/relationships and table counts, checkpointed, flushed, and published without overwrite. Publication requires a local filesystem supporting hard links (such as APFS). The live vault remains intact on failure.

Backups retain client registrations; restoring an old backup can restore old permissions. Revoke restored clients before enabling sharing. Migration exclusively locks the legacy database and verifies its encrypted copy before optionally deleting legacy files. SSD deletion is not secure erasure. Original ZIPs, snapshots, Time Machine history and old copies can remain plaintext.

## Limits and maintenance

Encryption cannot protect an unlocked screen, screenshots, stolen passwords, privileged memory inspection, clipboard contents, approved exports, or another agent's retained context. Page authentication detects tampering but not substitution of a complete older valid database (rollback). File sizes and access timing remain visible. Keep macOS/FileVault and dependencies current and retain encrypted backups separately.

No independent security or cryptographic audit of this application has been completed. Automated verification uses synthetic data. Review dependency advisories every release, including transitive/platform warnings. Signing/notarization establishes distribution identity and integrity, not certification of the privacy model.
