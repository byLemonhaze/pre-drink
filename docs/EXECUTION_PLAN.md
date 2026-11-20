# Execution plan

Initial direction: 2026-11-19; revised for the owner-approved provider curation workflow on 2026-11-20. Original implementation baseline: `589967e`. Current evidence is in VALIDATION and the chronological WORK_LOG.

## Outcome

Pre-Drink should be a useful daily macOS app and a credible portfolio project: import official AI exports into a private encrypted archive, curate facts/decisions/projects, assemble briefs, preview exactly what will be exported, and deliberately provide context to another product. Manual local features are free and require no model API. The main demonstration is the sharing preview and the control it gives the owner.

Official export import currently supports ChatGPT only. Provider-bound ChatGPT/Codex and Grok writing roles are implemented; Grok session summaries do not imply Grok ZIP support. Claude/Grok import remains later work. Source is independently implemented, informed by Loominary; it is not a Loominary fork.

## Non-negotiable boundaries

- Keep the repository private and public downloads/Pages disabled until the owner explicitly chooses to launch.
- Never commit real exports, vaults, personal chat text, screenshots of personal data, credentials, passwords, signing material or raw private test logs. A private repository is not an exception.
- Use official exports only. No live capture, scraping, browser credentials or userscripts.
- Keep the encrypted archive as the source of truth. Sharing, plaintext exports and cloud-backed agents cross separate boundaries.
- Optional external same-provider curation is in scope; mandatory cloud inference and an app-run background model service are not. Offline manual use must remain complete. No cloud sync, monetization, demand tests, semantic search, Gemini, mobile or team features.
- Approval cannot override private sources or detected credentials. Every changed contribution revokes its prior release. Owner/manual edits require review; agent revisions can be released only by separate review or an already enabled narrow owner policy, with the basis recorded.
- Test personal data only in isolated encrypted copies after a verified backup. Destructive recovery tests use disposable synthetic vaults exclusively.
- A passing test suite does not establish a formal security audit or a signed public release.

## Status vocabulary and evidence

**Planned** means agreed but not built. **In progress** means active work remains. **Implemented** means code exists; validation may remain. **Verified** requires recorded evidence for the stated environment. **Blocked** names an external prerequisite. **Deferred** is outside the current milestone. Do not use “done” to combine these states.

The implementation/evidence ledger is [WORK_LOG](WORK_LOG.md). [VALIDATION](VALIDATION.md) records checks and limits. [REVIEW_RESPONSE](REVIEW_RESPONSE.md) maps the supplied review findings. This file owns milestone order and exit criteria; the roadmap is its short index.

## Sequence and planning estimate

M1 → M2 → M3 → provider curation acceptance → M4 → M5. The curation implementation is additive and precedes new import providers. At the owner’s request, Apple enrollment is deferred to the final release stage. Development and all independent verification proceed now. Work can prepare fixtures or release documentation early, but must not present unfinished milestones as delivered.

The initial 3–5-day / 1–2-week estimate predates the expanded autonomous-curation requirement and is superseded. Remaining effort must be estimated after the fictional model-led curation run, the pending representative export and fresh review. No calendar-ready claim is justified while those inputs and distribution checks remain open. Work continues on independent checks without waiting for Apple enrollment.

| Milestone                              | Current state                                               | Exit evidence                                                                                                                              |
| -------------------------------------- | ----------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| M1: dependable ChatGPT workflow        | In progress                                                 | Current native approval flow, false-positive regressions, isolated upgrade verification, a real MCP client, and clear build/vault identity |
| M2: deliberate sharing preview         | Implemented; basic native flow verified                     | Preview/export equality, eligibility revalidation, no surprise source metadata, selective copy/file export                                 |
| M3: vault lifecycle and recovery       | Implemented; lifecycle validation in progress               | Tested restore UI, password change, deletion/retention, destructive synthetic recovery exercise                                            |
| Provider curation                      | Implemented; deterministic/native fictional checks recorded | Fresh model-led write/resume, shared retrieval, source isolation and quality acceptance                                                    |
| M4: Claude, then Grok                  | Planned                                                     | Provider/version fixtures, normalization and cross-provider privacy/import tests, local representative export checks                       |
| M5: release and portfolio presentation | Planned; Apple prerequisites external                       | Signed/notarized artifact, clean-machine installation, release evidence, fictional-data demo and reviewed documentation                    |

## M1 — Dependable ChatGPT workflow

