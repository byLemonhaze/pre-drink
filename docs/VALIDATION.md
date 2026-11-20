# Validation record — 0.2.0

Verified locally on 2026-11-19 using synthetic data only. No user export, vault, private screenshot, account credential or raw session log is included here.

| Check                                      | Result                                                                                                                                    |
| ------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------- |
| Frontend TypeScript + production bundle    | Passed                                                                                                                                    |
| Prettier and Rust formatting               | Passed                                                                                                                                    |
| Cargo tests (locked dependencies)          | 50 passed: 15 importer/vault, 19 security, 9 unit, 5 sharing, 1 recovery, 1 actual stdio MCP; plus one explicitly invoked scale benchmark |
| Clippy, all targets, warnings denied       | Passed                                                                                                                                    |
| Optimized Apple Silicon app bundle         | Previously built at a31608c, approximately 19.8 MiB; this review-fix pass builds a debug app                                              |
| Signing inspection                         | Local linker/ad-hoc signature only; no Developer ID identity, sealed public-distribution signature or notarization ticket                 |
| Native synthetic workflow                  | Vault creation, picker import, source-linked memory approval, brief approval, context preview and manual lock verified                    |
| Landing page                               | Desktop and 375/320-pixel layouts reviewed; no horizontal overflow after fix; FAQ and download-status navigation verified                 |
| Repository guard and staged-content review | Source/documents/synthetic fixtures only; ignored data and build outputs excluded                                                         |

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

The destructive recovery case removes only a disposable generated vault, restores its independently encrypted backup and compares all logical tables (including provenance, snapshots, clients and FTS), correct-password behavior, source restrictions, sharing-off, revocation and permissions. It validates the documented manual recovery path; in-app restoration and password rotation remain unimplemented.

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
