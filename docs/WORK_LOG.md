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

## Codex installed-client acceptance

- **Passed:** eight Codex CLI phases / ten actual MCP calls, independently checked from tool events, covering approved context, exclusions, scope, edits, source privacy, lock/unlock and persisted revocation. Existing ChatGPT sign-in was used; no API key or persistent MCP registration was added.
- **Grok:** installed Grok Build exposes local stdio support, but authenticated ACP session setup did not complete. No Grok model call ran. Owner sign-in is pending; no public tunnel was created. Claude remains deferred at the owner's request.
- **Privacy:** only invented fixture content was supplied to Codex. Credentials and raw events were kept outside Git; disposable vault/config files were cleaned after revocation. Native connection-screen and physical-drop verification remain open.
- **Verification:** baseline `01b472a` has passing GitHub checks. This evidence/documentation batch makes no production code or dependency change.

## Grok installed-client acceptance

- **Resolved:** owner browser OAuth sign-in completed; Grok Build 1.0.41 accepted cached authentication.
- **Passed:** eight policy phases / ten actual MCP calls with independently asserted results, covering approved context, exclusions, scope, edits, private sources, lock/unlock and revocation after backend reopen. Four Grok tool-discovery calls also ran. The redundant-unlock error in the local test driver was corrected and the remaining case retried successfully.
- **Privacy:** only invented fixtures were shared; no API key, public tunnel, persistent client registration, personal export or vault was used. Temporary vault/configuration storage was cleaned after revocation/lock. Logs stay outside Git.
- **Verification and limits:** `fc495f0` passed GitHub checks. No product code/dependency changed in this documentation update. Native setup screens, native app restart, physical file drop and representative-data gates remain open; Claude is deferred.

## Reviewed restore after complete vault loss

- **Added:** a second destructive recovery path using the same reviewed replacement APIs as the app, with full data/provenance/policy comparison after a fresh backend session. Old client credentials are intentionally removed and denied.
- **Added:** encrypted first/later-page damage and partial-page truncation checks, preserved original/input bytes, failed-stage cleanup and successful retry. A disconnected backup source after inspection does not change the private reviewed copy; applied reviews cannot be replayed after reopen.
- **Passed:** all nine affected lifecycle/recovery cases, all-target Clippy, formatting and the repository guard; production frontend and isolated debug bundle build. `0b169f8` has passing remote checks.
- **Native limit:** computer-control attachment stalled and returned an expired-continuation error. The isolated app reported ready in 1,262 ms, but no new native connection/recovery workflow was verified. The ordinary vault was not unlocked or modified; the isolated process was stopped. No personal content or raw logs enter Git.
- **Next:** native client setup/restart, owner-entered password change, representative isolated export/upgrade evidence and physical file drop. Apple enrollment stays last.

## Client-specific setup settings

- **Implemented:** Agent access offers Codex/Grok Build TOML and generic JSON, with destination/merge instructions and a clear distinction between hiding settings and revoking access. The generated result retains its own format label even if the next client selection changes.
- **Safety:** registration and serialization are atomic; unsupported paths leave no credential behind. Tokens stay in environment entries, never arguments. No client files, sharing scope, app privacy rules or default sharing state are changed automatically.
- **Passed:** 36 affected Rust cases, frontend/native debug builds, formatting, the repository guard, all-target Clippy, and read-only installed-client configuration checks. Codex read exact command/arguments/environment; Grok discovered the expected stdio executable. No model invocation or personal content was needed.
- **Remaining:** native connection-screen walkthrough and full app restart remain open after the earlier control-tool failure. Signing/enrollment and public release remain deferred. `afeb3bf` passed GitHub verification.

## Round 3 security review fixes

