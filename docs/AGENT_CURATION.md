# Provider-isolated curation and session memory

Status: the first provider-isolated curation implementation is present in this branch: authenticated scoped writes, source manifests/checkpoints, revisions, private drafts, owner batch review, narrow opt-in automatic publication, and app controls. This extends the existing encrypted vault and manual memory system. Deterministic backend/transport and fictional native workflow checks have evidence; Codex and Grok each passed a three-session fictional write/resume/retrieval run with narrow semantic evidence. Broader quality and Claude remain open; see CURATION_ACCEPTANCE. See VALIDATION. ChatGPT, Grok and Claude ZIP adapters and matching roles are implemented with fictional contract coverage; representative exports, local inference and Apple distribution remain separate acceptance work.

## Outcome

The owner should not have to read a large chat archive or approve thousands of extracted sentences. An authorized agent does the curation work and produces a cohesive, evolving memory layer that other authorized agents can retrieve. The owner sets access/sharing policy, reviews a concise overview and resolves meaningful exceptions. Automatic drafting and deciding what other agents can read are separate permissions.

Two entry points serve this outcome:

1. **Bulk history:** import an official export, then have its authorized same-provider curator process the allowed archive in resumable batches, consolidate repeated information and build useful memory.
2. **Session handoff:** explicitly ask a connected agent to save useful memory from the current chat. It proposes or applies permitted updates to its own contributions, without requiring another full export. Future sessions can retrieve that context.

Official exports remain the source of imported historical conversations. Explicit agent-authored notes are a second input to the curated layer, not live scraping or an assertion that the full source transcript was imported. Always distinguish imported evidence from a session summary authored by an agent.

## Default provider boundaries and current enforcement

| Source                                 | Allowed curator                                                                                | Other authorized agents                                                              |
| -------------------------------------- | ---------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ |
| ChatGPT history and its working memory | Explicitly registered OpenAI/ChatGPT/Codex curator                                             | Read only eligible shared contributions                                              |
| Grok history and its working memory    | Explicitly registered Grok curator                                                             | Read only eligible shared contributions                                              |
| Claude history and its working memory  | Explicitly registered Claude curator                                                           | Read only eligible shared contributions                                              |
| Shared memory                          | Each originating curator updates its own contributions through the permitted revision workflow | Retrieve permitted shared context; cannot overwrite another provider's contributions |

These are required backend access boundaries, not a suggested routing preference. Do not trust a caller-supplied provider/model name. The owner binds a registered client to its allowed curator role and source scope. Provider origin does not prove a session already remembers the export, grant access or bypass sensitive-content rules. Private/sensitive sources remain excluded with no agent-access override in this implementation; inclusion in a curation job never implies shared publication.

A future local-AI curator is desired, but its role and permitted sources need an explicit owner-defined rule. The owner subsequently approved a future explicit cross-provider delegation option; it is not implemented by today’s grants. Claude now has the same own-provider rule as ChatGPT and Grok. A local runner and delegated source scopes require separate implementation, never a silent exception.

## Curation behavior

Process the allowed source snapshot in bounded, resumable batches. Extract durable facts, preferences, project knowledge, decisions, constraints and unresolved questions. Distinguish the owner's statements from assistant suggestions; preserve uncertainty instead of turning speculation into fact. Consolidate repetition, track superseded decisions and avoid copying long raw conversations into shared memory.

Each contribution needs stable identity, curator/client attribution, source provenance, source revision and its own revision history. Imported-source references must resolve to actual records; session summaries carry an explicit session-origin type and whatever source reference is genuinely available. A supplied URL or label is not proof that the transcript exists in the vault.

Later imports update the relevant source snapshot. When the agent explicitly starts its next manifest, new/changed/unacknowledged evidence is selected; imports do not automatically start or schedule a model. Jobs need durable checkpoints, cancellation, retry and idempotent writes. A repeated session-save request should not create duplicate memories. A stale writer must not overwrite a newer revision. Source deletion, privacy changes and contradictions must remain visible and affect downstream eligibility.