Goal: a person can use the current app predictably before new providers expand the data surface.

1. Identify the app build and active vault in the UI. Clearly distinguish development/test storage from ordinary installed use. Show version/build information without publishing local paths or personal data. Do not invent a default password or ship shared test credentials.
2. Replace the broad “10+ digits” sensitive rule with contextual/structured detection. Unix seconds/milliseconds and harmless identifiers must not be flagged solely for their length. Formatted phone numbers, financial/medical indicators and actual credential patterns must remain protected. Keep credential blocking separate from general warning explanations in the design; do not add a blanket sensitivity override.
3. Handle existing flags safely. Reclassification must be deliberate and atomic, retain manual privacy, and prevent previously hidden material from becoming agent-visible without fresh review. Test retained branches and source-derived restrictions, not only new imports.
4. Verify schema-v2 → v3 approval reset on an isolated encrypted copy of a representative existing vault. Record only pass/fail evidence in the repo. Check content/provenance preservation, exactly-once reset, subsequent approval persistence and backup compatibility. Do not downgrade the owner's live vault to simulate an upgrade. If the available vault is already migrated, record that limitation and use a faithful synthetic old-version fixture.
5. Finish native verification of save → draft → review → approve for memory and briefs. Test edits during review, changed/private/deleted sources, lock/reopen, stale responses, and keyboard access. The new review interface was not verified end to end at the baseline; subsequent evidence belongs in WORK_LOG/VALIDATION and does not automatically close all of these cases.
6. Exercise native file picking and actual file drop, representative ChatGPT exports, repeat imports, branch retention and large-library behavior. Use a synthetic 5,000-conversation benchmark and record environment/timings; do not invent a performance claim before measurement.
7. Verify one actually installed MCP client against a synthetic vault: configuration, tool discovery, context retrieval, memory-only metadata boundary, sharing off, lock and revocation. Record client/version. Additional clients need their own checks before compatibility is advertised.

Exit: all items have evidence or an explicitly agreed reduction in the advertised scope; no unresolved data-loss, approval-bypass or credential-disclosure defect. Keep any missing native/real-data checks visibly open.

## M2 — Preview and selective sharing

Goal: “This is what leaves the vault” is a property of the implementation, not just marketing text.

- Center a sharing screen on a selected approved brief or an explicitly selected set of approved notes. Show the full outgoing Markdown, scope, item count/size, and owner-only reasons for exclusions. Do not leak restricted titles or contents in exported exclusion notices.
- Use one backend assembly/policy path for preview and export. Bind the prepared payload to the reviewed item/source versions. Revalidate at copy/export; if content or privacy changes, reject the stale operation and require a fresh preview. A successful operation emits the exact reviewed text.
- Support Copy Markdown and Save Markdown with a clear destination step. A file or clipboard copy is plaintext outside vault encryption; nothing is sent to a provider automatically. Avoid silent overwrite and do not log exported content.
- Distinguish a selected immutable export from a live MCP client grant. Today's client grants cover all eligible notes/briefs and dynamic queries; previewing one topic does not narrow permission to that topic. Show this distinction honestly. Per-brief/project grants are a separate access-control change, not an implication of this screen.
- Optional file presets such as AGENTS.md/CLAUDE.md may change formatting, never eligibility; show the final formatted content before export. Do not automatically write instruction files into another project.
- Test source changes, deletion, secrets, private flags, stale review tokens, concurrent lock, zero eligible results, large content, Unicode, clipboard errors and file-write failures.

Exit: native copy/file export verified, byte-for-byte comparison with the preview passes, all privacy/staleness cases pass, and the demo can show the complete workflow with fictional data. Sharing off must remain the default.

## M3 — Vault lifecycle and recoverability

- Add an in-app restore flow that inspects a backup without replacing the active vault, asks for its password locally, validates integrity/schema/relationships, and preserves the current vault before an explicit replacement step. On failure, keep the original usable. Sharing stays off after restore; in-app restore removes client grants and disables automatic-publication rules before activation.
- Add password change using a separately encrypted, verified replacement and a crash-safe swap. Test wrong current password, interruption, disk-full/no-write cases and reopen with old/new passwords. Existing backups retain their own old passwords. No stored recovery password or hidden bypass.
- Add per-conversation deletion and snapshot retention/purge with clear app confirmation. Define behavior for linked notes/briefs and FTS records; deleted evidence must never leave derived content silently shareable. Explain that original exports, backups and filesystem snapshots can remain.
- Add the destructive recovery acceptance test: create a disposable synthetic vault with branches, notes, briefs, privacy flags and client registrations; take and verify a backup; close and remove that test vault and journals; restore into an empty location; compare data/provenance/policies and exercise a new session. Never target personal vaults or real-data copies with this test.
- Test corrupted/truncated backups, incorrect passwords, unsupported schemas, destination collisions, interruption and restored permissions. Verify unsupported backup filesystems fail safely; support exFAT only after implementing and testing safe no-overwrite publication.

