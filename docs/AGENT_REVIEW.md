# Independent agent review guide

Review the implementation as an early working native product with outstanding release gates. Do not assume it is audited or public-release ready because tests pass.

## Start here

Read EXECUTION_PLAN and WORK_LOG first for current scope/status, then README, PRODUCT, ARCHITECTURE, SECURITY, MCP, RECOVERY and REVIEW_RESPONSE. Source is independent of Loominary; see PROVENANCE. Current adapters and own-provider curator roles cover ChatGPT, Grok and Claude with fictional contracts; representative exports remain open. Read AGENT_CURATION before evaluating this feature. Cloud curation is optional; verify offline manual use remains independent. User data is deliberately absent from this repository.

## Safe local verification

Use synthetic fixtures and a new temporary data directory. Never use a live personal vault for tests or request passwords/exports in a GitHub issue. Owner-directed representative-data verification may use an isolated encrypted copy after a verified backup; do not commit its contents, paths, counts, screenshots or logs. Destructive recovery tests use synthetic data only. Release builds ignore the data-directory override; use debug builds for isolation.

```sh
npm ci
npm run build
npm run test:startup
npm run test:reading
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

Not yet verified: representative real exports, real-export and UI scale performance, every supported OS/hardware, broader sleep/lock-screen coverage, independent security audit. Owner-assisted file drops and native client-configuration screens now have evidence in NATIVE_WALKTHROUGH. Restore/password change and deletion/retention are implemented; fuller native failure and hardware durability evidence remains open. Per-brief grants and tagging UI remain roadmap items.

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

Read `src-tauri/tests/fixtures/README.md` and the ChatGPT manifest. The golden examples are fictional contracts, not claims about an officially versioned export format. Review expected normalization separately from parser output, including branch IDs/nearest parents and omitted messages. Run `--test import_contract`; use `scripts/create_import_fixtures.py` only in fresh local output folders for native acceptance. Owner-assisted real file-drop checks now pass; representative-export checks remain distinct from fictional picker/drop evidence. Codex CLI and Grok Build have since passed fictional MCP policy checks; see MCP_ACCEPTANCE for exact scope. Native connection-screen verification is recorded in NATIVE_WALKTHROUGH.

## Latest recovery evidence

`tests/recovery.rs` now compares destructive manual recovery with app-managed reviewed restore after total synthetic vault loss, including restart from the selector, complete logical-table comparison and deliberate client-registration removal. `tests/lifecycle.rs` also rejects encrypted first/later-page corruption and partial-page truncation without changing the original, verifies retry, and restores from the reviewed private copy after its source disappears. Nine affected cases pass. These tests do not simulate physical power loss or every disk-full boundary. Native automation attachment failed in the latest isolated attempt; no new screen-level evidence should be inferred from it.

## Client configuration review focus

Review `client_config.rs`, `desktop::create_client`, and `Agents.tsx` together: selected format must match the returned configuration and destination instructions, failed formatting must roll back registration, paths must survive serialization without becoming shell expressions, and tokens must remain environment entries. Grok expands `${...}` in settings, so affected paths are rejected. Settings are shown locally and never written into another agent's files. Verify the native selector, consent reset, one-time display and hide/revoke distinction; compiled/read-only parser evidence does not replace that walkthrough.

## Round 3 fixes to review first

Review `privacy_recheck.rs` and `PrivacyRecheck.tsx`: explicit individual complete-item selections, limits, private/change staleness, conversation/message dependency, rollback and approval reset. Check `Vault::initialize` applies stronger detection before existing-vault agent access without automatically clearing old flags. The new tests exercise the reported plaintext shapes and opening an encrypted v3 vault with an existing chat-enabled client.

Review `retained_vaults.rs`, `RetainedVaults.tsx` and the backup receipt in Session: backup outside storage, byte/identity verification, change/session binding, active/pending exclusion, canonical generation enumeration, links/unknown files, partial-failure wording and strictly enumerated deletion. `tests/retained_vaults.rs` uses disposable encrypted vaults only. Verify native selection/confirmation and failure UI separately; automated checks do not establish that walkthrough.

Review macOS clipboard preparation in `sharing.rs`: current-host-only, confidential marker before plaintext, failure handling. The native API test uses a unique pasteboard; it is not cross-device or clipboard-manager acceptance evidence. No auto-clear timer is implemented. Keep Apple signing last and the repository private.

The new native walkthrough closes the Round 3 privacy-selection and old-copy deletion submission checks, plus native configuration, exact clipboard delivery and full app restart/sharing reset using fictional data. See NATIVE_WALKTHROUGH and the latest VALIDATION entry. The cleanup spacing adjustment was visually verified in the rebuilt Mac app. The later owner-assisted retry verifies successful native password change: old phrase rejected, new phrase accepted, generation replaced, archive/memory/brief preserved and sharing off. Clients were already revoked before rotation, so populated-registration removal has automated rather than new native evidence. The later owner-assisted Finder drops verify a valid ZIP import, omission disclosure, alternate-branch search and duplicate-free repeat import. Earlier automated gestures were inconclusive; the owner-performed drops now provide native evidence. Both fictional clients were revoked before locking and quitting. The fixture generator must never be repurposed to open personal storage.

## Screen-lock correction review

Owner-assisted testing reproduced CW-2: Control–Command–Q did not close the old app session with a native Save panel open, and its pre-lock export was still allowed. Review `mac_lock.rs` for the separate Core Foundation distributed observer, process-lifetime callback storage, immediate-delivery selection and one-way session locking. The payload and unlock signal must never grant access. The process-local native-center regression closes the session, disables sharing and blocks the pre-lock writer without broadcasting fake system events. Workspace sleep/session events and idle/gap handling remain independent safeguards; the screen-lock notification name is not a public compatibility promise and delivery may be delayed/dropped. The rebuilt app passed owner-assisted screen locking with a native Save panel open: the broker switched off and the stale export created no file. A separate owner-performed Apple-menu sleep/wake cycle also left the vault locked and sharing off. Both reopened with preserved fictional data and sensitivity. These are this host's native observations, distinct from the callback regression; other OS/hardware configurations, delivery latency and physical power-loss remain outside this evidence.

## Native scale and keyboard review

`examples/native_scale_fixture.rs` generates only fresh temporary fictional storage for native pagination/search checks. See the latest NATIVE_WALKTHROUGH and VALIDATION entries. Do not treat 5,000 short conversations as long-chat, large-memory-list or real-export coverage.

Review `Modal.tsx` and its four callers: native modal background inertness, initial/return focus, forward/reverse Tab boundaries, Escape cancellation and pending-action dismissal guards. Approval and deletion confirmations still use their existing backend review checks; opening or cancelling a modal must not approve, delete or save anything.

## Long-chat and curation browsing review

Review `PagedList.tsx`, `ConversationMessages.tsx`, `Memory.tsx` and `Briefs.tsx` for filter-before-page behavior, page reset/clamping, keyboard focus, hidden branches/restricted notes and full-source preservation. `npm run test:reading` renders real components to verify bounded output, retained-branch filtering, inert Markdown resources and explicit long-message previews. It does not simulate interactive WebView focus or replace native acceptance. Use `examples/native_reading_fixture.rs` for a separate fictional 2,000-message conversation, 3,000 memories and 500 briefs. The later `PagedConversation.tsx`/`reader.rs` change limits conversation IPC to 20 complete messages. Memory/brief result collections still load fully; one large message still has no IPC byte cap. The original rendering tests exercise shared message cards but do not test asynchronous page requests. Review stale-response rejection, error/retry, full-source search and privacy checks in the new path.

## Storage fault review

`src-tauri/src/storage_failure.rs` adds encrypted-file fault regressions. A per-connection SQLite page limit forces capacity failures during import, memory edit and brief edit; complete logical snapshots must match the previous state, reopening must succeed and retry must work after lifting the limit. A separate test spawns a writer, flushes uncommitted changes, waits for an explicit boundary marker, kills that process and reopens the vault to verify search, approvals and retry. All paths are temporary fictional storage. Review the scope carefully: these do not inject filesystem ENOSPC at backup/publication boundaries or simulate hardware power loss.

## Appearance and compact sharing review

Review `public/theme.js`, `public/theme.css` and `ThemeSwitch.tsx` for light-default initialization, constrained preference values, safe storage failures and appearance-only native permissions. No vault content belongs in localStorage. The same icon source must serve the app bundle, lock screen and sidebar. Check `VaultGate.tsx` for contained details/restore dialogs and busy guards; normal lock/first-run/error states should fit the minimum window, with internal scrolling retained for exceptional long content/accessibility. `Sharing.tsx` pages choices without dropping off-page selections; the exact outgoing preview, review checkbox and backend approval/privacy checks remain mandatory. That visual batch did not change agent permissions. The later schema-4 implementation below adds scoped writing, while self-authorized grants/publication policy remain forbidden.

## Provider curation implementation review (schema 4)

The prior documentation-only status above describes the earlier UI batch. Review the new implementation in `agents.rs`, `curation.rs`, `curation_jobs.rs`, `curation.sql`, `mcp.rs`, `Curation.tsx` and the changes to shared delivery/migration. Credential scope must be server-owned and enforced across raw search, source reads, private notes/history, privacy mutations, job manifests and writes. Check that shared-only packs never expose foreign archive metadata. Review old-grant downgrade, retained manual approvals, actual write idempotency, expected revisions, source-version invalidation, atomic batch review, safe automatic publication, attribution and undo/withdrawal.

`curation_tests.rs` covers fictional mixed-provider boundaries, replay/stale/evidence failures, all-source propagation through briefs/exports, rollback, encrypted backup and incremental resume. `tests/mcp.rs` exercises the compiled stdio transport, including the new write schema and lock denial. The separate CURATION_ACCEPTANCE result now establishes a bounded installed-Codex write/resume run; it does not establish general semantic quality. The later Grok run adds separate provider-matched writing evidence; Claude remains unverified. Real ZIP acceptance still awaits the requested export. Inspect VALIDATION/WORK_LOG for actual evidence, not plans.

## Current acceptance handoff

Prioritize an independent inspection of the provider boundary, cumulative source versions, retry receipts, publication basis and restore-policy resets. Treat all received note text as untrusted reference data. Verify cloud curation is an optional explicit grant, never a prerequisite for offline manual use. Do not assume an imported ChatGPT archive is already remembered by a new Codex session.

The three-session Codex fictional write/retry/resume/retrieval run passed; inspect CURATION_ACCEPTANCE and its event verifier before extrapolating. Grok now has separate three-session evidence (29 calls); next model acceptance should cover Claude and less scripted/more varied curation under bounded explicit usage. Test both foreign-provider directions. Native owner controls have separate evidence; broad semantic quality and arbitrary web-chat connectivity remain unverified. Keep private logs outside Git.

## Provider expansion review

Inspect `provider_import.rs`, ZIP dispatch, explicit platform persistence, schema-5/6 table rebuilds and Claude JSON registration. Review `provider_import.rs` integration tests and migration fixtures from the actual v4 schema and v5 migration. Explicitly verify Claude saved-job creation after both upgrade paths, not only registration or note writes. Verify all three source-read/write directions, same-ID collisions, unchanged reimports and restricted alternate branches. Parentless arrays use supplied order; graph active leaves are only display fallbacks. Reasoning/tools/media must stay out of reader/shared text. Do not interpret optional future delegation as an existing exception to these grants.

## Follow-up boundaries for the current batch

Review the precommit `vault-layout-v1` guard in `lifecycle.rs`, including ordinary failure cleanup and first-switch fail-closed recovery. Two child-process kill tests now cover immediately before/after selector publication; they do not simulate hardware power cuts. Confirm production builds contain no fault checkpoint.

Inspect `LargeMessageReader.tsx` for lossless Unicode boundaries, bounded inert rendering, part navigation and full-source retention. The 48,000/16,000 thresholds bound mounted text, not database/IPC allocation. Native evidence and the new fixture are separate from the SSR assertions.

Review the Grok driver and provider-general fixture: temporary authentication must not modify the real cache; only the three allowed vault tools and discovery may execute; counters/deadlines and cleanup must hold. The actual run used the prior initial auth-link setup, while the final code seeds a private copy. No model rerun is claimed for that filesystem refinement. Broader curation usefulness is specified in CURATION_QUALITY, not established by the small functional fixture. QUICKSTART documents existing setup rather than claiming an implemented wizard. See REVIEW_RESPONSE for disposition of the owner's latest product review.

## Guided setup and quality corpus follow-up

Review `AgentSetup.tsx`, `client_config.rs`, `broker.rs` and `session.rs`. Setup must default to shared-only, reset consent on scope/application changes, expose a token only during setup, and never imply that settings alone establish connectivity. `connection_status` returns permissions only. Owner verification stores successful client IDs for the current unlock, not request/response content or model identity; lock clears it. Existing installed-model runs predate this wizard.

The ten-case `quality-v1` corpus has separate expected/must-not criteria, including cross-conversation correction, consolidation, uncertainty, hostile reference text, French and literal project-name retrieval. Packaging/import verification does not establish model quality. See CURATION_QUALITY for the unscored protocol and consent boundary.

Review `ChoiceSelect.tsx` for the owner-reported menu remaining above Chrome after minimizing Pre-Drink. No HTML `<select>` should recreate an OS popup; options stay in the WebView. Check selection versus highlight, Escape bubbling, Tab focus, long source labels, window blur/minimize, light/dark and modal scrolling. The component/static regression checks do not substitute for native interaction evidence.
