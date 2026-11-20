# Execution plan

Agreed direction: 2026-11-19. Implementation baseline: `589967e`.

## Outcome

Pre-Drink should be a useful daily macOS app and a credible portfolio project: import official AI exports into a private encrypted archive, curate facts/decisions/projects, assemble briefs, preview exactly what will be exported, and deliberately provide context to another product. Manual local features are free and require no model API. The main demonstration is the sharing preview and the control it gives the owner.

Current implementation supports ChatGPT only. Claude and Grok are planned milestones, not available features. Source is independently implemented, informed by Loominary; it is not a Loominary fork.

## Non-negotiable boundaries

- Keep the repository private and public downloads/Pages disabled until the owner explicitly chooses to launch.
- Never commit real exports, vaults, personal chat text, screenshots of personal data, credentials, passwords, signing material or raw private test logs. A private repository is not an exception.
- Use official exports only. No live capture, scraping, browser credentials or userscripts.
- Keep the encrypted archive as the source of truth. Sharing, plaintext exports and cloud-backed agents cross separate boundaries.
- No automatic extraction, cloud sync, monetization, demand tests, semantic search, Gemini, mobile or team features in this execution plan.
- Approval cannot override private sources or detected credentials. Changing content requires review again.
- Test personal data only in isolated encrypted copies after a verified backup. Destructive recovery tests use disposable synthetic vaults exclusively.
- A passing test suite does not establish a formal security audit or a signed public release.

## Status vocabulary and evidence

**Planned** means agreed but not built. **In progress** means active work remains. **Implemented** means code exists; validation may remain. **Verified** requires recorded evidence for the stated environment. **Blocked** names an external prerequisite. **Deferred** is outside the current milestone. Do not use “done” to combine these states.

The implementation/evidence ledger is [WORK_LOG](WORK_LOG.md). [VALIDATION](VALIDATION.md) records checks and limits. [REVIEW_RESPONSE](REVIEW_RESPONSE.md) maps the supplied review findings. This file owns milestone order and exit criteria; the roadmap is its short index.

## Sequence and planning estimate

M1 → M2 → M3 → M4 → M5. At the owner’s request, Apple enrollment is deferred to the final release stage. Development and all independent verification proceed now. Work can prepare fixtures or release documentation early, but must not present unfinished milestones as delivered.

Planning allowance: roughly 3–5 focused working days for a dependable ChatGPT-only candidate, and 1–2 weeks for the broader release candidate, excluding Apple account processing and external review. These are effort estimates, not calendar commitments or a promise of unattended work between sessions. Re-estimate after M1 using real export sizes, native validation and remaining defects. Provider format changes and recovery failures can extend the work. A release candidate is not a public release until every release gate is met.

| Milestone                              | State at plan creation                        | Exit evidence                                                                                                                              |
| -------------------------------------- | --------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| M1: dependable ChatGPT workflow        | In progress                                   | Current native approval flow, false-positive regressions, isolated upgrade verification, a real MCP client, and clear build/vault identity |
| M2: deliberate sharing preview         | Implemented; basic native flow verified       | Preview/export equality, eligibility revalidation, no surprise source metadata, selective copy/file export                                 |
| M3: vault lifecycle and recovery       | Implemented; lifecycle validation in progress | Tested restore UI, password change, deletion/retention, destructive synthetic recovery exercise                                            |
| M4: Claude, then Grok                  | Planned                                       | Provider/version fixtures, normalization and cross-provider privacy/import tests, local representative export checks                       |
| M5: release and portfolio presentation | Planned; Apple prerequisites external         | Signed/notarized artifact, clean-machine installation, release evidence, fictional-data demo and reviewed documentation                    |

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

- Add an in-app restore flow that inspects a backup without replacing the active vault, asks for its password locally, validates integrity/schema/relationships, and preserves the current vault before an explicit replacement step. On failure, keep the original usable. Sharing stays off after restore; explain/review restored client grants before enabling them.
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

## Next execution batch

M1 and selected-sharing batches are committed with passing remote CI. The lifecycle batch now implements reviewed restore/password changes, strict active-generation selection with preserved recovery copies, restored-client reset, conversation deletion and raw-snapshot cleanup. Basic native restore and deletion-preview cancellation now pass with fictional data. Complete owner-driven password-change verification and broader lifecycle/failure coverage before considering M3 closed. Startup now has measured launch evidence and recovery paths; the historical intermittent stall is not reproduced and broader startup verification remains open. Outstanding representative-data upgrades, physical-drop, installed-client and full lifecycle/keyboard checks remain explicit M1/M2 gates. M4 providers follow the agreed sequence; Apple enrollment stays at M5. No public launch is implied by a passing local suite.

The ChatGPT import-acceptance batch adds versioned fictional parser contracts and native picker/reimport/omission/branch-search/privacy/cancellation evidence. Physical file drop remains open; native picker checks must not be counted as drop verification. Next independent M1 priority is installed-client MCP setup and remaining representative-export/upgrade evidence; M3 still needs broader native failure and owner-entered password-change checks.

Codex CLI now passes the fictional installed-client retrieval/privacy/lifecycle matrix through the backend fixture. This closes that client's protocol/tool-result check, not native configuration-screen or app-restart verification. Grok Build has also passed those eight policy phases after owner sign-in; Claude is deferred. Next independent work is remaining representative-export/upgrade evidence and native recovery/lifecycle coverage. The owner requested a later UI overhaul after core functionality is reliable; Apple enrollment remains last.

The destructive recovery gate now has backend evidence for both manual copying and the app's reviewed restore after total synthetic vault removal, including persisted selection, exact data/policy comparison and credential reset. Damaged encrypted pages, truncation, retry and backup-source removal after inspection are covered. Native connection/recovery checks remain open because computer control failed to attach to the separate acceptance build; do not repeat the slow attachment indefinitely or substitute backend tests for screen evidence. Next owner-assisted native task is password change in disposable storage, followed by connection setup/restart and physical drop when reliable control is available.

Client-specific settings generation is now implemented to remove manual JSON-to-TOML translation from Codex/Grok setup. Both installed clients have read-only configuration-parser evidence in addition to the earlier tool-call checks. Native configuration-screen acceptance still needs reliable Mac control or an owner walkthrough; it is not closed by serializer tests. Keep the remaining lifecycle/export gates ahead of provider expansion and Apple signing.

The owner prioritized Round 3 review findings before further polish. Individual flag clearance, stronger on-open detection, backup-gated previous-vault cleanup and Mac-only confidential clipboard export are implemented in this batch. Their automated evidence and remaining native limitations are in VALIDATION and REVIEW_RESPONSE. Next is a disposable native walkthrough of the new controls alongside outstanding connection/restart, owner-entered password change and physical-drop gates. Representative export/upgrade checks remain owner-local and never enter Git. ChatGPT remains the only import provider; future provider work and UI overhaul are not part of this batch. Apple enrollment remains last.

Native Round 3 acceptance now covers individual flag review/application, sensitive exclusion, approval resets, backup-gated/typed old-copy removal with explicit owner confirmation, Codex/Grok configuration screens, native copy/paste and a full app quit/relaunch with sharing reset. The only product change in this walkthrough batch is cleanup-confirmation spacing. The spacing fix was visually verified and fictional clients revoked before locking/quitting. Native password change, physical cross-window drop and representative data checks remain open; see NATIVE_WALKTHROUGH for the detailed evidence rather than treating every native gate as closed.