- **Privacy:** replaced count-only clearance with complete individual unchecked items and explicit selected IDs. New detection covers natural-language credential assignments, short PIN/passcodes and contextual bare phones. Opening a vault applies stronger restrictions before agent reads without clearing old flags; classification changes revoke approvals. A conversation cannot clear over a still-flagged message.
- **Recovery copies:** added reviewed size/date/file listings and typed deletion of inactive encrypted generations. The backend requires an unchanged app-verified backup outside storage in this unlocked session, verifies the active vault and refuses stale reviews, active/pending generations, links and unknown contents. No recursive deletion or secure-erasure claim. Actual user copies were not removed.
- **Clipboard:** current-host-only copying and a confidential marker written before plaintext, with failure handling. A unique native pasteboard test leaves the user's clipboard untouched; manager compliance remains outside app control.
- **Verification:** 82 distinct Rust checks, four startup regressions, frontend/native debug builds, formatting, all-target Clippy and source-only guard passed. Repository remains PRIVATE; author identity remains byLemonhaze. Documentation maps each R3 finding to evidence and explicit limits. No dependency update or personal content is part of this batch.
- **Next:** disposable native walkthrough of new review/cleanup controls, connection/restart, owner-entered password change and physical drag-and-drop; representative export/upgrade evidence stays local. UI overhaul can follow core acceptance. Apple enrollment remains last.

## Native walkthrough of Round 3 controls

- **Passed in the real Mac app:** selective privacy review and parent/message safety checks; draft/source-review reset; verified backup via native Save panel; backup/typed-confirmation gates; owner-confirmed deletion of the fictional old copy; configuration screens for Codex and Grok Build; exact preview, draft exclusion, native copy/paste; full quit/relaunch with surviving data/registrations and sharing reset.
- **Fixed:** cleanup confirmation text ran together visually. Added scoped spacing, a separate phrase field and wrapping action buttons. Frontend/native build and startup checks pass; visual reinspection in the rebuilt native app confirms the spacing fix.
- **Reproducible:** added a generator that creates only fresh fictional temporary storage, plus NATIVE_WALKTHROUGH. All-target Clippy and formatting pass. No personal data, generated vaults, screenshots or credentials are committed.
- **Remaining:** native password-change completion was not observed (existing phrase still unlocks the same generation). Two Finder drag attempts produced no import and did not establish a valid cross-window gesture. Both remain unverified, not diagnosed app failures.
- **Finish:** revoked both fictional clients, locked and quit the separate app. Fictional temporary fixtures/backups stay outside Git; the personal vault was not used. Apple signing/public release stay deferred.

## Owner-assisted password-change retry

- **Passed:** owner submitted the native change; independent keyboard-based unlock checks reject the old fictional phrase and accept the new one. The active generation changed, prior recovery generation is listed, archive/memory/brief and sensitivity remain, and sharing is off.
- **Limit:** clients were already revoked before rotation, so this run does not establish removal of populated registrations. Physical file drop and broader failure/representative-data checks remain open.
- **Finish:** left the separate fictional app locked. Documentation only; no personal data or product-code change. Supersedes the earlier incomplete-handoff status.

## Owner-assisted physical drop and repeat import

- **Passed:** first Finder drop adds 2 fictional conversations/6 messages and reports 7 omitted hidden/tool/system/non-text messages. Native branch display and search confirm retained alternate text.
- **Repeat:** second drop reports 0 new/2 refreshed; vault remains at 4 conversations/8 messages with no duplicates. Sensitive content stays hidden; sharing stays off.
- **Finish:** fictional vault locked. Updated acceptance/review documents; no product-code or personal-data changes. Valid physical-drop acceptance is closed; representative exports and broader failure cases remain separate.

## Failure handling and Mac screen-lock correction

- **Native passes:** malformed/oversized declared JSON rejection; cancelled plaintext export without a file; truncated backup rejection and intact-backup retry; quit before restore confirmation and reopen with unchanged active data/password.
- **Reproduced defect:** the owner's Mac screen lock did not close the baseline vault or disable sharing with a native Save panel open; the pre-lock export could still be saved. Only fictional data was involved.
- **Fix:** one-way distributed screen-lock observer, existing independent sleep/session/idle protections retained, missing notification center fails app setup. No notification can unlock or enable sharing. Core Foundation types promoted from an existing locked dependency.
- **Checks:** 83 Rust checks, four startup checks, formatting and all-target Clippy pass. Rebuilt-app screen-lock retest passes with the Save panel open: sharing off before saving, stale export denied and no output file. Separate owner-performed sleep/wake also locks the vault and disables sharing; unlocking preserves data and sensitivity. Notification delivery/OS coverage remains an explicit limit. The fictional app was locked and quit; no personal vault was used.

