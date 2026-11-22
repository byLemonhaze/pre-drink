# Pre-Drink

**Your AI history, sober and ready.**

A free, MIT-licensed macOS memory manager: provider-private histories and working memory feed a shared layer you control. Built on Tauri 2, React, TypeScript and SQLCipher/SQLite FTS5. Official ZIP import supports recognized **ChatGPT, Claude and Grok** account-export formats; one representative ChatGPT export has passed isolated backend and native post-import checks. Broader native lifecycle coverage and real Claude/Grok export checks remain open. The repository is a private pre-release until the owner chooses public launch.

**Cloud curation is optional.** Import, search, manual memory editing, briefs, selected exports and backups work offline without a model, account or API key. You can separately authorize a connected agent to curate its own provider’s history. Local-model curation is planned, not implemented.

Official imports now support encrypted pause/resume checkpoints across lock and restart. Choose the same ZIP to continue preparation, or finish without it once preparation is complete. Nothing from an unfinished import reaches search or agents. See [resumable imports](docs/IMPORTS.md).

## From private history to shared memory

| Layer                    | Purpose                                                            | Access                                                                      |
| ------------------------ | ------------------------------------------------------------------ | --------------------------------------------------------------------------- |
| Provider archive         | Original export snapshots, searchable text and alternate branches  | Owner; explicitly authorized matching-provider clients for eligible sources |
| Provider working memory  | Agent drafts, evidence, revisions and resumable job progress       | Owner and matching-provider curators                                        |
| Shared memory            | Released facts, decisions and projects, plus approved manual notes | Authorized clients read eligible summaries; no foreign raw history          |
| Briefs and context packs | Specifications and handoffs assembled from eligible knowledge      | Owner-reviewed briefs and explicit selected exports                         |

ChatGPT/Codex curates ChatGPT history, Grok curates Grok history and Claude curates Claude history. All three have separate working-memory scopes. Other providers cannot read those private drafts or raw sources. Client credentials bind the scope; they must never be reused across providers.

For manual use, choose **Keep as memory**, edit/classify the note, save, then review it. For optional agent curation, use **Agent access → Set up a connection** to choose scope, install private MCP settings and verify a permission-only request. Shared-only access is the default; explicitly choose a matching curator for drafting. Curators can save session context or work through bounded archive batches. The app provides batch review, edit/history, held undo and withdrawal. It starts no model or background job by itself.

Automatic publication is a separate opt-in, off by default. Only future eligible projects/decisions can use it; facts and recognized links/instruction-like content require review. Sensitive, stale or conflict-flagged contributions stay withheld. Automatic releases are labelled separately from owner-reviewed ones. Detection and model accuracy are imperfect. See [curation contract](docs/AGENT_CURATION.md).

These are Pre-Drink memories, independent of a provider’s built-in memory. A cloud-backed curator may send its allowed context to its provider and consume that provider’s subscription/API allowance. The offline workflow remains available without any curator.

See [the quickstart](docs/QUICKSTART.md) for the first useful memory and optional agent setup.

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

- Import official ChatGPT, Claude or Grok ZIPs using the native picker or file-drop handler. See [provider contracts and limits](docs/PROVIDERS.md).
- Recognizes ChatGPT/Claude `conversations.json` and numbered `conversations-N.json`, and Grok `prod-grok-backend.json`, including nested paths, without extracting files to disk.
- Preserves JSON snapshots and text branches across repeat imports. The latest imported snapshot sets the displayed branch; when the provider has no confirmed active leaf, a warned latest-leaf fallback is used. Manual privacy flags survive imports.
- FTS5 searches titles and text across branches using literal words combined with AND within a title or message. The conversation reader loads 20 messages at a time; its local find scans complete text in the selected branches.
- Messages over 12,000 characters start collapsed. Expanding messages over 48,000 UTF-16 code units uses plain-text parts of about 16,000 units, keeping very large Markdown out of the renderer. Stored text and search remain complete.
- Markdown supports code, lists, and tables. HTML execution, remote images, and clickable external links are disabled.
- Attachments, audio, images, tool results, and reasoning channels are not rendered or separately imported. Keep the original ZIP if you need those assets.
- Imports are atomic per ZIP, limited to 256 MiB of uncompressed conversation JSON and 100,000 ZIP entries. Earlier successful ZIPs remain if a later ZIP in a batch fails.