Exit: automated destructive recovery and native restoration both pass, password-change failures preserve a recoverable copy, deletion policy is tested, and RECOVERY documents the actual supported behavior.

## M4 — Multi-provider ingestion

Implement Claude first, then Grok, retaining the same archive/memory/brief/sharing model.

- Establish fixture folders such as `src-tauri/tests/fixtures/<provider>/<observed-format>/`. Each has synthetic input, expected normalization and a manifest describing observed export structure/date, parser contract and supported/omitted fields. Use an official format version only if the provider supplies one; otherwise label an internal observed-format identifier explicitly.
- Obtain/inspect representative official export structures locally. Do not infer that another provider uses ChatGPT's mapping. Do not put sanitized personal chats in fixtures; generate fictional data matching the observed structure.
- Normalize provider-qualified IDs, timestamps/time zones, roles, text blocks, branches where available, titles and source snapshots. Preserve provenance and report omissions; never imply unavailable attachments/tool content were imported.
- Test missing/unknown fields, evolving layouts, duplicates, repeat imports, invalid JSON/ZIPs, size limits, traversal-shaped entries, atomic failure and cross-provider ID collisions.
- Run all privacy, source invalidation, approval and export tests across providers. Add regressions before changing support for a new observed format. Unsupported structures fail clearly without partial insertion.

Exit per provider: synthetic fixtures pass, representative local exports are checked, limitations are documented, and UI/README only advertise verified support. Cross-provider search and curation must work before claiming the three-provider milestone.

## M5 — Release and portfolio presentation

Apple track, deferred to this final stage at the owner’s request: owner checks/enrolls in the appropriate Apple Developer account and completes identity, legal and payment steps. The agent can prepare signing instructions/build tooling, but must not choose a legal identity, accept agreements, purchase enrollment or put credentials in chat/repo. Do not claim enrollment has started or been approved without evidence.

Release work:

1. Finish open security findings or explicitly disposition them with severity, rationale, evidence and accurate user-facing limits. Obtain a separate review of the current changes before claiming independent assurance.
2. Run release checks from a clean reviewed commit; build for each advertised architecture/OS. Verify Developer ID, hardened runtime, notarization acceptance, stapled ticket, Gatekeeper, checksum and source commit.
3. Test install, first-run password creation, import, curation, export/MCP, lock, backup, update and recovery on a clean user environment. No Gatekeeper bypass instructions.
4. Rewrite README around the supported product, a quick start, architecture, privacy boundaries, test evidence and known limitations. Add a 60–90 second demo using only fictional data, emphasizing the exact sharing preview. Never record a personal vault.
5. Prepare release notes, supported-platform matrix, checksum/source provenance and download assets. Keep the landing download disabled until the verified artifact exists. Repository visibility and landing publication require the owner's explicit launch decision.

Exit: release evidence is complete and the owner chooses publication. No enrollment-duration or notarization-duration guarantee is part of the schedule.

## Working and documentation routine

For each bounded change: identify requirement/finding → inspect current code and data boundaries → implement → run meaningful regression checks → inspect native behavior where applicable → update evidence/docs → review staged files for private data → commit/push under the owner's configured Git identity to the private repo.

Update these documents in the same change as behavior:

| Document                          | Owns                                                                       |
| --------------------------------- | -------------------------------------------------------------------------- |
| EXECUTION_PLAN                    | Order, scope, acceptance criteria, dependencies and owner actions          |
| WORK_LOG                          | What changed, status, evidence, remaining work and handoff                 |
| VALIDATION                        | Exact check/environment/result/date and explicit unverified cases          |
| PRODUCT / ARCHITECTURE / SECURITY | User behavior, data model and implemented guarantees/limits                |
| MCP / RECOVERY / RELEASING        | Operational instructions that match shipped behavior                       |
| REVIEW_RESPONSE                   | Disposition of each supplied finding and regression evidence               |
| ROADMAP                           | Short current/next/later summary with links, not a second conflicting plan |
| AGENT_REVIEW                      | Where the next reviewer starts, safe setup and open questions              |