## Native large library and keyboard dialogs

- Added a reproducible generator for a fresh encrypted fictional 5,000-conversation/100,000-message native library; no personal exports or default storage are opened.
- Native pagination (100 → 200), unique search beyond loaded pages, empty results/recovery and source-linked draft memory saving pass. This is functional native evidence, not a rendering-speed guarantee or representative export check.
- Fixed a reproduced modal defect: Tab could reach archive controls behind the memory editor and Escape did not dismiss it. Memory, brief, approval and archive-deletion dialogs now share native WebView modality, explicit focus wrapping and cancellation; pending actions cannot be dismissed.
- Final native checks pass: forward/reverse Tab wrapping, Escape cancellation, 980 × 640 memory/brief dialog scrolling and accessible action buttons, and empty-brief validation without a new saved item. Both fictional sessions were locked and quit.
- Frontend/native builds, four startup checks, formatting, all-target Clippy and the source-only guard pass. No production Rust or dependency changes; remote verification is tracked separately. Representative official exports remain outstanding; personal content and authentication details never belong in this log.

## 2026-11-20 — Long-chat and curation browsing

- Added 20-message reader pages, 25-card memory/brief pages, full-loaded-collection conversation/brief search, explicit long-message expansion and searchable approved source selection. Stored originals, provenance and sharing policy are unchanged.
- Native fictional 2,000-message/3,000-memory/500-brief checks passed; fixed two issues found in that walkthrough: search autocorrection on blur and keyboard focus loss at a disabled final-page button.
- Added five component-render regressions to local checks and CI. Frontend/native builds, reading/startup tests, formatting and all-target Clippy pass. Remote verification is recorded separately. No new local full-Rust-suite pass is claimed for this frontend-only behavior change.
- Documented remaining full-collection IPC and maximum-size limits. Official export is being prepared; representative-data verification stays pending. No private content or generated artifacts are committed.
- Next independent batch: failure/interruption checks and a quieter, more compact dark system/retro UI pass; keep exact sharing preview central. Apple signing stays last and publication requires the owner's launch decision.

## 2026-11-20 — Storage faults, cream/orange appearance and compact sharing

- Added encrypted SQLite capacity and killed-writer recovery regressions; 85 Rust checks and all-target Clippy pass. The faults preserve committed content, approvals and search, and permit retry. Filesystem publication faults and physical power loss remain outside this evidence.
- Added cream/orange light mode by default and optional dark mode, a non-sensitive local appearance preference, matching startup palette and one canonical glass icon. The lock screen now has a spacious two-column composition; diagnostics and restore use contained dialogs. The duplicate decorative glass was removed after owner feedback. Final visual acceptance remains with the owner.
- Reduced sidebar/header clutter, grouped reader privacy/deletion controls, kept lock/sharing status visible while scrolling and paged sharing choices without losing selections. The exact preview still excludes restricted content and requires review before copy/save.
- Native fictional minimum/normal window checks, wrong-password/retry, first-run fit, restore cancellation/focus, theme persistence, archive/memory and sharing checks pass within the scope in VALIDATION. Fixed the first-run footer clipping found during this pass. Five startup and five reading regressions, frontend/native builds, formatting and the tracked-file guard pass. Remote checks are recorded separately.
- Recorded the owner's source-provider curation idea in PRODUCT: strict same-provider working spaces (OpenAI curates ChatGPT; Grok curates Grok), proposals and reviewed shared memory readable by other authorized agents, with a separately authorized local-AI rule possible later. Documentation only; no agent write access, automatic promotion, paid inference or additional import provider is implemented.
- Personal storage was not opened. The official export remains pending; Apple signing and public launch stay deferred. Next: representative-export/upgrade verification when delivered, further bounded memory/maximum-message and publication-fault work, then release documentation and fresh review.

## Owner clarification — autonomous memory is a core requirement