A shared view can collect eligible contributions from multiple curators. One agent must not silently settle conflicts by changing another provider's notes. Flag conflicting contributions for targeted resolution or retain clearly attributed alternatives. Keep the owner-facing overview concise: what changed, why it matters, what needs a decision and which content remains excluded.

## Sharing and review

Do not recreate manual archive review as a mandatory approval click for every extracted note. Support review/release at a coherent collection or revision-batch level. More automatic publication may follow an explicit, narrowly scoped owner policy. The app now provides separate per-provider switches for archive and session publication. Both default off and affect future writes only. Facts, links and instruction-like text require review. Only eligible projects/decisions may use automatic publication. Current manual approvals are not reinterpreted as automation consent.

Agents cannot grant themselves access, change privacy rules, widen publication scope or read another provider's private workspace. A curator may create/update its own draft or policy-eligible contribution but cannot self-authorize a new sharing policy. Sensitive findings and important unresolved conflicts should remain withheld or surfaced as exceptions according to the owner's policy. Bulk processing must not weaken the existing source-privacy and stale-review protections.

All imported text, including instructions embedded in conversations, is untrusted source material. It must never be used as authority to run commands, expand grants, publish data, delete source history or change another curator's workspace. Revisions and undo are required for reviewing and correcting generated knowledge; automatic curation is not a guarantee of factual accuracy.

## Model execution and privacy

Cloud curation is optional, never mandatory. Offline import/search/manual editing/briefs/exports/backups remain usable without connecting any agent. The vault stores and controls memory; MCP is the connection, not the model. An external connected agent can perform curation without making a paid model API a mandatory app dependency. A future in-app model runner or scheduler would require separate execution, credential and cost controls. No automatic purchase or API spending is authorized by these requirements.

Cloud-backed curation can transmit permitted content to that provider. Even when an export originated there, a new agent session is a new processing context. Explain that boundary before granting source access. A local model is a future execution option, not currently implemented offline inference. The archive/vault stays local and encrypted at rest; no claims that a cloud agent's processing stays on the Mac.

## Implementation and remaining acceptance

1. Design and review the provenance, curator-role, workspace, revision and job schemas. Preserve current manual memories and approval semantics during migration.
2. Add bounded source retrieval and scoped draft/revision operations. Current search snippets are not sufficient to process a whole archive reliably. Verify isolation at the backend before connecting an agent.
3. Implement idempotent session handoff as the smallest end-to-end path: explicitly save useful context, inspect the resulting revision, retrieve it in a fresh authorized session and undo it.
4. Add resumable bulk ChatGPT curation, incremental reimport handling and concise change/exception review.
5. Add collection-level publication and any explicitly selected automation policy, with source restrictions and staleness enforced at delivery.
6. Verify with fictional data and the actual connected client, then perform separately authorized owner-local export acceptance. Grok and Claude parser/scope contracts now have fictional tests; actual model-led and representative-export acceptance still need their own evidence.

Acceptance must cover denied cross-provider reads/writes, rejected policy changes by a curator, prompt-injection source text, fabricated source references, stale writers, interrupted jobs/retry without duplication, private-source propagation, source changes, meaningful undo and sharing-off on lock/revocation. Verify the specific chat/client connection before promising that it can update the vault; prior CLI retrieval tests do not establish this new write workflow.

## Implemented contract (schemas 4–6)

