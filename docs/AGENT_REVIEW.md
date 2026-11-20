# Independent agent review guide

Review the implementation as an early working native product with outstanding release gates. Do not assume it is audited or public-release ready because tests pass.

## Start here

Read EXECUTION_PLAN and WORK_LOG first for current scope/status, then README, PRODUCT, ARCHITECTURE, SECURITY, MCP, RECOVERY and REVIEW_RESPONSE. Source is independent of Loominary; see PROVENANCE. Current scope is ChatGPT only. User data is deliberately absent from this repository.

## Safe local verification

Use synthetic fixtures and a new temporary data directory. Never use a live personal vault for tests or request passwords/exports in a GitHub issue. Owner-directed representative-data verification may use an isolated encrypted copy after a verified backup; do not commit its contents, paths, counts, screenshots or logs. Destructive recovery tests use synthetic data only. Release builds ignore the data-directory override; use debug builds for isolation.

```sh
npm ci
npm run build
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

Not yet verified: representative real exports, actual file drag, real-export and UI scale performance, every supported OS/hardware, external client setup screens, full sleep/lock-screen behavior, independent security audit. In-app restore, password rotation, per-brief grants and tagging UI remain roadmap items.

## Immediate review focus

M1 starts with structured phone/context detection, atomic owner-reviewed reclassification, persistent manual privacy and prevention of silent sharing after a flag clears. Check reimport behavior too. Verify that applying even a stale recheck turns sharing off and that source-derived briefs return to draft after a classification change. Verify build/vault identity against the actual launched binary rather than assuming similarly named bundles use the same storage. See EXECUTION_PLAN for M1's remaining native/export/client/upgrade gates.

## Current sharing/import/recovery review focus

Review `sharing.rs` and `Sharing.tsx`: exact byte equality between preview and copy/save; eligibility/exclusion handling; source and unlock-generation staleness; no archive metadata in payloads; 50-item/128,000-byte bounds; no overwrite; clipboard/file error handling. Verify that the UI distinguishes a selected export from live MCP access. Review the message-evidence trigger and transactional batch FTS rebuild, including identical/changed reimports, duplicate IDs and retry after rollback. `tests/recovery.rs` exercises complete synthetic vault removal/restoration; lifecycle UI is still absent. `tests/benchmark.rs` is ignored in ordinary CI and must be requested explicitly. See VALIDATION for measured environment/results and remaining native checks.
