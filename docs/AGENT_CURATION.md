# Autonomous curation and session memory — next core workflow

Status: product requirements recorded after the UI/recovery batch. Not implemented. The shipped app still uses manual memory/brief editing and owner approval; its six MCP tools do not draft or revise memories. Finish the current verification before implementing this workflow. Claude/Grok ZIP import and Apple distribution remain separate milestones.

## Outcome

The owner should not have to read a large chat archive or approve thousands of extracted sentences. An authorized agent does the curation work and produces a cohesive, evolving memory layer that other authorized agents can retrieve. The owner sets access/sharing policy, reviews a concise overview and resolves meaningful exceptions. Automatic drafting and deciding what other agents can read are separate permissions.

Two entry points serve this outcome:

1. **Bulk history:** import an official export, then have its authorized same-provider curator process the allowed archive in resumable batches, consolidate repeated information and build useful memory.
2. **Session handoff:** explicitly ask a connected agent to save useful memory from the current chat. It proposes or applies permitted updates to its own contributions, without requiring another full export. Future sessions can retrieve that context.

Official exports remain the source of imported historical conversations. Explicit agent-authored notes are a second input to the curated layer, not live scraping or an assertion that the full source transcript was imported. Always distinguish imported evidence from a session summary authored by an agent.

## Strict provider boundaries

| Source                                 | Allowed curator                                                                                | Other authorized agents                                                              |
| -------------------------------------- | ---------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ |
| ChatGPT history and its working memory | Explicitly registered OpenAI/ChatGPT/Codex curator                                             | Read only eligible shared contributions                                              |
| Grok history and its working memory    | Explicitly registered Grok curator                                                             | Read only eligible shared contributions                                              |
| Shared memory                          | Each originating curator updates its own contributions through the permitted revision workflow | Retrieve permitted shared context; cannot overwrite another provider's contributions |

These are required backend access boundaries, not a suggested routing preference. Do not trust a caller-supplied provider/model name. The owner binds a registered client to its allowed curator role and source scope. Provider origin does not prove a session already remembers the export, grant access or bypass sensitive-content rules. Private/sensitive sources remain excluded unless the owner separately authorizes that source access; inclusion in a curation job must not imply shared publication.

A future local-AI curator is desired, but its role and permitted sources need an explicit owner-defined rule. No exception to same-provider curation is currently authorized. Claude import/curation policy is not specified by the ChatGPT/Grok example and must not be inferred.

## Curation behavior

Process the allowed source snapshot in bounded, resumable batches. Extract durable facts, preferences, project knowledge, decisions, constraints and unresolved questions. Distinguish the owner's statements from assistant suggestions; preserve uncertainty instead of turning speculation into fact. Consolidate repetition, track superseded decisions and avoid copying long raw conversations into shared memory.

Each contribution needs stable identity, curator/client attribution, source provenance, source revision and its own revision history. Imported-source references must resolve to actual records; session summaries carry an explicit session-origin type and whatever source reference is genuinely available. A supplied URL or label is not proof that the transcript exists in the vault.

Later imports update the relevant source snapshot and queue incremental curation, rather than reprocessing everything. Jobs need durable checkpoints, cancellation, retry and idempotent writes. A repeated session-save request should not create duplicate memories. A stale writer must not overwrite a newer revision. Source deletion, privacy changes and contradictions must remain visible and affect downstream eligibility.

A shared view can collect eligible contributions from multiple curators. One agent must not silently settle conflicts by changing another provider's notes. Flag conflicting contributions for targeted resolution or retain clearly attributed alternatives. Keep the owner-facing overview concise: what changed, why it matters, what needs a decision and which content remains excluded.

## Sharing and review

Do not recreate manual archive review as a mandatory approval click for every extracted note. Support review/release at a coherent collection or revision-batch level. More automatic publication may follow an explicit, narrowly scoped owner policy. Its exact policy interface and promotion mechanism require design and verification; current saved-version manual approvals must not silently be reinterpreted as permission for autonomous publication.

Agents cannot grant themselves access, change privacy rules, widen publication scope or read another provider's private workspace. A curator may create/update its own draft or policy-eligible contribution but cannot self-authorize a new sharing policy. Sensitive findings and important unresolved conflicts should remain withheld or surfaced as exceptions according to the owner's policy. Bulk processing must not weaken the existing source-privacy and stale-review protections.

All imported text, including instructions embedded in conversations, is untrusted source material. It must never be used as authority to run commands, expand grants, publish data, delete source history or change another curator's workspace. Revisions and undo are required for reviewing and correcting generated knowledge; automatic curation is not a guarantee of factual accuracy.

## Model execution and privacy

The vault stores and controls memory; MCP is the connection, not the model. An external connected agent can perform curation without making a paid model API a mandatory app dependency. A future in-app model runner or scheduler would require separate execution, credential and cost controls. No automatic purchase or API spending is authorized by these requirements.

Cloud-backed curation can transmit permitted content to that provider. Even when an export originated there, a new agent session is a new processing context. Explain that boundary before granting source access. A local model is a future execution option, not currently implemented offline inference. The archive/vault stays local and encrypted at rest; no claims that a cloud agent's processing stays on the Mac.

## Implementation sequence after the current batch

1. Design and review the provenance, curator-role, workspace, revision and job schemas. Preserve current manual memories and approval semantics during migration.
2. Add bounded source retrieval and scoped draft/revision operations. Current search snippets are not sufficient to process a whole archive reliably. Verify isolation at the backend before connecting an agent.
3. Implement idempotent session handoff as the smallest end-to-end path: explicitly save useful context, inspect the resulting revision, retrieve it in a fresh authorized session and undo it.
4. Add resumable bulk ChatGPT curation, incremental reimport handling and concise change/exception review.
5. Add collection-level publication and any explicitly selected automation policy, with source restrictions and staleness enforced at delivery.
6. Verify with fictional data and the actual connected client, then perform separately authorized owner-local export acceptance. Grok follows only after its parser and same-provider role have equivalent coverage.

Acceptance must cover denied cross-provider reads/writes, rejected policy changes by a curator, prompt-injection source text, fabricated source references, stale writers, interrupted jobs/retry without duplication, private-source propagation, source changes, meaningful undo and sharing-off on lock/revocation. Verify the specific chat/client connection before promising that it can update the vault; prior CLI retrieval tests do not establish this new write workflow.