- Owner registration binds a credential to ChatGPT/Codex, Grok or Claude. This is a credential boundary, not model attestation: never reuse a credential in another provider's client. Generic clients stay shared-only. Multiple explicitly authorized clients of the same provider share that provider workspace; client attribution remains in revision history.
- Upgrading from schema 3 preserves manual memories and approvals. Legacy broad chat grants become shared-only; the owner must issue new provider-specific registrations. The original archive, passwords and encrypted storage format are retained.
- `curate_memory` is one typed MCP tool with `status`, `list_sources`, `read_source`, `list_notes`, `write`, `history`, `undo`, `jobs`, `job_sources` and `save_job` actions. No action accepts a provider override or changes owner policy.
- Write uses an idempotency key and expected revision. Retries return the original receipt (not a claim about current publication). Changed payloads with the same key, stale writers and foreign IDs fail. Source references include current evidence versions and cannot be dropped. Each update revokes the old shared revision before any newly permitted release.
- Source versions cover text, role, branch/parent, active-path and timestamp changes. Identical reimports keep evidence versions stable. Missing, restricted or changed evidence blocks every derived shared delivery, including dependent briefs and owner exports.
- Imported sources remain raw evidence; session summaries are explicitly labelled agent assertions, with a private session label. Shared payloads expose provider, origin, contribution revision and `owner_review` versus `automatic_policy`, but never archive IDs/titles or private session labels.
- Owner batch review binds up to 30 complete contributions and current source eligibility to a hash. All selected items must still be eligible and unchanged. Conflict flags can be resolved through an owner edit; edits/undo create a private revision. Withdrawal stops future sharing and curator resurrection while preserving history.
- `save_job` with no ID snapshots eligible, not-yet-acknowledged source IDs and versions. The manifest is stable; raw source content is read live only if still permitted. The agent saves contributions before advancing the ordinal checkpoint. Jobs and writes are retry-safe; checkpoints use expected revisions. Restarting the client resumes the saved job. New jobs include unacknowledged/new/changed evidence, including new IDs before an older scan cursor. Acknowledgement records the agent's claim of completed work, not proof of semantic completeness.
- No background model runner is started by the app. The connected external agent performs inference and must choose to continue/resume. Curation saves no API keys, makes no provider API calls, and creates no mandatory paid dependency.
- Restore/password rotation retains encrypted drafts/history but removes client registrations, disables automatic publication policies and leaves sharing off.

## Limits and review priorities

The phrase/URL detector is deliberately conservative and incomplete. It does not prove prompt-injection safety. All shared content remains untrusted reference data, and a receiving model must not treat it as permission to execute commands or change access. Conflict detection and factual consolidation depend on the curator; setting a conflict flag is enforced, identifying every contradiction is not guaranteed. Even project/decision prose can be personal: automatic sharing requires explicit owner opt-in and should remain off for cautious use.

Private/sensitive raw content has no override in this implementation. Local AI has no implemented curator role; adding one must enforce explicit source scopes and the provider-separated default. The existing facts/decisions/projects taxonomy is retained; general knowledge can be represented in these notes or manually curated briefs. Per-project access grants, automatic cross-note conflict discovery, background scheduling and a dedicated knowledge type remain future work.

This implementation does not prove that a particular ChatGPT web chat can reach the local app. A compatible local MCP client must be configured. Codex and Grok now have separate fictional writing acceptance in CURATION_ACCEPTANCE; neither establishes general summary quality. Topic packs match saved summary text lexically, so keep recognizable project/topic names when curating. The already-requested real ChatGPT export is still awaited; no personal archive was used to implement this feature.

## Planned advanced delegate option

The owner approved an optional mode in which one chosen agent can curate selected providers’ histories, including all supported histories if explicitly selected. This supersedes “never under any configuration” as a future product restriction; the current backend remains strictly provider-bound. Cloud processing and source scope are independent choices.

Before implementing it: store source providers separately from the curator identity; issue a dedicated revocable grant with explicit source selection; disclose the cloud destination and potential usage; write into an attributable delegated workspace; keep publication separately controlled and off initially; enforce privacy/staleness across every source; test denied unselected sources and migration with no automatic scope expansion. Reading several archives does not authorize rewriting existing foreign-owned contributions. No current credential can activate this mode through a tool argument.
