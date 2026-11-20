# Pre-Drink

**Your AI history, sober and ready.**

A free, open-source macOS vault for official **ChatGPT exports**, built with Tauri 2, React, TypeScript, and SQLCipher/SQLite FTS5. No cloud account, API key, telemetry, live capture, or automatic extraction is required.

## Three layers

| Layer   | Purpose                                                                                                 | Sharing                                                                          |
| ------- | ------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| Archive | Original conversation JSON snapshots, searchable text, alternate branches: the story behind a decision. | Raw chat access requires an explicit additional client permission.               |
| Memory  | Editable facts, decisions, and project notes linked to source messages.                                 | Explicit approval; private, sensitive, and restricted-source notes are excluded. |
| Briefs  | Curated Markdown specifications and handoffs assembled from reviewed memories and your writing.         | Explicit approval; changed or restricted sources block sharing until reviewed.   |

Select **Keep as memory** on a message, rewrite it into a useful note, choose its kind, and save. Editing leaves the original conversation intact. Open **Briefs** to assemble a specification from those notes. Approval makes content eligible for authorized clients; it does not turn sharing on.

These are Pre-Drink's memories, independent of ChatGPT's own saved-memory feature. Manual editing, search, encryption, context packs, and MCP make no paid AI calls. Cloud-assisted editing is not implemented. If added later, it must be opt-in with explicit disclosure of data sent and usage charges.

## Run on your Mac

Requirements: macOS 12+, Xcode command-line tools, Node.js 22.12+ (or a compatible newer release), and Rust 1.93+. Native verification used Apple Silicon.

```sh
npm ci
npm run tauri -- dev
```

Build an optimized local app:

```sh
npm run tauri -- build
open "src-tauri/target/release/bundle/macos/Pre-Drink.app"
```

The local bundle is not a Developer ID signed and notarized public release. See [release signing](docs/RELEASING.md).

Choose a separate vault password of at least 14 characters. Prefer a long, unique passphrase stored in your password manager. There is no password reset or hidden recovery key. Every launch starts locked. Existing plaintext v0.1 libraries have a verified encrypted migration flow.

`npm run dev` runs an interface preview only; vault operations require the native app.

## Import and search

- Import official ChatGPT ZIPs using the native picker or file-drop handler. Claude and Grok are postponed.
- Recognizes `conversations.json` and numbered `conversations-N.json`, including nested paths, without extracting files to disk.
- Preserves JSON snapshots and text branches across repeat imports. The latest imported snapshot sets the active branch; manual privacy flags survive imports.
- FTS5 searches titles and text across branches using literal words combined with AND within a title or message.
- Markdown supports code, lists, and tables. HTML execution, remote images, and clickable external links are disabled.
- Attachments, audio, images, tool results, and reasoning channels are not rendered or separately imported. Keep the original ZIP if you need those assets.
- Imports are atomic per ZIP, limited to 256 MiB of uncompressed conversation JSON and 100,000 ZIP entries. Earlier successful ZIPs remain if a later ZIP in a batch fails.

The sensitive-content detector checks emails, structured or explicitly labeled phone numbers, common English address patterns, medical/financial terms, credential assignments, known token shapes and high-entropy candidates. Bare epoch timestamps and generic technical mentions of passwords/API keys do not trigger a flag by themselves. It has false positives and false negatives. Review what you approve. A restricted source conversation also restricts derived memory and briefs.

**Settings → Recheck sensitive flags** previews how current rules classify already saved content. Applying the review stops agent sharing; any classification change returns all approved memories/briefs to draft. Manual private flags remain private, detected secrets stay blocked, and reimporting alone never clears old restrictions. After you re-enable sharing, eligible raw chats are again available to chat-enabled clients.

Official exports are the deliberate ingestion method, not a temporary shortcut. New chats require a newer export; existing curated notes persist. See [product rationale](docs/PRODUCT.md).

## Review and share selected context

**Share context** lets you choose memories and briefs, inspect the complete outgoing Markdown and explicitly copy or save that exact text. Private, sensitive, unapproved or source-blocked items are excluded; explanations stay in the app. Every delivery rechecks the saved versions and current session. A changed item/source or lock requires a fresh preview. Existing files are never overwritten.