- The intended product is an evolving shared memory across AI tools. Agents must handle bulk archive curation, incremental imports and explicit end-of-chat memory updates; the owner cannot reasonably inspect the entire archive or individually approve thousands of extracted notes.
- Same-provider boundaries are strict: OpenAI/ChatGPT/Codex curates ChatGPT history; Grok curates Grok history. Other authorized agents read eligible shared contributions without editing another provider's private source memory. Local-AI access is a later explicitly defined rule.
- Added AGENT_CURATION with provenance, resumability, idempotency, revisions/undo, collection-level publication policy, privacy boundaries and acceptance criteria. This supersedes the earlier optional-workspace/per-note-proposal framing. Implementation follows completion of the current checks; existing MCP tools and grants are unchanged.

## Provider-isolated agent curation — 2026-11-20

- **Direction:** owner approved the separate-provider image flow. This is an additive change to the existing app, not a rewrite. ChatGPT/Codex and Grok have separate raw-source/working-memory scopes; only eligible curated summaries cross through shared memory. Grok ZIP import remains planned.
- **Storage/access:** schema 4 adds encrypted provider grants, working contributions, source versions, complete revisions, publication attribution, retry receipts, stable job manifests and progress acknowledgements. Manual notes and approvals survive the upgrade. Old broad chat grants become shared-only instead of being guessed from client names. Source/privacy/branch changes invalidate evidence; identical imports do not.
- **Agent path:** `curate_memory` enables scoped source reads, revision-aware/idempotent session and archive writes, history/held undo, and resumable incremental jobs. Every existing raw path now enforces provider scope. Shared context packs contain no raw excerpts. Agents cannot change owner policy or overwrite another provider's contributions.
- **Owner path:** Agent access has separate workspaces, complete batch review, editing/conflict resolution, history, withdrawal, saved progress and explicit automatic-publication controls. Both automation switches default off. Only future eligible projects/decisions may use those rules; facts, URLs/instruction-like text need review, and sensitive/stale/conflicting content is withheld. Released items identify provider, origin, revision and owner-review versus automatic-policy basis.
- **Native findings:** an initial inline preview could appear below the fold; moved batch review and editing to the existing modal component. The fictional walkthrough verified two-note batch release, edit-to-private revision, history, earlier text saved as a held revision, separate Grok workspace, withdrawal, and a shared-context preview containing only the remaining reviewed fact. Automatic sharing remained off and no external agent was connected. Keyboard focus restoration received a targeted follow-up fix and passed in the rebuilt native preview: History → Escape returned focus to its opening button. The fictional app was left locked.
- **Boundaries:** no personal vault/export was used, no model API was called, and no provider credentials were added to the repository. Repository visibility was independently confirmed PRIVATE. Earlier Codex/Grok retrieval runs are not represented as model-driven curation acceptance. Actual-model quality, representative real-export acceptance and Apple distribution remain open.

## Documentation reconciliation — provider curation

- Replaced the manual-only product definition with provider-private archives/working memory, controlled shared memory and owner-authored briefs. Integrated the behavior into README, architecture/schema, execution order, privacy, MCP, recovery, sharing, design, review and release guidance instead of leaving contradictory appendices.
- Cloud curation is explicitly optional. Offline import/search/manual curation/briefs/export/backup requires no model or key. Local-model execution is planned, not advertised as present. Separate Grok session-writing support from its unimplemented ZIP parser.
- Kept chronological validation history intact while updating current status and next gates. Earlier model-client retrieval is not new curation-quality acceptance. Static landing copy is updated locally only; no hosting/publication change is authorized.

## Provider scope correction and expansion — 2026-11-20