Record only sanitized product-level results. Private-data evidence stays local outside the repo. Record local tests and remote CI separately; queued CI is not a pass. Do not rerun expensive unchanged checks without a reason, and do not run Cargo builds/tests concurrently in one target directory.

## Current execution status and next batch

Completed evidence now includes fictional native import by picker and Finder, duplicate-free repeat import, alternate-branch search, saved-version approval, exact selected clipboard/file export, privacy reclassification and approval resets, verified backup and old-copy cleanup, successful password change, client-configuration screens and sharing-off after full app restart. Codex CLI and Grok Build separately passed the fictional MCP policy matrix. Destructive synthetic recovery and the 5,000-conversation backend benchmark have recorded evidence; neither substitutes for real-export/UI acceptance. Earlier chronological checkpoints remain in WORK_LOG and VALIDATION.

The current failure-handling batch verifies malformed/oversized declared JSON rejection, export cancellation, damaged-backup rejection and retry, and quit before restore confirmation. It reproduced a Mac screen-lock gap with an open Save panel. A dedicated one-way screen-lock observer and regression are implemented. The rebuilt app passed the owner-assisted lock/export retest: sharing disabled and no stale export written. A separate Apple-menu sleep/wake cycle also locked the vault, disabled sharing and preserved data on unlock. See NATIVE_WALKTHROUGH and SECURITY for the notification-delivery limits.

Native acceptance now includes the fictional 5,000-conversation/100,000-message library and a separate 2,000-message conversation with 3,000 memories and 500 briefs. Reader and curation views have bounded rendering, complete loaded-collection search and verified privacy/branch filters. Modal keyboard isolation, final-page focus and search autocorrect fixes have native evidence. See NATIVE_WALKTHROUGH for exact coverage and limits: full-collection IPC memory, maximum-size Markdown and broader accessibility/OS coverage remain open. The later compact/theme walkthrough adds minimum-window checks for normal lock, first run, errors and current workspace controls.

Remaining order before Apple signing:

1. Exercise the new writing workflow with a configured real local MCP client against fictional storage: session save, retry, revision, source manifest resume, reviewed release and fresh shared retrieval. Earlier Codex/Grok retrieval acceptance does not close this gate. Use explicit bounded external-model usage; never send a personal archive as a test shortcut.
2. Complete representative owner-local export/upgrade checks in isolated encrypted copies when the already requested official ZIP arrives. Verify useful consolidation and omitted/uncertain information separately from parser and transport success. No further export request is needed now.
3. Profile larger IPC/maximum-message behavior, exercise additional filesystem publication/interruption boundaries, and finish broader keyboard/resize/error-state coverage. SQLite-capacity and pre-commit killed-writer tests already have evidence. Never use personal storage for destructive tests.
4. Obtain a fresh review of schema 4 and source/publication boundaries, reconcile release evidence and prepare the fictional demo. New provider parsers remain a later milestone; define their scopes explicitly before advertising them.
5. Apple enrollment/signing/notarization and clean-install verification come at the final distribution stage. Public repository/download publication still requires the owner’s explicit launch decision.

Do not treat successful native paths as proof of all crash, hardware, lock-screen or clipboard-manager behavior. Keep open cases in VALIDATION and the next reviewer brief; queued GitHub checks are not passes.

The cream/orange light-default UI and optional dark mode remain the current visual base. Further visual refinement is secondary to working curation and sharing. Source scopes are now changed only through explicit registrations; broad legacy grants are downgraded during migration. Cloud curation is optional and local inference remains planned.

## Provider curation implementation update — 2026-11-20

The owner approved the image flow and authorized implementation on the existing base. Provider-bound archive access, private agent contributions, retry/revision protection, durable incremental source jobs, batch review, narrow opt-in automatic publication and history/withdrawal controls are now implemented. Schema 4 preserves manual approvals and downgrades broad legacy chat permissions. The API and owner UI share the same encrypted store and privacy delivery checks.

Deterministic tests, compiled MCP writing and fictional native review/edit/history checks now have evidence in VALIDATION; those establish only their stated scope. Model-led quality/large-real-export acceptance remains a separate gate; no real archive or provider API expenditure is part of this implementation. Keep Apple enrollment at the end and the repository private. See AGENT_CURATION for the implemented contract and explicitly retained limits.
