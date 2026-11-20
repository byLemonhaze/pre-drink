# Work and handoff log

Updated 2026-11-19. This is an implementation/evidence record, not a record of private conversations or credentials.

## Product baseline

Pre-Drink is an independent Tauri/React macOS app, with SQLCipher/Argon2id storage and official ChatGPT ZIP ingestion. It preserves an archive, lets the owner curate memory and briefs, and exposes eligible context through authenticated local MCP. Free local features need no model API. The repository remains private until the owner chooses publication. Claude/Grok, selective file export, in-app restore and password change are not implemented at this baseline.

## Completed implementation batches

| Batch                               | Implementation                                                                                                                                                                                                                     | Evidence and limits                                                                                                                                                                                                                                |
| ----------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Encrypted vault and curated context | ChatGPT ZIP parser, branches/snapshots, SQLite FTS5, editable memory/briefs, provenance restrictions, encrypted backup/migration, local authenticated MCP, startup/idle/macOS lifecycle locking                                    | Synthetic importer/security/MCP checks; earlier native curation workflow. Real exports, full lock-screen coverage and release signing remain open.                                                                                                 |
| `011294d` repository verification   | Switched CI to the supported macOS runner configuration used by the project                                                                                                                                                        | Workflow configuration committed; remote execution is separate evidence from local tests.                                                                                                                                                          |
| `a31608c` dark interface            | Graphite system panels, restrained amber accents, matching static landing assets and app icons                                                                                                                                     | Native/static layout inspection recorded; landing stays unpublished.                                                                                                                                                                               |
| `589967e` review response           | Credential shapes/entropy checks, minimal agent memory response, separate saved-version approval, schema-v3 rescan/one-time approval reset, socket request deadline/private-parent check, CI audits and release integrity metadata | 35 local tests passed, frontend/format/Clippy passed, debug bundle built. Backup test also checks approval persistence. New review UI did not complete native verification because automation lost connection. See VALIDATION and REVIEW_RESPONSE. |

These batches establish working code, not completion of the public-release gates. An AI-assisted external review identified useful defects; it is not a formal cryptographic/security certification.

## Current work

**M1 — In progress.** The first implementation batch adds structured/contextual phone detection, removes bare technical credential-word false positives, adds an explicit atomic sensitivity recheck, preserves restrictions across reimports, and displays build/vault identity. The plan, roadmap and handoff are reconciled. Forty-three local automated tests, frontend build, formatting and Clippy pass. A separately named native debug build subsequently verified memory/brief approval, displayed source review, edit-to-draft and linked-brief blocking, an unflagged timestamp, recheck preview/apply with no changes, build/storage identity and manual lock. See EXECUTION_PLAN for requirements and completion criteria. Do not mark M1 complete until the native, real-client, migration and representative-import evidence is recorded.

## Next milestones

1. M1: dependable ChatGPT workflow and verification.
2. M2: exact outgoing-content preview, selective copy/Markdown export.
3. M3: restore/password change/delete/retention, including a disposable wipe-and-restore test.
4. M4: Claude then Grok, with synthetic fixtures per provider/observed export format.
5. M5: signed release, clean installation, fictional-data demo, README and owner-approved publication.

Apple enrollment is an owner action deferred until the final release stage; its status has not been confirmed. Do not report it as started.

## Open evidence / handoff

