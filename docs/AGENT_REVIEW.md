# Independent agent review guide

Review the implementation as an early working native product with outstanding release gates. Do not assume it is audited or public-release ready because tests pass.

## Start here

Read README, PRODUCT, ARCHITECTURE, SECURITY, MCP and RECOVERY in that order. Source is independent of Loominary; see PROVENANCE. Current scope is ChatGPT only. User data is deliberately absent from this repository.

## Safe local verification

Use synthetic fixtures and a new temporary data directory. Never open a personal vault or request passwords/exports in a GitHub issue. Release builds ignore the data-directory override; use debug builds for isolation.

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

27 automated cases cover importer behavior, storage and privacy boundaries, encryption/backups/migration, and one real stdio MCP integration. Native synthetic-data verification covered creating a vault, picker import, approved memory, approved brief, context preview and manual lock. See VALIDATION for final command results and dependency warnings.

Not yet verified: representative real exports, actual file drag, large-library performance, every supported OS/hardware, external client setup screens, full sleep/lock-screen behavior, independent security audit. In-app restore, password rotation, dedicated Markdown file export, per-brief grants and tagging UI remain roadmap items.