The sensitive-content detector checks emails, structured or explicitly labeled phone numbers, common English address patterns, medical/financial terms, credential assignments, known token shapes and high-entropy candidates. Bare epoch timestamps and generic technical mentions of passwords/API keys do not trigger a flag by themselves. It has false positives and false negatives. Review what you approve. A restricted source conversation also restricts derived memory and briefs.

**Settings → Recheck sensitive flags** previews how current rules classify already saved content. Applying the review stops agent sharing; classification changes return only affected memories and their dependent briefs to draft; unrelated approvals remain intact. Opening with stronger rules reports the added restrictions and reset count. Manual private flags remain private, detected secrets stay blocked, and reimporting alone never clears old restrictions. After you re-enable sharing, eligible raw chats are again available only to matching-provider source-enabled clients.

Official exports are the deliberate ingestion method, not a temporary shortcut. New historical transcripts require a newer export; explicit agent-authored session summaries can update memory without importing a transcript. Existing curated notes persist. See [product rationale](docs/PRODUCT.md).

## Review and share selected context

**Share context** lets you choose memories and briefs, inspect the complete outgoing Markdown and explicitly copy or save that exact text. Private, sensitive, unapproved or source-blocked items are excluded; explanations stay in the app. Every delivery rechecks the saved versions and current session. A changed item/source or lock requires a fresh preview. Existing files are never overwritten.

The file and clipboard are plaintext outside vault encryption. Mac clipboard copies stay on the current device and are marked confidential for cooperating history managers. Nothing is sent to an AI provider automatically, and this selection does not narrow existing live MCP grants. See [sharing workflow and limits](docs/SHARING.md).

If a newer import changes a source message's text or role, its linked memories return to draft and dependent briefs require source review again. An unchanged reimport preserves approvals.

## Delete archive material deliberately

The conversation viewer offers **Delete conversation…**, with an impact preview and typed confirmation. Deletion removes messages, raw snapshots and search entries; source-linked memories remain blocked drafts and dependent briefs stay blocked. **Vault settings → Archive snapshot retention** separately removes older raw JSON copies while retaining the current raw version and all message branches. These actions do not erase independent backups, old vault generations or original ZIPs.

## Encryption, backup, and agents

Initial database: `~/Library/Application Support/com.predrink.vault/vault.enc.sqlite3`. After restore/password change, `active-vault.json` selects a private generation folder under the same app directory. Build details show the active path. Do not downgrade to a build that ignores this selection. SQLCipher 4.20 encrypts archive, memory, briefs, search index, and database journal pages. Password keys use Argon2id. Read [security design and limits](docs/SECURITY.md).

The app normally locks after 15 minutes without owner activity. **Agent access → Start agent work session** explicitly enables sharing and keeps the vault unlocked for one hour, four hours, or until the Mac locks. A timed session locks at its deadline even if the app is active. Agents cannot start or extend it. Manual lock, macOS screen lock/sleep/display sleep/session deactivation, and a detected watchdog pause still close the database and disable sharing. Stopping sharing cancels the session; unlocking never resumes it.

**Vault settings → Encrypted backup** creates a separately encrypted and verified copy. **Restore or change password** prepares a verified replacement, shows its impact and preserves the previous encrypted vault before switching. Restore is also available from the lock screen. Both operations remove agent registrations, disable automatic-publication rules and leave sharing off. Old backups/recovery copies retain their old passwords; follow [recovery instructions](docs/RECOVERY.md).

**Agent access** authorizes named clients with individual revocable credentials and generates settings for Codex, Grok Build, Claude local MCP or other local MCP clients. Clients receive eligible shared memory/briefs by default; raw-chat search requires an additional matching-provider permission. Curation requires a separate write grant. The app must remain open, unlocked, and sharing enabled. No network port is opened. An authorized cloud agent may send received context to its model provider. See [MCP setup and tools](docs/MCP.md).

## Verification