The file and clipboard are plaintext outside vault encryption. Nothing is sent to an AI provider automatically, and this selection does not narrow existing live MCP grants. See [sharing workflow and limits](docs/SHARING.md).

If a newer import changes a source message's text or role, its linked memories return to draft and dependent briefs require source review again. An unchanged reimport preserves approvals.

## Encryption, backup, and agents

Database: `~/Library/Application Support/com.predrink.vault/vault.enc.sqlite3`. SQLCipher 4.20 encrypts archive, memory, briefs, search index, and database journal pages. Password keys use Argon2id. Read [security design and limits](docs/SECURITY.md).

The app locks manually, after 15 minutes without owner activity, on macOS sleep/display sleep/session deactivation, and after a detected watchdog pause. Locking closes the database, hides the library, and disables sharing. Agent requests do not keep it unlocked.

**Settings → Save encrypted backup** creates a separately encrypted and verified copy. Restore is currently manual: follow [recovery instructions](docs/RECOVERY.md), preserving your current vault first.

**Agent access** authorizes named clients with individual revocable credentials. Clients receive approved memory/briefs by default; raw-chat search requires additional permission. The app must remain open, unlocked, and sharing enabled. No network port is opened. An authorized cloud agent may send received context to its model provider. See [MCP setup and tools](docs/MCP.md).

## Verification

Run sequentially; do not build Tauri concurrently with Cargo tests in the same target directory.

```sh
npm run build
npm run format:check
cargo test --manifest-path src-tauri/Cargo.toml --locked
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
```

Suites cover imports, branches, rollback, encryption, tampering, wrong passwords, backup, migration, approval/provenance restrictions, client permissions, revocation, idle locking, and actual stdio MCP calls. Socket integration tests need an environment allowing local Unix sockets.

Earlier native synthetic-data verification (before the separate review screen): create password → import ZIP → create approved source-linked memory → assemble approved brief → preview context → lock and hide library. Import verification used the native picker; a physical file drag and representative real exports still need validation. The current synthetic native checks also cover separate approval, source display, edit invalidation, a zero-change flag recheck, build/vault identity and manual lock; broader failure/lifecycle cases remain open. See VALIDATION. This is not an independent security audit.

The lock screen and Settings show the app version/build and whether the app uses normal or isolated test storage. Each vault uses its own owner-chosen password; there is no shared default password.

Generate synthetic fixtures with `python3 scripts/create_sample_export.py`. Debug builds allow `PRE_DRINK_DATA_DIR=/tmp/pre-drink-check` for isolated verification. Release builds ignore this override.

## Documentation

- [Execution plan and completion criteria](docs/EXECUTION_PLAN.md)
- [Work completed and agent handoff](docs/WORK_LOG.md)
- [Validation record](docs/VALIDATION.md)
- [Product vision, requirements and ideas](docs/PRODUCT.md)
- [Architecture and design decisions](docs/ARCHITECTURE.md)
- [Interface direction](docs/DESIGN.md)
- [Security model](docs/SECURITY.md)
- [MCP integration](docs/MCP.md)
- [Backup and recovery](docs/RECOVERY.md)
- [Release signing](docs/RELEASING.md)
- [Roadmap and remaining gates](docs/ROADMAP.md)
- [Agent review guide](docs/AGENT_REVIEW.md)
- [Project provenance](docs/PROVENANCE.md)

This is an **independent implementation, not a Loominary fork**. Loominary informed the export and branch model. No Loominary source, live userscript, extension code, or branding was copied. See provenance for references and reuse policy.

## Security review follow-up

See [review findings and responses](docs/REVIEW_RESPONSE.md) for the implemented fixes and remaining release gates. Saving now always produces a draft; use **Review & approve** to approve the saved version. On the first schema-v3 upgrade, existing contents are rescanned and previous memory/brief approvals are cleared once. Content and passwords are preserved. Developer ID signing and notarization are still required before public distribution.
