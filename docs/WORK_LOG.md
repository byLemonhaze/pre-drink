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

Apple enrollment is an owner action alongside M1; its status has not been confirmed. Do not report it as started.

## Open evidence / handoff

- Native two-step review now has synthetic evidence for the basic successful flow and edit invalidation. Full keyboard, concurrent-change, source-privacy and sleep/lock coverage still need their own native checks.
- Approval reset on an isolated encrypted copy of a representative old vault; never manipulate the live schema version.
- False-positive corrections applied to already flagged data without silently widening sharing.
- Representative official exports and physical file drop.
- One installed MCP client beyond the automated stdio harness.
- Synthetic large-library benchmark and complete native lifecycle/lock checks.
- Full recovery and credential-change failure paths; current backup verification is not yet a wipe-and-restore acceptance test.
- Developer ID/notarization and clean-machine installation.
- Remote CI passed at `589967e` ([run](https://github.com/byLemonhaze/pre-drink/actions/runs/44633362843)); the new M1 batch still needs its own remote result. Local pass counts are not evidence of remote success.

## Entry format for subsequent changes

For each completed batch record: milestone/task; commit; behavior changed; checks and environment; private-data handling (only pass/fail, no contents/paths); unresolved limitations; next task. Keep implementation status separate from native/release verification. Link detailed changes to their owning documents instead of duplicating contradictory status claims.

## M1 first batch — implementation and automated evidence

- **Behavior:** timestamps and ordinary technical mentions no longer flag by themselves; formatted/labeled phones, assigned credentials and recognizable/high-entropy secrets remain restricted. Settings can preview/apply a content-bound recheck. Applying disables sharing even on error; changed classifications clear approvals, preserve manual privacy/provenance, and require review. Reimports retain old sensitivity until explicit recheck.
- **Identity:** lock screen and Settings show version, source revision/local-change marker, build timestamp/profile, storage mode and an expandable local path. Release profile is not a signing claim.
- **Evidence:** 43 local automated cases (9 unit, 19 security, 14 importer/vault, 1 real stdio MCP), production frontend build, format and Clippy checks passed. Added regressions for timestamps/technical discussion, structured phones/assignments, recheck rollback/staleness, retained secret branches, reimport restrictions, sharing stop and build/storage identity.
- **Data handling:** automated tests use synthetic data. No real vault was migrated, wiped or uploaded in this batch.
- **Native evidence:** synthetic memory approval, brief approval with source display, note edit resetting approval and blocking the brief, timestamp remaining unflagged, zero-change recheck application, build/test-storage identity and manual lock passed. Changed-classification and failure cases are covered by automated backend tests, not yet the native UI. The review build initially displayed a blank window after a slow start, then loaded; cold-start timing remains an investigation item.
- **Remaining:** representative isolated-copy upgrade, real export/physical-drop/large-library and installed-client checks; broader native failure/keyboard/lifecycle cases. M1 remains in progress. M2–M5 remain planned.