- Native two-step review now has synthetic evidence for the basic successful flow and edit invalidation. Full keyboard, concurrent-change, source-privacy and sleep/lock coverage still need their own native checks.
- Approval reset on an isolated encrypted copy of a representative old vault; never manipulate the live schema version.
- False-positive corrections applied to already flagged data without silently widening sharing.
- Representative official exports and physical file drop.
- One installed MCP client beyond the automated stdio harness.
- Real-export/UI scale behavior and complete native lifecycle/lock checks; the normalized synthetic backend benchmark now passes.
- Full recovery UI and credential-change failure paths; the destructive synthetic manual-restore acceptance test now passes.
- Developer ID/notarization and clean-machine installation.
- Remote CI passed at `b2ce46a` ([run](https://github.com/byLemonhaze/pre-drink/actions/runs/44645007179)); the sharing/import/recovery batch at `be16955` also passed ([run](https://github.com/byLemonhaze/pre-drink/actions/runs/44652417525)); the lifecycle commit `f48a7f6` passed ([run](https://github.com/byLemonhaze/pre-drink/actions/runs/44661395645)). Local pass counts are not evidence of remote success.

## Entry format for subsequent changes

For each completed batch record: milestone/task; commit; behavior changed; checks and environment; private-data handling (only pass/fail, no contents/paths); unresolved limitations; next task. Keep implementation status separate from native/release verification. Link detailed changes to their owning documents instead of duplicating contradictory status claims.

## M1 first batch — implementation and automated evidence

- **Behavior:** timestamps and ordinary technical mentions no longer flag by themselves; formatted/labeled phones, assigned credentials and recognizable/high-entropy secrets remain restricted. Settings can preview/apply a content-bound recheck. Applying disables sharing even on error; changed classifications clear approvals, preserve manual privacy/provenance, and require review. Reimports retain old sensitivity until explicit recheck.
- **Identity:** lock screen and Settings show version, source revision/local-change marker, build timestamp/profile, storage mode and an expandable local path. Release profile is not a signing claim.
- **Evidence:** 43 local automated cases (9 unit, 19 security, 14 importer/vault, 1 real stdio MCP), production frontend build, format and Clippy checks passed. Added regressions for timestamps/technical discussion, structured phones/assignments, recheck rollback/staleness, retained secret branches, reimport restrictions, sharing stop and build/storage identity.
- **Data handling:** automated tests use synthetic data. No real vault was migrated, wiped or uploaded in this batch.
- **Native evidence:** synthetic memory approval, brief approval with source display, note edit resetting approval and blocking the brief, timestamp remaining unflagged, zero-change recheck application, build/test-storage identity and manual lock passed. Changed-classification and failure cases are covered by automated backend tests, not yet the native UI. The review build initially displayed a blank window after a slow start, then loaded; cold-start timing remains an investigation item.
- **Remaining:** representative isolated-copy upgrade, real export/physical-drop/large-library and installed-client checks; broader native failure/keyboard/lifecycle cases. M1 remains in progress. M2–M5 remain planned.

## Selected sharing, import performance and recovery acceptance batch

- **M1:** measured the 5,000-conversation/100,000-message encrypted normalized import. Replaced one FTS scan per conversation with a single transactional batch rebuild using memory-only temporary IDs. Same local debug dataset: import 80.732 → 3.719 seconds; repeat 169.483 → 4.004 seconds. Search after the change: 0.106–0.198 seconds. Rollback/retry, unaffected conversations, retained branches and duplicate versions pass. This excludes ZIP parsing, native rendering and real-export complexity.
- **Source review:** changed imported message text/role now resets its linked memories to draft and advances their version, blocking dependent briefs. Identical imports preserve approvals. A regression verifies both behaviors and stale sharing rejection. The existing recheck rollback test explicitly accounts for this earlier invalidation before testing its own rollback.
- **M2 implemented:** Share context chooser, complete Markdown preview, included/excluded counts, local exclusion reasons, explicit reviewed-content consent, native clipboard and new-file `.md` export. The backend revalidates item/source snapshots and unlock generation under the same session mutex as delivery. Private/sensitive/draft/source-blocked material is excluded; no archive provenance or exclusion notices enter the payload. No change to live MCP grants. See SHARING/SECURITY.
- **M2 native evidence:** isolated fictional vault; approved note plus blocked brief produces one included/one excluded; explanation visible only locally; copy succeeds and paste into an unsaved editor matches the preview; native file save succeeds and an independent byte comparison matches the complete 207-byte preview with mode 0600. Zero-eligible selection disables export; changing selection clears the preview; manual lock hides it. The paste-check editor was cancelled. Real clipboard-denial/disk-full/concurrent native interaction scenarios remain unverified; corresponding backend delivery-error/staleness checks pass.
- **M3 early evidence:** destructive recovery acceptance test creates a synthetic encrypted vault with branches, source-linked notes/briefs, privacy/sensitive flags and client registration; verifies backup under a different password; deletes the entire synthetic live directory and journals; restores into a fresh directory; compares all logical tables and verifies restrictions, sharing-off, correct password, permissions and revocation. In-app restore, password change and deletion/retention remain planned.
- **Checks:** 50 automated cases passed (9 unit, 15 importer/vault, 19 security, 5 sharing, 1 recovery, 1 actual stdio MCP), plus the separately invoked scale benchmark. Frontend build, formatting, Clippy with warnings denied and the separate native debug bundle passed. No dependency versions changed; only an existing macOS framework feature was enabled for clipboard support. Prior advisory results remain time-specific.
- **Privacy/release:** only generated fictional data was used. No personal vault was opened, migrated or wiped. Repository visibility was confirmed PRIVATE. Apple enrollment is explicitly deferred to M5. No public download or distribution claim.
- **Next:** complete outstanding M1 evidence when representative data/client setup is available; exercise remaining native sharing error paths; implement the M3 restore/password-change/deletion workflow without risking a live vault. M4 providers and M5 distribution follow the agreed order.

## M3 lifecycle implementation batch

- **Restore:** owner can inspect a standalone encrypted backup from Settings or the lock screen, review record counts and the preserved-vault location, then activate a verified replacement. The source backup is not modified. Restoration works without the old password and never overwrites the inaccessible old vault. Backups with non-empty journals or over 8 GiB are refused by this in-app path.
- **Password change:** authenticates the current password read-only, creates a new independently encrypted/verified generation and requires review before switching and locking. Both operations remove client registrations and leave sharing off. Old recovery copies and backups remain under their old passwords; the UI and recovery guide explicitly disclose this.
- **Publication:** private generated directories plus a strict active-vault selector. Prepared bytes, active path, session generation and connection changes are rechecked. Atomic selector publication keeps old database/journal files in place. Invalid selectors fail closed. Unsupported downgrade and manual recovery are documented. Cancel/lock/stale operations discard prepared copies where possible; interrupted preparations may remain encrypted on disk.
- **Deletion/retention:** impact preview and typed confirmation for conversation deletion and older raw JSON snapshot cleanup. Deleted conversation evidence blocks retained curated records, including after reimport. Snapshot cleanup preserves current raw JSON and all text branches. Backups and retained generations are outside this deletion scope.
- **Automated evidence:** 61 cases passed in the full suite, followed by passing affected lifecycle/retention reruns and one extra missing-selector regression (62 cases total); frontend compilation, Clippy and native debug bundling pass. New coverage includes real encrypted restore/rotation/reopen, preservation/client reset, invalid inputs, stale operations, unwritable publication, tampered/unpublished preparations, deletion rollback, missing-source policy and snapshot retention. See VALIDATION for exact scope and limits.
- **Native evidence:** locked-screen restore completed in a separately named fictional-data app, including native file selection, inspected counts, consent, successful replacement, unlock, sharing off, new generation location and manual lock. Conversation and older-snapshot deletion previews displayed expected counts and were cancelled. No destructive native deletion or password-change submission is claimed. A stale unlock error is cleared after successful replacement.
- **Remaining:** broader native lifecycle coverage, full disk-full/power-loss fault matrix, full owner-entered password-change flow and retained-generation cleanup UX. Intermittent blank/slow debug startup remains an unresolved release issue. M3 is implemented but its complete release evidence is still in progress. M1/M2 representative-data, installed-client, physical-drop and broader lifecycle checks remain open. Apple enrollment stays at the end.

- **Repository metadata:** removed the unused project Website URL while keeping the repository private. The Pages endpoint returned 404 and no configured Pages site was found; the personal profile website was unchanged. This does not publish or deploy anything.

## Startup recovery batch

- **Prior CI:** verified `f48a7f6` passed all remote checks ([run](https://github.com/byLemonhaze/pre-drink/actions/runs/44661395645)). Repository remains private.
- **Diagnosis:** three instrumented baseline launches reached usable UI in 0.568–1.358 seconds. A fresh-profile CUA attachment took 217 seconds while the app had already reported ready at 0.599 seconds. The original intermittent stall was not reproduced, so no root-cause cure is claimed.
- **Changes:** independent local loading/retry screen; caught application-module failures; render-error boundary that locks before reload; native app-attached timeout prompt when no bootstrap signal arrives; navigation/reload locks the backend; opt-in content-free debug phase timings; startup assets explicitly included in rebuild tracking. The four new bootstrap tests run in CI.
- **Evidence:** 62 Rust cases and four bootstrap cases pass; frontend build and formatting pass. Normal native startup, intentionally missing renderer chunk with retry, and a script-free native timeout/retry/close path are exercised in separate fictional/empty storage. See VALIDATION for timing, exact checks and limits. The app-attached prompt replaced an initial unparented system dialog during verification. The final normal build reached its lock screen in 590 ms and reloading an unlocked fictional library returned to the locked screen.
- **Privacy:** no personal vault, export or password used. Fault builds and diagnostic samples stay outside Git. No dependency updates, public publication or Apple enrollment.

## ChatGPT import acceptance batch

- **Scope:** close more M1 import evidence and establish the provider/contract fixture structure for later M4 work. ChatGPT remains the only supported provider.
- **Delivered:** three hand-authored fictional JSON inputs with independently specified normalization, a provenance/limitations manifest, five import regressions and a reproducible local ZIP generator. No actual exports or generated ZIPs are tracked; the private-data guard is unchanged.
- **Checks:** 20 targeted importer/vault cases, formatting, Clippy and the tracked-file guard pass. No production code or dependency version changed; the full CI result remains commit-specific.
- **Native evidence:** picker import and omission notes, repeat import without duplicates, inactive-branch search and keyboard toggle, malformed later-entry rejection without partial writes, private search exclusion, cancellation and lock all passed against fictional storage.
- **Open:** real Finder-to-app drop was not completed after an optional screen-permission preflight was blocked; no access expansion occurred. Representative real exports, broader keyboard coverage, real-client MCP setup and remaining lifecycle checks stay open. Apple enrollment remains last.

## Installed-client acceptance preparation

- **Prepared:** a fictional-data Claude Code acceptance matrix covering actual tool results, approval/edit/source restrictions, memory-only scope, sharing defaults, lock, revocation and restart. Installed CLI help was checked; no client connection or matrix pass is claimed.
- **Native follow-up:** the owner enabled Screen Recording, but cross-window drag remains unverified after desktop automation stalled. Earlier picker evidence remains valid; the permission change is not an import-test result.
- **Environment:** a pending Xcode license initially blocked local Apple Git/build tools. After owner setup on 2026-11-20, Git and the first-launch status check pass, and the macOS SDK resolves. iOS simulator downloads are not required for this Mac app. Paid Developer Program enrollment remains deferred. This documentation-only follow-up does not change application behavior.

## MCP fixture and lifecycle regression batch

- **Delivered:** developer-only disposable encrypted fixture for installed-client acceptance; independent lock/revocation and approval-invalidating edit checks in the existing stdio regression. No production code/dependency changes.
- **Passed:** local protocol-client matrix through the real MCP binary, fixture permissions/cleanup, targeted stdio regression, all-target Clippy and formatting. See VALIDATION for the exact scope.
- **Pending:** Claude Code model calls require the owner's external-usage budget choice after automatic review blocked an unapproved spending limit. No model call or account configuration change was made. Native client-registration UI, native restart and physical file-drop remain separate gates.
- **Product direction:** the owner confirmed a later UI overhaul; functionality and reliability remain the immediate priority. Existing styling is not a final design commitment.