- Owner clarified Grok import should already be included, then explicitly added Claude and requested Gemini/Copilot/others where official formats can be validated. Implemented independent Grok account-export and Claude conversation adapters on the existing bounded ZIP/transaction path. Provider labels and message authors now use actual stored origin.
- Added provider/version fixture manifests, three-provider import/access regressions and schema 5 for Claude grants. Existing v4 clients, policies, history and child records are preserved; no grant broadens. Claude configuration uses local MCP JSON. Actual Claude model execution is not claimed.
- Owner confirmed cloud curation is optional, and approved a future advanced cross-provider curator option with explicit source/destination consent. This changes future product policy, not current grants. Preserved default isolation and documented the required delegated workspace/provenance design. Gemini/Copilot official download guidance is recorded, with actual parser contracts still outstanding.
- The earlier curation/documentation commit `2ed7241` passed [GitHub verification](https://github.com/byLemonhaze/pre-drink/actions/runs/44830543395). This is evidence for that commit only; the provider expansion has its own checks recorded in VALIDATION.

- **Expansion verification:** all 106 Rust checks, five startup checks, six reading checks, Clippy and builds passed locally. Native Grok/Claude picker imports, two-ZIP repeat import, provider labels/branches, Claude search and isolated workspace/client wording passed on fictional storage. The v4 preview upgraded with prior curation statuses/grants intact and was left locked. See VALIDATION and NATIVE_WALKTHROUGH for evidence boundaries.

## Claude saved-job upgrade correction — 2026-11-20

Final constraint review found that the schema-5 migration widened grants/policies/notes but left the old saved-job constraint unchanged. Fresh vaults worked; upgraded v4 vaults rejected new Claude jobs. Added checks for v4 and already-v5 fixtures, observed both fail specifically at Claude job creation, then implemented schema 6 to rebuild jobs transactionally and preserve manifest children. Existing provider grants and publication rules do not broaden. This supersedes schema-5 completion claims for resumable Claude curation; the earlier import/native evidence remains valid for its stated scope.

## Installed Codex curation and handoff — 2026-11-20

- Added a developer-only encrypted fixture that accepts no personal storage and a bounded opt-in Codex driver using existing ChatGPT authentication. It uses temporary scoped credentials, an empty working folder and only Pre-Drink tools. No model/API dependency was added to the app.
- Three actual fresh sessions / 27 MCP calls passed: forbidden Grok/Claude sources, private session-save retry, archive draft, backend restart and stable job resume, revision retry, operator-reviewed release and shared-reader retrieval/curation denial. The fictional summary preserved cedar versus rejected oak, unresolved plants and an independently labelled session handoff; hostile imported instructions were ignored in this run.
- Actual event assertions and owner state, not the model's success claim, establish the result. Additional no-model stdio probes verified edit/source-privacy/lock exclusion. Fixture registrations were revoked and temporary configs/logs/vault removed. No personal archive was opened or sent to a provider; only invented text used the existing ChatGPT allowance.
- Updated current product, execution, roadmap, MCP, release and review guidance, with CURATION_ACCEPTANCE as the reproducible procedure. Broader semantic quality, other-client writing and real-export checks stay open. Apple enrollment/signing and public launch remain deferred.

## Grok writing, bounded message expansion and recovery hardening — 2026-11-20

- Generalized the disposable installed-client fixture to matching ChatGPT/Grok/Claude source contracts, extracted common prompts, added a bounded Grok ACP driver and made the receipt verifier provider-aware. Grok 1.0.41 passed three fresh fictional sessions, 29 vault calls and three discovery calls: foreign reads denied, session/archive retries idempotent, saved job resumed after backend replacement, reviewed shared retrieval and curation denial for a shared-only reader. Local edit/privacy/lock probes passed; normal shutdown revoked test registrations and removed private logs/configs/cache. Claude remains unauthenticated and unverified as a model client.
- Recorded the lexical retrieval limitation instead of treating it as semantic retrieval: Grok's archive summary omitted the topic name, so the topic pack returned only the matching session note while project listing returned both. New quality acceptance measures topic preservation, precision, coverage and owner corrections separately from tool success.
- Added the bounded plain-text reader for messages above 48,000 UTF-16 units, with approximately 16,000-unit lossless Unicode parts. Original source/search remain complete. Native minimum-window light/dark walkthrough passed with a 550,133-unit fictional message, including tail access, keyboard controls, collapse/search and lock. Temporary preview storage was removed.
- Reproduced a selector-publication interruption gap with an actual child-process kill. Moved the durable no-fallback guard before selector rename and validated existing guard files. Two kill-point regressions pass; first-switch ambiguity requires explicit recovery, and committed selection cannot silently revert to the old root after selector loss. Both generations remain recoverable. Recovery/security/architecture docs state the availability tradeoff and physical-power-loss limits.
- Read the owner's refreshed product review at its `2ed7241` snapshot. Added a disposition in REVIEW_RESPONSE, QUICKSTART and CURATION_QUALITY. Current Grok/Claude imports and Codex/Grok writing supersede that snapshot's gaps; real-export quality, guided onboarding, independent review and representative provider contracts remain open. No competitor exclusivity claim was adopted.
- Local checks: 109 Rust tests passed, one opt-in benchmark ignored; 12 frontend checks passed; build, formatting, all-target Clippy and source-only guard passed. npm audit reported zero vulnerabilities; Rust audit completed with three documented existing warnings. GitHub visibility was confirmed PRIVATE. Source-only publication and exact-head CI follow this record; no user data or screenshots are included.

Next: pending official export with isolated import/upgrade acceptance; explicitly scoped/budgeted cloud quality run only if the owner chooses it; Claude acceptance when an account is available; guided setup and further IPC/accessibility profiling; independent review and fictional demo. Apple enrollment/signing/notarization remain the final distribution stage. Optional cross-provider delegation and Gemini/Copilot adapters remain defined follow-ups, not silently enabled features.

## Guided setup, fictional quality preparation and dropdown fix — 2026-11-20

- Added guided agent setup with shared-only defaults, explicit scope acknowledgement, private settings instructions and request verification. Added the eighth MCP tool, `connection_status`, which returns permissions only. Successful delivery evidence is owner-local and clears on lock; settings creation is never reported as a connection.
- Moved conversation loading to 20-message backend pages, retaining full-source search, branches, privacy checks, stale-response rejection and explicit retry. Fictional payload profiling reduced the first-page response from about 24.7 MB to 49 KB; individual large-message and memory/brief collection bounds remain open.
- Prepared a versioned ten-case curation quality corpus and separate scoring rubric for corrections, rejected suggestions, duplicates, uncertainty, hostile quotes, sensitivity, French, project-name retrieval and provider walls. Generated archives pass all three production parser paths. No model quality score, cloud spend or real-export acceptance is implied.
- Addressed the owner's screenshot report of a native dropdown remaining over another app when minimized. Replaced all selects with an in-window accessible chooser; added keyboard, focus/visibility dismissal, selected-value labels and a regression against native popup reintroduction. Native keyboard/pointer, minimize dismissal and light/dark checks passed within the stated scope; no desktop-wide visual comparison is claimed.
- Updated README, quickstart, MCP contract, architecture, execution plan, review response, reviewer brief, quality protocol and validation/native evidence. Reconfirmed repository visibility PRIVATE and author identity byLemonhaze. No user data, configurations, screenshots or tokens are part of this change.
- Verification: 112 Rust checks, 14 frontend checks, all-target Clippy, formatting and native preview build passed. A temporary fictional registration was revoked, the preview locked/quit, and temporary storage cleaned up. Source-only push and exact-head CI are checked separately.

Next: the requested official export and isolated real-data import/upgrade acceptance; bounded model-quality scoring only with the owner's usage/data scope; Claude installed-model acceptance when available; individual-message and memory/brief IPC bounds; fresh independent security review and fictional demo; Apple signing/notarization last. The guided flow still needs manual external-client settings and a native positive verification result; no automatic connection or model launch is claimed.

## Zero-extra-cost constraint — 2026-11-20

The owner reported nearing the account's included Actions allowance and explicitly has no money for additional charges. The last automatic run for `6b8784a` passed all stages in 6m26s. Removed push, pull-request and weekly triggers; the workflow is manual-only. Future verification runs locally unless the owner explicitly authorizes use of included cloud capacity. No account billing setting was changed, and no repository visibility change is a substitute for the owner's private-until-ready requirement. The workflow/documentation-only cost-policy commit is checked locally and pushed with a CI-skip marker; it deliberately has no new GitHub run.

## Evening review implementation — 2026-11-20

Implemented the four immediate review follow-ups on the existing product, without new provider/API usage or remote CI:

1. Independent encrypted owner holds in schema 7. Agent updates/undo cannot clear them; old conflicts migrate conservatively. Fresh publication rechecks note content and session labels.
2. Explicit owner-started one-hour/four-hour/until-Mac-lock work sessions. Monotonic timed expiry, status/stop controls, existing provider permissions and authoritative lock/sleep/revocation remain. Ordinary sharing retains normal idle locking; grants never resume on unlock/restart.
3. One-command offline verification and a repository-local pre-push guard, installed without replacing other hooks. Receipts bind implementation, documentation and tool versions; Markdown-only changes get narrow fresh checks. No automatic Actions triggers were restored.
4. Targeted privacy reclassification with linked-only approval resets and an unlock count notice. Narrow status-prose/PIN false-positive fixes and contextual phone/SIN coverage. Broader regex/privacy limits remain documented.

Native fictional checks caught and fixed the curation editor's cramped textarea, then verified both themes, session start/stop, manual locking and hold clearance staying private. The isolated preview was locked and quit. The personal vault was never opened or modified.

Evidence: 119 Rust + 15 frontend + 5 Python cases passed, one optional benchmark ignored; production build, formatting, Clippy and fictional provider packaging/import checks passed. The first receipt correctly rejected documentation changes made during verification; final frozen-source verification then passed and wrote the receipt. Review, product, architecture, security, recovery, MCP, quick-start and execution documentation were reconciled. Repo visibility was read back as PRIVATE; latest remote CI remains the previous successful `6b8784a` run.

Remaining: representative official export checks when the requested ZIP arrives; broader quality/Claude installed-model acceptance subject to explicit usage authorization; larger memory/brief IPC and accessibility/OS coverage; review of current source; fictional demo and release packaging. Retained nonblocking backlog includes OS credential integration/peer identity, minimal access metadata, KDF-format evolution and dependency warnings. Apple enrollment/signing/notarization stays at the final distribution stage. Public visibility/downloads still require the owner's launch decision.

## Local account verification and backend library pages — 2026-11-21

Added kernel peer-account checks to both Unix socket endpoints, before token transmission/request parsing; enforce parent ownership and refuse deletion of non-socket stale paths. Tokens/provider scopes remain unchanged. Client reply buffers are zeroized on early error paths too. Added direct `libc` dependency from the existing locked graph, without dependency upgrades.

Replaced full owner memory/brief collection responses with a 25-record library command used by Memory, Briefs, selected sharing and brief source selection. Full Unicode content search, privacy/eligibility filtering, counts and page clamping happen in Rust while retaining at most two pages. Displayed briefs alone load source IDs. UI cancellation guards stale results, resets filters, retries errors and retains sharing choices across searches. The source picker now pages all eligible choices, keeps source text complete and uses non-submit navigation buttons inside the editor.

Final local verification passed: 124 Rust + 16 frontend + 5 Python tests, one optional benchmark ignored; production build, formatting, source guard, Clippy and three-provider fictional packaging/import checks. The first full run caught a Clippy style issue after tests passed; corrected source passed the complete rerun and wrote its receipt. Native fictional walkthrough passed, the preview was locked/quit and this run’s temporary storage was removed. Updated architecture, security, MCP, quick start, review response, roadmap and execution/reviewer briefs. Remaining work includes indexed query optimization, individual-record byte bounds, broader native/accessibility/filesystem evidence, representative exports, current-source independent review, quality acceptance and final Apple distribution. Same-account/root malware, credential storage integration and executable identity are not solved by UID checks. No paid usage or private fixtures are authorized for publication.

## Representative ChatGPT export check — 2026-11-21

The owner supplied an official export locally. Read the original ZIP without extracting attachments or writing its contents to plaintext diagnostic files. Independent source accounting matched the production parser's normalized totals and omissions. A temporary verifier exercised fresh isolated SQLCipher storage, complete normalized-message preservation, verified encrypted backup/open, duplicate-free repeat import, search/paging/default privacy, reopen and wrong-password rejection. Both development and optimized runs passed all checks and removed their generated encrypted copies. No model/API or remote CI was used, and the existing personal vault was not opened or changed.

Development-mode sensitivity scans were noticeably slow. A private function-stack sample and fictional microbenchmark identified the matching engine; optimized execution of unchanged rules was materially faster. No speculative detector change was shipped. The temporary verifier was removed from the repository working tree. Only generic verification evidence is documented; source contents, identifiers, filenames, personal counts and raw diagnostics are excluded from Git.

Remaining: representative native workflow and existing-vault upgrade checks, real Claude/Grok export acceptance, broader performance/hardware and curation-quality evidence, fresh independent review and final Apple enrollment/signing/notarization. Real archive curation has not started and no source access has been granted to cloud agents.
