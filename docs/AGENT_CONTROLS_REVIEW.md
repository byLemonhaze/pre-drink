# Agent controls review — Nov 22, 2026

Reviewed baseline: `515454c4c706`. Two reproducible defects were fixed: revocation after an idle extended work session, and out-of-order activity refreshes. No additional blocking defect was established in the inspected paths. This review was performed by the implementation agent, not a separate reviewer or a professional security auditor. The independent follow-up gate remains open.

## AC-1 — Revoke-all could stop before invalidating credentials

**Severity: Medium (control reliability). Fixed; regression failed before the change.** After an extended agent work session passed the normal 15-minute owner-idle threshold, `Session::revoke_all` validated access, ended the work session and called `access()` again. Normal idle expiry then locked the vault before the credential deletion ran. Sharing was stopped immediately and the command returned an error, so this was not continued access while locked; credentials nevertheless remained registered for a later sharing session.

The method now records the verified owner action before ending the work session. Already-expired sessions still fail the first access check and cannot be revived. The operation continues to stop sharing before attempting persistent deletion, checks the reviewed client list, and retains its storage-error fallback. The regression covers both an idle-but-authorized extended work session and an already-expired ordinary session.

Implementation: `src-tauri/src/agent_activity.rs:201`, `Session::revoke_all`; regression: `revoke_all_finishes_after_a_long_idle_work_session_without_reviving_expired_sessions`.

## AC-2 — Delayed reads could overwrite newer control status

**Severity: Medium (misleading access status); no backend authorization bypass established. Fixed; both regressions failed before the change.** A Refresh activity request could resolve after a later pause or history clear and replace the current client cards/history with its older snapshot. The owner could see an allowed connection after pausing it, or see cleared events reappear. The database retained the correct state, but the owner could not rely on the visible result.

The owner view now invalidates outstanding reads before control mutations and unmount, and applies only the newest refresh/page response. Obsolete errors are discarded as well. Two deferred-IPC regressions exercise the actual component handlers: late refresh after confirmed pause, and late refresh after confirmed history clear. The test harness controls hooks and promise completion; it does not simulate a browser, native accessibility or VoiceOver speech.

Implementation: `src/Agents.tsx:30`, refresh/pagination/control handlers; regressions in `tests/reading.test.mjs:628` and `tests/reading.test.mjs:652`.

## Inspected boundaries

- The broker requires the same OS account, bounded input and authentication, holds the session mutex through response delivery, and denies locked/global-off access before vault dispatch.
- All eight tools pass through persistent pause enforcement before dispatch. Resume retains the existing provider and curation scope. New owner control/history commands are absent from the MCP tool surface; unsupported names remain rejected.
- Shared-memory eligibility and provider-specific history/curation checks remain in backend dispatch; the UI does not grant permissions. Agent labels render as React text.
- Activity stores only bounded encrypted metadata. Unknown tool names are normalized; invalid credentials are not attributed. Started/processed/failed semantics and delivery limitations remain explicit.
- Clear history checks unlock generation; revoke-all checks the reviewed registration IDs. Transactions preserve records on injected failures; failure to pause/revoke stops global sharing.
- Schema 9 remains additive and rejects older readers; migration/rollback and encrypted restore tests cover the new tables. Restore removes registrations and cannot reactivate historical grants.
- No new remote requests, packages, schema changes, credential changes, or encryption changes were introduced by these fixes.

## Validation and limits

The three new regressions failed against the reviewed implementation, then passed after the fixes. Focused agent-activity and frontend checks passed. Full frozen-source verification passed **219 tests (172 Rust, 26 frontend, 21 Python)**, production build, formatting, source guard, three-provider fictional ZIP checks and all-target Clippy. One optional benchmark remains ignored. The matching recent advisory receipt passed with the three existing upstream warnings unresolved. The previous fictional native walkthrough remains evidence for the unchanged layout/basic controls; this review does not claim a new native reproduction of deliberately delayed IPC or a 15-minute physical wait. Existing dependency warnings, representative Claude/Grok exports, human VoiceOver announcements, separate review, and final Apple distribution retain their documented limits.

## Subsequent native acceptance

A later retry recovered native control and passed pause/resume, actual owner-confirmed history clearing and revoke-all from an active one-hour session, then quit/reopen persistence with no registrations and sharing off. The isolated app was left locked. Exact idle-time and deferred-response regressions remain automated evidence; no native timing injection or 15-minute physical wait is claimed. The current-source free release preflight passed at `6b13364` with 219 tests and fresh advisories. Full scope is recorded in NATIVE_WALKTHROUGH and VALIDATION. This follow-up does not change the separate-review requirement.

## Nov 24 — Focused self-review of logging/UI follow-up

Reviewed the `fd54333` → `720b57a` runtime delta and its regression tests at `1009828`: activity SQL/retention/counters, authentication and broker boundary, migration and backup stamps, transaction failure paths, single-client confirmation and stale-response guard, plus small onboarding/navigation changes. No additional blocking application-code finding was identified in this bounded review. This is the implementing assistant's follow-up, **not an independent review** and does not replace the supplied report's snapshot boundary.

Confirmed that unauthenticated input only increments the singleton numeric total after the same-account, parsed, unlocked/sharing-enabled boundary; caller identifiers/tool text are not persisted. Paused requests remain attributed only after valid authentication. Per-client retention and fair overall eviction preserve sparse clients under the tested flooding workload; archived registrations intentionally share one bounded bucket. Insert/trim/count changes are transactional, counters saturate at exact JavaScript integer bounds, and schema 9→10 preserves permissions/approvals. Single revoke checks acknowledgement before IPC, targets the reviewed ID and preserves the existing backend revocation/fail-closed behaviour. Native confirmation/cancel/focus evidence is now recorded in NATIVE_WALKTHROUGH.

A fresh advisory refresh did identify a newly published dependency vulnerability, RUSTSEC-2026-0594, in cookie 0.18.2. Updated only its lockfile version/checksum to patched 0.18.3. The affected private-cookie feature was absent from the inspected Apple Silicon feature graph; no PrivateJar or fork call was found in application sources. This is scoped reachability evidence, not a general claim about all dependencies. No advisory exception or gate bypass was added.

After the patch, `npm run verify:local` passed **226 checks (177 Rust, 28 frontend, 21 Python)**, fresh npm/RustSec audits, both Mac dependency-warning graphs, build/format/source guard, three-provider fictional curation fixtures and Clippy. The same three disclosed warnings remain. No paid Actions, personal-data processing, models or Apple account action was used.
