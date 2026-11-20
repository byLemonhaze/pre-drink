# Independent agent review guide

Review the implementation as an early working native product with outstanding release gates. Do not assume it is audited or public-release ready because tests pass.

## Start here

Read EXECUTION_PLAN and WORK_LOG first for current scope/status, then README, PRODUCT, ARCHITECTURE, SECURITY, MCP, RECOVERY and REVIEW_RESPONSE. Source is independent of Loominary; see PROVENANCE. Current scope is ChatGPT only. User data is deliberately absent from this repository.

## Safe local verification

Use synthetic fixtures and a new temporary data directory. Never use a live personal vault for tests or request passwords/exports in a GitHub issue. Owner-directed representative-data verification may use an isolated encrypted copy after a verified backup; do not commit its contents, paths, counts, screenshots or logs. Destructive recovery tests use synthetic data only. Release builds ignore the data-directory override; use debug builds for isolation.

```sh
npm ci
npm run build
npm run test:startup
npm run format:check
cargo test --manifest-path src-tauri/Cargo.toml --locked
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
python3 scripts/create_sample_export.py
npm run tauri -- build --debug
PRE_DRINK_DATA_DIR=/tmp/pre-drink-review ./src-tauri/target/debug/pre-drink
```

Run Cargo/build steps sequentially. Socket tests require local Unix sockets. Use a unique temporary directory if the example already exists. Review dependency advisories with npm audit and cargo audit. Do not add private fixtures or generated artifacts to Git.

## Priority review areas

1. **Crypto:** raw-key/salt API correctness, KDF format stability, WAL/temp behavior, error paths, cleanup, corrupt-page handling, password lifecycle. Verify with an independently built SQLCipher reader if feasible.
2. **Lock lifecycle:** transactions/imports in flight, idle owner activity, sleep/resume/session deactivation, app restart and source UI unmount. Verify behavior on actual lock-screen configurations; observers are not a universal instantaneous-lock guarantee.
3. **Policy:** privacy/approval on every tool, live source changes, orphaned sources, brief version invalidation, manual-source ambiguity, restricted viewer not becoming an export override.
4. **MCP:** real external client configuration, malformed JSON, oversized input, auth/scope/revocation, stalled clients, no password propagation, local socket and instance-lock races.
5. **Import:** representative redacted or locally held real exports, provider format drift, duplicate IDs, split ZIPs, malformed graphs, large archives, repeated imports and snapshot growth.
6. **Backup/migration:** interruption before/after publication, wrong backup password, target collision, old WAL state, verified plaintext migration, restoration of older client grants, disk-full behavior.
7. **Frontend:** untrusted Markdown, CSP, private source navigation, stale async updates after lock, approval resets on edit, sensitive detection feedback, keyboard access.
8. **Distribution:** Developer ID, hardened runtime, notarization and Gatekeeper. Never treat encryption as a substitute for signing.

Report findings with severity, exact code path, reproduction, impact, and a proposed fix. Separate proven bugs from hypotheses and usability wishes. Do not submit user data or credentials as evidence.

## Evidence and limitations

The baseline at `589967e` had 35 automated cases covering importer behavior, storage and privacy boundaries, encryption/backups/migration, and one real stdio MCP integration. Earlier native synthetic-data verification covered creating a vault, picker import, approved memory, approved brief, context preview and manual lock. A later M1 native run verifies basic separate approval, source display, edit invalidation, zero-change flag recheck, identity and lock; see the explicit scope in VALIDATION. See VALIDATION for final command results and dependency warnings.

Not yet verified: representative real exports, actual file drag, real-export and UI scale performance, every supported OS/hardware, external client setup screens, full sleep/lock-screen behavior, independent security audit. Restore/password change and deletion/retention are implemented; fuller native failure and hardware durability evidence remains open. Per-brief grants and tagging UI remain roadmap items.

## Immediate review focus

