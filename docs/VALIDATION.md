# Validation record — 0.2.0

Verified locally on 2026-11-19 using synthetic data only. No user export, vault, private screenshot, account credential or raw session log is included here.

| Check                                      | Result                                                                                                                                                                                      |
| ------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Frontend TypeScript + production bundle    | Passed                                                                                                                                                                                      |
| Prettier and Rust formatting               | Passed                                                                                                                                                                                      |
| Cargo tests (locked dependencies)          | 62 passing cases across full-suite and targeted runs: 13 unit, 15 importer/vault, 19 security, 5 sharing, 5 lifecycle, 3 retention, 1 recovery, 1 stdio MCP; prior explicit scale benchmark |
| Clippy, all targets, warnings denied       | Passed                                                                                                                                                                                      |
| Optimized Apple Silicon app bundle         | Previously built at a31608c, approximately 19.8 MiB; this review-fix pass builds a debug app                                                                                                |
| Signing inspection                         | Local linker/ad-hoc signature only; no Developer ID identity, sealed public-distribution signature or notarization ticket                                                                   |
| Native synthetic workflow                  | Vault creation, picker import, source-linked memory approval, brief approval, context preview and manual lock verified                                                                      |
| Landing page                               | Desktop and 375/320-pixel layouts reviewed; no horizontal overflow after fix; FAQ and download-status navigation verified                                                                   |
| Repository guard and staged-content review | Source/documents/synthetic fixtures only; ignored data and build outputs excluded                                                                                                           |

## Dependency advisories

The ZIP dependency was upgraded from 4.6.1 to 9.7.3 after six published advisories were found. The importer does not extract ZIP entries to disk, and encrypted-entry support is disabled, but the outdated library was still replaced and regression checks rerun. SQLCipher was updated to 4.20.0 / SQLite 3.54.1 through rusqlite 0.43.0.

`npm audit` reported zero vulnerabilities. `cargo audit` completed without vulnerability findings but with three advisory warnings, **not a completely warning-free report**:

- `chrono 0.4.45`: unmaintained ([RUSTSEC-2026-0543](https://rustsec.org/advisories/RUSTSEC-2026-0543)); pulled by the official MCP SDK/schema dependencies in the macOS graph. No fixed upgrade was offered. Monitor upstream replacement; do not suppress the warning as resolved.
- `proc-macro-error 1.0.4`: unmaintained ([RUSTSEC-2024-0370](https://rustsec.org/advisories/RUSTSEC-2024-0370)); present in the cross-platform lockfile, absent from the active macOS dependency graph.
- `glib 0.18.5`: iterator unsoundness ([RUSTSEC-2024-0429](https://rustsec.org/advisories/RUSTSEC-2024-0429)); present in the cross-platform lockfile, absent from the active macOS dependency graph. Linux is not supported by this release; do not reuse this assessment for a Linux build.

Scans are time-specific and not proof of absence of vulnerabilities. Re-run at release. The application has not received a formal independent security audit. An owner-supplied AI review and its dispositions are recorded in REVIEW_RESPONSE.

## Remaining evidence needed

Representative real ChatGPT export compatibility; physical drag-and-drop; real-export and native large-library behavior/cancellation; complete sleep and lock-screen behavior; individual third-party client configuration; supported OS/hardware matrix; independent security review; Developer ID signing/notarization and verified distribution artifact.

The landing page intentionally does not offer the ad-hoc local bundle as a signed public download. Source builds are available under MIT.

## Review-fix verification

At `589967e`, 35 cases passed with the stronger approval, credential and metadata boundaries. The backup regression additionally verifies that a schema-v3 encrypted backup preserves current approvals when restored. Frontend compilation, formatting, Clippy and the repository guard pass. Refreshed npm/cargo audits report the same results above. The debug native bundle builds successfully.

At `589967e`, the new two-step review screen still needed an end-to-end native UI check. Automation reached the lock screen, but synthetic unlock attempts did not establish a verified session; the computer-use connection then became inactive and timed out, including after reconnecting. No successful native approval flow was claimed for that patch at that time. Later M1 evidence is recorded below. Earlier native workflow verification in the table applies to the previous interface. Only synthetic temporary fixtures were used; no personal export or password was requested.

## M1 first implementation batch

Current local suite: 43 passing cases, including timestamp/technical-word false positives, structured phones/credential assignments, explicit recheck staleness/rollback, retained secret branches, reimport restrictions, sharing stop and build/storage identity. Frontend production compilation, formatting and Clippy pass. Reclassification requires an owner action; it is not silently applied to personal vaults. The earlier 35-case evidence remains a historical baseline. No dependency versions changed in this batch, so the preceding advisory scan remains time-specific evidence rather than a newly claimed scan.

Remote GitHub verification for `589967e` completed successfully ([run](https://github.com/byLemonhaze/pre-drink/actions/runs/44633362843)). This does not establish the result of the newer M1 changes until their own workflow finishes.

### Native M1 check

A separately named debug bundle with an explicit synthetic storage folder verified: unlock, memory saved-version approval, brief approval with the current source note displayed, editing an approved note back to draft, blocking its linked brief, keeping a bare epoch timestamp unflagged, flag-recheck preview/application with zero changes, expanded build/storage details, and manual lock hiding the library. The ordinary app/personal vault was not used. Changed-flag application, stale previews and rollback are automated backend evidence, not yet native UI evidence.

Initial launch/tool access was slow and first showed an empty window; it subsequently loaded and the native checks completed. Do not count this as acceptable cold-start performance without measurement. Keyboard coverage, concurrent changes during native review, full sleep/lock behavior, representative-data upgrades and installed-client checks remain open.

## Selected sharing and recovery batch

Local full suite: **50 passed**, plus one explicit ignored-by-default benchmark. Frontend production build, Prettier/Rust formatting, Clippy all targets with warnings denied and a separate native debug bundle passed. Tests cover exact selected payload/file equality and Unicode byte counts; exclusion of private/sensitive/draft/missing items; omission of provenance; changed source reimport and source privacy; edit/delete/session/idle staleness before side effects; zero eligible results; duplicate/oversize selections; file collision/symlink/invalid destination and propagated delivery errors. Batch-index rollback/retry and duplicate-version handling also pass.

The destructive recovery case removes only a disposable generated vault, restores its independently encrypted backup and compares all logical tables (including provenance, snapshots, clients and FTS), correct-password behavior, source restrictions, sharing-off, revocation and permissions. At that batch it validated the documented manual recovery path; later lifecycle implementation and evidence are recorded below.

Native macOS debug verification with isolated fictional storage: approved-note/blocked-brief selection, complete preview, owner-only exclusion explanation, clipboard export and matching paste into an unsaved editor, native Markdown save, zero-eligible disabled actions, selection invalidation and manual lock. The saved native export independently matched all 207 preview bytes and mode 0600; the blocked brief and its exclusion text were absent. Remaining native cases include actual clipboard denial, disk-full, dialog cancellation, concurrent edits/lock while a dialog is open and fuller keyboard/resize coverage. Backend staleness/failure tests do not replace those UI checks.

### Reproducible synthetic scale measurement

Environment: local macOS 27.2.1, arm64, unoptimized Rust test profile, encrypted SQLCipher database on local temporary storage. One run before and one after the batch-index correction, using the same generated 5,000 conversations with 20 short text messages each. No personal data or path is recorded.

| Operation                     |    Before |   After |
| ----------------------------- | --------: | ------: |
| Initial normalized import     |  80.732 s | 3.719 s |
| Repeat normalized import      | 169.483 s | 4.004 s |
| First 100 conversation rows   |   0.115 s | 0.106 s |
| Search `garden`               |   0.187 s | 0.198 s |
| Search `garden planning`      |   0.208 s | 0.184 s |
| Chat-enabled context assembly |   0.189 s | 0.180 s |

Command: `cargo test --manifest-path src-tauri/Cargo.toml --locked --test benchmark -- --ignored --nocapture`.

This measures normalized import/index/search, not ZIP decompression/parsing, UI rendering, long/media-heavy chats, cold-start latency, cancellation or all hardware. It is a recorded observation, not a general performance guarantee. The ordinary suite skips the benchmark to avoid treating timing as a flaky CI assertion.

Remote CI at `b2ce46a` passed ([run](https://github.com/byLemonhaze/pre-drink/actions/runs/44645007179)). The current batch's remote result is pending until pushed and run; local results above are separate evidence.

## Lifecycle implementation batch

Local validation: **61 passed in the full suite**, followed by passing affected lifecycle/retention suites and one additional missing-selector regression (**62 cases total, 13 unit cases**), including 5 lifecycle and 3 archive-retention cases alongside all previous importer/security/sharing/recovery/MCP cases. Frontend production build, Clippy all targets with warnings denied and a separate native debug bundle pass. The earlier scale benchmark is unchanged historical evidence, not a newly run benchmark for this batch.

New automated evidence: restore preserves the chosen backup byte-for-byte and retains the previous encrypted vault; registration removal; correct old/new password behavior; repeated rotations; locked/empty-store recovery; private directory/file modes; edit/lock/idle/cancel/wrong-token/wrong-password rejection; malformed/newer-schema/active-journal refusal; corrupt/missing selector fails closed after layout publication; tampered preparation rejection; unpublished-stage restart model; unwritable publication folder preserves the original. Deletion tests cover exact impact counts, missing-source blocking after reimport, removal from FTS, current-snapshot retention, stale confirmation and transactional rollback after a synthetic deletion failure.

Limitations: the interruption case models unpublished state and restart, not an actual power cut. Full disk-full injection at each filesystem boundary, full native password-change entry/submission, process interruption during every publication boundary, automatic retained-generation cleanup and additional backup filesystems remain open. Password changes retain older encrypted copies under old passwords and disclose that fact; they do not claim secure erasure.

Remote CI for `be16955` passed ([run](https://github.com/byLemonhaze/pre-drink/actions/runs/44652417525)). The lifecycle commit `f48a7f6` also passed all remote checks ([run](https://github.com/byLemonhaze/pre-drink/actions/runs/44661395645)). No personal vault or export was used for these checks.

### Native lifecycle check

A separately named macOS debug app with a fresh fictional vault verified the locked-screen restore path: native backup picker, password entry for an existing backup, integrity review showing 1 conversation/1 message/1 memory/0 briefs, explicit consent, successful replacement, unlock with the backup password, restored archive content, sharing off, new active-generation location in Settings, and manual lock. Conversation deletion preview showed 1 message/2 raw snapshots; snapshot cleanup preview showed 1 older snapshot. Both destructive previews were cancelled, leaving the library intact. Backend tests, not native deletion submission, establish deletion execution/rollback. Full native password-change entry/submission still needs owner-driven verification. A small subsequent UI fix clears a stale unlock error after successful replacement; it is covered by frontend compilation, not a repeated native flow.

Cold-start remains a release issue: the separate debug app initially showed an empty window and later loaded after restarting and waiting. No cause or reliable timing was established. Successful recovery after loading does not resolve startup reliability. No personal vault, personal export or personal password was used.

## Startup recovery and measured launch evidence

The lifecycle commit `f48a7f6` passed every GitHub verification step, including 62 Rust cases, frontend build, formatting, dependency audits, Clippy and the tracked-file guard. Those results apply to that commit; consult the startup commit’s own GitHub run for its remote result.

On this macOS arm64 machine, opt-in native timing measured the existing app with instrumentation before recovery changes: 1,358 ms to usable lock screen, 568 ms on repeat launch, and 599 ms to Create vault with a fresh app identifier/empty storage. The fresh-profile automation attachment itself took 217 seconds, after the native interface-ready signal. This establishes an observation-tool delay for that run; it does not establish the cause of every historical blank window. A subsequent healthy launch with the new bootstrap reached interface-ready in 732 ms. These are individual debug-build observations, not a performance guarantee or clean-machine release benchmark.

Four Node regressions exercise the standalone bootstrap: timeout/retry, missing script/rejected import, successful mount ignoring late failure, and unrelated resource errors. The full current Rust suite passed all 62 cases (the opt-in benchmark remains skipped); frontend compilation and Clippy passed. Native fault injection uses temporary copies of built assets/configuration outside Git, with isolated empty or fictional storage: deleting only the React renderer chunk produces the visible startup failure card; Try again reloads and returns to that recoverable error while the asset remains absent. A separate `about:blank` test starts without any app scripts and reaches the app-attached native recovery prompt; retry reloads the empty view, a second timeout presents recovery again, and Close app exits successfully. After restoring the normal build, a fictional encrypted vault reached the usable lock screen in 590 ms; unlocking and choosing the web-view Reload action returned to the locked screen. These injections validate recovery paths, not a reproduction of the original intermittent WebKit issue.

No network fallback, remote telemetry, CSP relaxation or password bypass was added. Remaining evidence: broader hardware/OS startup and sleep/resume checks, actual WebKit process crash recovery, native React render-exception/lock-failure injection, and diagnosis if the original intermittent stall recurs with phase timing. The watchdog depends on a responsive native event loop; it cannot recover a frozen OS or main thread.

## ChatGPT import contracts and native acceptance

The targeted importer suite passes all 20 cases (15 existing plus five new contracts); formatting, Clippy and the tracked-file guard pass. Five new `import_contract` regressions cover three versioned fictional normalization examples plus archive-layout/reimport/search behavior, inactive-branch privacy, later malformed/duplicate records with no partial database write, and oversized declared JSON rejection before decompression. Fixtures live under `src-tauri/tests/fixtures/chatgpt/mapping-v1`; the manifest identifies this as an internal contract, not an official export version. Normalized timestamps are explicit floating-point numbers; input/raw JSON values retain their original number representation. No production parser changes or new provider support were required.

Native verification used the existing startup-recovery build with separate fictional storage: picker import of two conversations/six messages; seven omitted messages explained in Import notes; inactive-branch search; Tab navigation and Space activation of the retained-branches checkbox; displayed alternate reply/follow-up; repeat import reporting zero new/two refreshed with stable totals; malformed later JSON entry rejected with unchanged existing totals; manual privacy hiding the search result; picker cancellation preserving state; and manual lock. This is focused keyboard evidence, not a complete accessibility audit. The fixture packaging script reproduces both ZIPs locally and refuses overwrite.

Physical cross-window file drop remains unverified. The optional screenshot permission preflight was rejected by automatic approval review because it could request new macOS Screen Recording access. No permission was granted or bypassed; native picker checks continued through existing computer-use access. Representative personal-export and installed third-party MCP client verification remain separate open gates.

## MCP acceptance harness and independent lifecycle checks (2026-11-20)

The developer-only `mcp_acceptance_fixture` creates new fictional encrypted storage and uses the production broker/policy APIs. A local protocol client exercised the actual `pre-drink --mcp` binary against it: six-tool discovery; sharing-off denial; approved fact/source note and brief retrieval; draft/private/sensitive marker exclusion; memory-only chat denial; edit invalidation; source privacy exclusion; lock while credentials are still valid; unlock leaving sharing off; revocation; backend-session replacement with persisted revocation; owner-only directory/config permissions; and normal-exit temporary-file cleanup. All passed. This was a local protocol client, not Claude Code, and session replacement is not a native app restart test.

The existing automated stdio regression now checks lock before revocation, explicit sharing-off after unlock, context-pack filtering, edit invalidation and revocation after reopening. The targeted MCP regression, Clippy across all targets (including the example), and formatting pass. Socket checks require the environment to permit local Unix sockets. No production application behavior or dependencies changed.

Claude Code is installed and authenticated, but no model invocation was made: automatic approval review required an explicit external-usage spending limit. The installed-client gate remains pending. The owner subsequently enabled Screen Recording, but desktop automation stalled before physical file-drop evidence could be obtained; the earlier permission-blocked entry above records the prior attempt only.

## Installed Codex client acceptance (2026-11-20)

The owner selected Codex and Grok rather than authorizing Claude usage. Codex CLI 0.169.1 completed eight phases with ten actual MCP calls against the disposable encrypted backend fixture and production stdio bridge. Tool event assertions verified sharing-off denial; approved standalone/source facts and approved brief; absence of draft/private/sensitive markers; memory-only chat-search denial; edited-note exclusion; source restriction producing an empty facts list; lock denial before revocation; unlock retaining sharing-off; revocation with sharing on; and revocation surviving backend-session replacement/reopen. Every completed tool event targeted Pre-Drink. No model-generated shell/file/browser/other-agent operation executed.

The test used the existing ChatGPT login, no API key, ephemeral Codex sessions and per-invocation MCP settings. Credentials were passed through environment forwarding, not shell arguments or prompts. Fixture storage/configuration was removed on normal exit after revocation/lock. Raw client output was inspected locally and not committed. The source baseline `01b472a` passed the full GitHub workflow before this evidence update; this batch changes documentation only.

Grok Build 1.0.41 initialized its ACP interface, advertising the `grok.com` authentication method. Its explicit authentication attempt timed out; a subsequent direct session attempt returned authentication required. No Grok model invocation or successful tool call occurred. Unrelated connectors were verified disabled in an isolated temporary configuration, and fictional fixtures were cleaned. Owner sign-in is the next step, not a public tunnel or API purchase. These results do not cover Grok web/Bot, native desktop connection screens, native app restart, physical file drop, personal exports or additional provider importers.

## Installed Grok client acceptance (2026-11-20)

The owner completed browser OAuth sign-in, resolving the earlier block. Grok Build 1.0.41 then advertised and accepted cached account authentication. Eight policy phases passed with ten actual MCP calls across three fresh fictional fixtures: sharing-off denial; approved standalone/source facts and brief retrieval with draft/private/sensitive exclusions; memory-only chat denial; edited-note exclusion; private-source exclusion; lock; unlock retaining sharing-off; revoked-client denial; and continued denial after replacing/reopening the backend session. The ten calls comprise three calls in the approved-content phase and one in each other phase. Local assertions checked completed/failed ACP tool events and their returned content. Four built-in discovery calls also ran; no other tools executed.

The restart case required a test-driver correction: fixture restart already unlocks the backend, so a redundant unlock was rejected before the final model call. A fresh single-case retry with the correct sequence passed. This was a local driver issue, not an application code change. Native app restart and connection-screen verification remain open.

Only invented content was sent to Grok using the owner's existing account; no API key, public endpoint or persistent MCP registration was added. Each fixture revoked its credential, locked and removed temporary vault/configuration storage on normal exit. Raw local event logs are excluded from Git. Baseline documentation commit `fc495f0` passed GitHub checks before this update. This batch changes documentation only.

## Destructive reviewed-restore and damaged-backup checks (2026-11-20)

The targeted lifecycle/recovery suites pass all **nine cases** (seven lifecycle, two destructive recovery). All-target Clippy with warnings denied, formatting and the tracked-file guard pass. Three new cases extend the existing coverage. The destructive fixture now exercises both manual copying and the app's prepare/review/apply APIs after completely removing the synthetic live directory and journals. It recreates the backend session from the persisted selector and compares all logical tables, except the intentional removal of old client registrations in app-managed restore. It also verifies backup bytes, source restrictions, correct password, sharing-off, old-token denial and file permissions. This is backend evidence, not a native-screen or operating-system restart check.

Damage checks flip bytes in the first and a later encrypted page and truncate a partial final page. Each input is rejected without mutation of the original or input, no replacement selection is published, failed preparations are removed, and the original reopens. A subsequent valid restore succeeds. Another case removes the original backup after successful inspection, restores from the reviewed private copy, reopens successfully with no clients/sharing, and refuses replay of the applied review.

The production frontend and separate debug bundle build successfully from `0b169f8`; an isolated fictional launch reported interface-ready at 1,262 ms. Native computer control then failed to attach with an expired-continuation error after a long tool stall. No connection-screen, password-change or additional native-restore result is claimed. The ordinary installed vault stayed locked and was not used. The isolated process was stopped. Native controls and owner-entered password-change submission remain open.

GitHub verification of `0b169f8` passed before this batch. This batch changes tests/documentation only; it does not add a provider, alter encryption or claim release readiness.

## Client-specific configuration generation (2026-11-20)

Agent access now selects Codex, Grok Build or generic JSON before registration. The backend serializes the correct local stdio settings, with credentials only in environment entries. Configuration failure rolls back registration. Invalid/control-character paths are refused, and Grok paths containing environment-substitution syntax are rejected rather than silently reinterpreted. Scope and consent remain explicit; no settings file is edited and sharing is not enabled automatically.

All **36 affected Rust checks** pass: 16 unit cases (three new configuration cases), 19 security cases and the actual stdio MCP integration. New checks cover round-trip spaces/quotes/backslashes/Unicode, exact keys and argument boundaries, both scopes and revocation for every format, invalid-path rollback and unknown-format rejection. The first socket attempt was blocked by the execution environment; the permitted local-socket run passed. Frontend production build, native debug app bundle, formatting, tracked-file guard and all-target Clippy with warnings denied pass. TOML 1.2.1 was promoted from an existing locked transitive dependency; no dependency version was changed.

Read-only installed-client checks used production-generated fictional settings in temporary configuration homes. Codex CLI 0.169.1 returned the exact stdio command, arguments and environment. Grok Build 1.0.41 discovered the generated entry with the exact executable path; its inspection output does not expose all environment values, which remain covered by parser tests. No model calls, persistent account configuration edits, personal vaults or real export data were involved. Temporary settings were removed. New native controls are compiled but not yet verified through the Mac interface.

The preceding recovery commit `afeb3bf` passed its full GitHub workflow. Consult the configuration change's own run for its remote result.

## Round 3 privacy, retained-copy and clipboard fixes (2026-11-20)

All **82 distinct Rust checks** pass: an 80-case full desktop-feature suite, followed by the eight-case privacy-recheck suite containing two additional regressions added during final review. The opt-in 5,000-conversation benchmark remains ignored in ordinary runs. Four startup regressions, production frontend build, native debug app bundle (51.85 MiB), formatting, all-target Clippy with warnings denied and the tracked-file guard pass. No dependency versions changed. GitHub visibility was verified PRIVATE before publication of source changes.

Privacy coverage includes the reported natural-language password/phone shapes and short PINs while preserving timestamp/technical-discussion cases. Individual selection, unknown/duplicate/unshown IDs, complete-text limits, message/parent dependencies, stale review, transactional rollback and approval reset are covered. A schema-v3 encrypted fixture with an existing chat-enabled registration reopens with stronger restrictions; both agent search and context-pack calls exclude its fictional password. No model/provider calls or personal vaults were needed.

Four disposable retained-copy cases verify successful original-database/journal and later-generation removal; preservation of active data, selector, layout marker, external backup and unrelated files; typed confirmation; forged paths; missing/changed backups; changed vault data; stale sessions/files; pending replacements; symlinks/hard links; unknown directory entries; and reopening the remaining vault and independent backup. These tests use fresh encrypted synthetic vaults, never the owner's storage. Native click-through and filesystem crash/fault injection are separate outstanding checks.

The macOS clipboard test writes a fictional Markdown pack to a unique private pasteboard and confirms exact text plus the confidential marker. It does not read or overwrite the user's general clipboard. Apple's current-host-only option is set in production code; actual cross-device transfer and individual clipboard-manager behavior were not exercised. No automatic clipboard clearing is implemented.

New UI controls are compiled, but native item-by-item flag review and retained-copy deletion submission have not yet been exercised. The earlier computer-control attachment failure remains a reason to avoid claiming backend tests as native acceptance. Existing owner-assisted password-change, connection/restart, physical-drop and representative-data gates remain open. Apple enrollment/signing and public release remain deferred.

## Native Round 3 walkthrough (2026-11-20)

The separately named `Pre-Drink Walkthrough` debug app used a fresh fictional vault under a generated temporary directory. Its lock screen and sidebar explicitly displayed separate test storage. The native control attachment succeeded this time after a slow initial connection; subsequent controls and the full app restart worked.

Observed native passes: individual flag candidates default unchecked; no-selection application clears nothing; conversation-only clearance is rejected without changes; reviewed timestamp message/conversation clearance succeeds while the genuinely sensitive fixture stays hidden; memory returns to Draft and its brief requires source review. Backup gating disables old-copy removal. The native Save panel creates a verified encrypted backup outside the vault directory, clears password fields, and unlocks the cleanup review. An incomplete typed phrase stays disabled. After explicit owner confirmation, actual old-copy deletion succeeds and the refreshed list is empty.

Native Codex and Grok Build configuration screens generate the isolated executable/socket settings with memory-only scope and sharing off. No real client configuration is modified or model called. A selected-sharing preview includes the approved fictional note and excludes its draft brief; native clipboard copy and paste into an unsaved editor show the same Markdown, and that editor is cancelled. This proves native copy delivery, not cross-device/third-party clipboard-manager behavior.

A full native quit while sharing is on, then relaunch and unlock, preserves the archive/privacy classifications and both fictional registrations while restoring sharing off. The currently active vault survives the old-copy cleanup and reopens successfully. The separate native build includes a small layout fix discovered during the walkthrough: cleanup acknowledgement and typed phrase now use separate rows. Visual reinspection in the rebuilt native app confirms the acknowledgement, phrase input and action buttons are separated correctly. A fixture-only inactive backup copy was staged for this visual recheck; the review was cancelled without deleting it. Both fictional clients were then revoked, sharing stayed off, the vault was locked and the separate app quit.

Native password change remains unverified: the reported handoff completion did not result in an observed generation change, and the existing fictional password still unlocked the active vault. No completed submission was witnessed; this is not enough to diagnose a password-change defect. Two attempted Finder-to-app drags produced no import; the cross-window control gesture was inconclusive. These two native gates remain open, alongside representative owner-local export/upgrade coverage. No personal password was requested or personal vault used.

The reproducible generator is `examples/native_walkthrough_fixture.rs`; initial replacement and legacy flags are fixture setup, not native password-change evidence. The generator ran successfully, all-target Clippy with warnings denied passes, frontend/native builds succeed, formatting and all four startup regressions pass. Full Rust security results remain those of the preceding batch; no production Rust behavior changes here. See [NATIVE_WALKTHROUGH](NATIVE_WALKTHROUGH.md) for the detailed matrix and safe setup.

## Owner-assisted native password-change retry (2026-11-20)

The owner completed the fictional password change and reopened the vault. Native verification then locked it, entered the previous phrase using keyboard input and observed rejection, entered the new phrase and observed successful unlock. Settings shows a different active generation and lists the former generation as a retained recovery copy. Archive still contains two conversations/two messages, one sensitive conversation stays hidden, the approved cedar-planter memory and draft brief remain, and agent sharing is off. No clients are listed; they were revoked before this rotation, so native removal of populated registrations is not established by this run. The successful native password-change gate is now closed; automated registration-reset evidence and native failure-path limits remain distinct. The separate app was left locked. Physical Finder drop and representative owner-local export/upgrade checks remain open. No personal vault was involved.

Before the handoff, direct accessibility value-setting did not populate the password field correctly; ordinary keyboard input opened the fixture with its existing phrase. Those entry failures are not evidence of damaged storage. The password-change verification above used keyboard input. No product code changed in this retry.

## Owner-assisted Finder drop and repeat import (2026-11-20)

Two physical Finder-to-app drops were performed by the owner using the generated fictional ChatGPT ZIP. After the first drop, native inspection shows 2 new/0 refreshed conversations and 6 processed text messages; vault totals rise from 2 conversations/2 messages to 4 conversations/8 messages. The report discloses 7 hidden/tool/system/non-text omissions. The imported reading-nook conversation retains its alternate mahogany reply and old-lamp history; native search for the alternate reply term finds it. The pre-existing sensitive conversation remains hidden and sharing is off.

After the second drop of the same ZIP, native inspection shows 0 new/2 refreshed and the same 4 conversations/8 messages: no duplicate chats/messages. The vault was left locked. This closes the valid physical-drop/repeat-import gate and supersedes earlier inconclusive automated gestures. Malformed/oversized physical-drop coverage, representative owner-local exports/upgrades and broader native lifecycle failure paths remain separate. Documentation only; no personal content or product code changed.