Run sequentially; do not build Tauri concurrently with Cargo tests in the same target directory.

```sh
npm run build
npm run format:check
cargo test --manifest-path src-tauri/Cargo.toml --locked
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
```

Suites cover imports, branches, rollback, encryption, tampering, wrong passwords, backup, migration, approval/provenance restrictions, client permissions, revocation, idle locking, and actual stdio MCP calls. Socket integration tests need an environment allowing local Unix sockets.

Fictional native checks cover picker/Finder import, approval, exact selected export, recovery/password change, Mac lock/sleep, large-library browsing and the new curation review/edit/history flow. Codex now also passed a three-session fictional write/retry/resume/reviewed-retrieval check (27 MCP calls); see [curation acceptance](docs/CURATION_ACCEPTANCE.md). Grok Build also passed three provider-matched writing/resume/retrieval sessions (29 MCP calls), with no personal data. One representative ChatGPT export also passed isolated backend and native post-import checks. Real Claude/Grok exports and broader release gates remain open. See [VALIDATION](docs/VALIDATION.md) for evidence and limits. This is not an independent security audit.

The lock screen and Settings show the app version/build and whether the app uses normal or isolated test storage. Each vault uses its own owner-chosen password; there is no shared default password.

Generate synthetic fixtures with `python3 scripts/create_sample_export.py`. Debug builds allow `PRE_DRINK_DATA_DIR=/tmp/pre-drink-check` for isolated verification. Release builds ignore this override.

## Documentation

- [Execution plan and completion criteria](docs/EXECUTION_PLAN.md)
- [Work completed and agent handoff](docs/WORK_LOG.md)
- [Validation record](docs/VALIDATION.md)
- [Product definition, requirements and ideas](docs/PRODUCT.md)
- [Agent curation and session-memory contract](docs/AGENT_CURATION.md)
- [Curation quality and owner-effort acceptance](docs/CURATION_QUALITY.md)
- [Provider import formats and support matrix](docs/PROVIDERS.md)
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

See [review findings and responses](docs/REVIEW_RESPONSE.md) for the implemented fixes and remaining release gates. Manual saving produces a draft; use **Review & approve** to approve the saved version. On the first schema-v3 upgrade, existing contents are rescanned and previous memory/brief approvals are cleared once. Content and passwords are preserved. Developer ID signing and notarization are still required before public distribution.

Privacy recheck requires individual selections before clearing saved flags; updated detection can add restrictions when a vault opens. Settings also provides reviewed cleanup of previous encrypted vault copies, gated on a fresh verified backup. See [security review response](docs/REVIEW_RESPONSE.md) and [recovery guide](docs/RECOVERY.md) for safeguards and remaining verification gates.

Schema 4 preserves manual notes and approvals, adds encrypted curation state and turns old broad raw-chat grants into shared-only access. Schema 5 adds Claude scopes while preserving existing v4 grants, policies and history. Schema 6 completes the saved-job constraint migration for both v4 and v5 vaults, so Claude can start resumable jobs after an upgrade. Issue fresh provider-scoped registrations when needed. Never downgrade a migrated vault; keep an encrypted backup.

## Verification cost policy

Run `npm run verify:local` after staging new source files, then `npm run hooks:install` once to enable the exact-source pre-push guard. Development checks run locally. GitHub Actions is manual-only to preserve the owner's included allowance and avoid extra costs. Do not dispatch it or enable automatic triggers without explicit owner approval. The repository stays private until the owner chooses launch. See [execution plan](docs/EXECUTION_PLAN.md) for the current verification baseline and cost constraints.

## Large libraries and local connections

Memory, briefs, sharing choices and brief-source selection request 25 complete items at a time. Search covers the full saved collection, including later pages; selected sharing items stay selected when you change pages or search. Individual large records and full-scan query performance still need further profiling.

The local agent bridge and app both verify the connecting OS account before exchanging credentials. Scoped tokens and privacy/publication checks remain required. This is not model identity verification or protection from malware running as your own account. See [security](docs/SECURITY.md) and [validation](docs/VALIDATION.md).

Current release gates and evidence: [Release readiness](docs/RELEASE_READINESS.md).