M1 starts with structured phone/context detection, atomic owner-reviewed reclassification, persistent manual privacy and prevention of silent sharing after a flag clears. Check reimport behavior too. Verify that applying even a stale recheck turns sharing off and that source-derived briefs return to draft after a classification change. Verify build/vault identity against the actual launched binary rather than assuming similarly named bundles use the same storage. See EXECUTION_PLAN for M1's remaining native/export/client/upgrade gates.

## Current sharing/import/recovery review focus

Review `sharing.rs` and `Sharing.tsx`: exact byte equality between preview and copy/save; eligibility/exclusion handling; source and unlock-generation staleness; no archive metadata in payloads; 50-item/128,000-byte bounds; no overwrite; clipboard/file error handling. Verify that the UI distinguishes a selected export from live MCP access. Review the message-evidence trigger and transactional batch FTS rebuild, including identical/changed reimports, duplicate IDs and retry after rollback. `tests/recovery.rs` exercises complete synthetic vault removal/restoration; lifecycle replacement and archive-removal UI now exist with dedicated regression suites. `tests/benchmark.rs` is ignored in ordinary CI and must be requested explicitly. See VALIDATION for measured environment/results and remaining native checks.

## Lifecycle review focus

Review `lifecycle.rs`, `Session::path/lock`, and `VaultMaintenance.tsx`: supported-schema and standalone-backup validation, no mutation of chosen backups, current-password authentication, prepared digest/session/change-counter binding, reset of restored credentials, strict selector resolution, old-vault/journal preservation, publication/flush error distinctions and cancellation cleanup. Verify recovery from both locked and empty storage and repeated rotations. The selector is a new storage-layout boundary; older builds must not be used after switching. Inspect retained-copy disclosure and manual recovery instructions.

Review `retention.rs` and `ArchiveRemoval.tsx`: typed owner confirmation, transaction rollback, FTS/snapshot deletion, missing-source markers after reimport, and distinction between older raw snapshots and retained message branches. Tests model unwritable publication and unpublished preparations, not every disk-full or physical power-loss point. Native password change should be entered/submitted by the owner, using a disposable fictional vault for review. No personal passwords should be requested in repository issues or chat.

## Startup review focus

A static local loading/retry screen starts independently of React. Application-module rejection and a 10-second bootstrap timeout produce a recoverable message; a render-error boundary hides the failed UI and locks the vault before offering reload. A native 15-second watchdog offers retry/close if even the bootstrap never reports ready, and ignores a delayed response once the interface has started. Every main-page navigation locks the session and stops sharing. Verify failure recovery as well as ordinary launches; do not equate automation attachment time with app startup latency. Debug builds accept `PRE_DRINK_STARTUP_TRACE=1` for fixed phase names and elapsed milliseconds only, with no paths, errors, credentials or vault contents. Do not enable broader private-content logs.

## Import contract review focus

Read `src-tauri/tests/fixtures/README.md` and the ChatGPT manifest. The golden examples are fictional contracts, not claims about an officially versioned export format. Review expected normalization separately from parser output, including branch IDs/nearest parents and omitted messages. Run `--test import_contract`; use `scripts/create_import_fixtures.py` only in fresh local output folders for native acceptance. Real file-drop and representative-export checks remain distinct from the successful picker evidence. Codex CLI and Grok Build have since passed fictional MCP policy checks; see MCP_ACCEPTANCE for exact scope. Native connection-screen verification remains open.

## Latest recovery evidence

`tests/recovery.rs` now compares destructive manual recovery with app-managed reviewed restore after total synthetic vault loss, including restart from the selector, complete logical-table comparison and deliberate client-registration removal. `tests/lifecycle.rs` also rejects encrypted first/later-page corruption and partial-page truncation without changing the original, verifies retry, and restores from the reviewed private copy after its source disappears. Nine affected cases pass. These tests do not simulate physical power loss or every disk-full boundary. Native automation attachment failed in the latest isolated attempt; no new screen-level evidence should be inferred from it.
