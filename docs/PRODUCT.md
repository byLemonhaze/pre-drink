# Product definition

**Pre-Drink — Your AI history, sober and ready.**

## What we are building

Pre-Drink is a free, MIT-licensed, macOS-first memory manager for people who use several AI products. It keeps their imported histories in an encrypted local vault, lets each authorized provider curate only its own history, and makes selected knowledge available as a shared memory layer. Useful context can move between agents without giving each agent the other providers’ raw archives.

This extends the existing app. The encrypted archive, search, manual notes, briefs, sharing preview and recovery system remain the foundation. It is an independent implementation informed by Loominary, not a fork; see [PROVENANCE](PROVENANCE.md). The repository remains private until the owner chooses public launch.

**Cloud curation is optional, never a requirement.** Import, search, manual memory editing, briefs, selected exports, encryption and backups work offline with no model, API key or account. A connected external agent can do the curation work if the owner authorizes it. The app contains no model runner or background inference service. A local-model curator is a desired future option, with an explicit access policy still to be designed.

## Current support and intended audience

The first release serves an individual who wants durable context across AI tools and control over what they share. Large archives make agent-assisted curation a core optional workflow: using it must not require reading the entire archive or approving thousands of individual notes.

Official ZIP import now supports recognized **ChatGPT, Claude and Grok** formats, with versioned synthetic contracts and remaining representative-export acceptance. Each has a matching curator role and session-summary writing. Gemini, Copilot and others are requested next candidates, contingent on validated official formats; see [PROVIDERS](PROVIDERS.md). A compatible local MCP client must be configured; this does not connect an arbitrary ChatGPT or Grok web chat automatically.

Official exports are the deliberate historical input. No scraping, userscripts, browser credentials or live capture. Explicit end-of-session notes are a second input to memory, not a captured transcript. User exports and vault contents never belong in GitHub, even while the repository is private.

## Information layers

| Layer                       | Contents                                                                                                                   | Who can read or change it                                                                                                      |
| --------------------------- | -------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| Provider archive            | Imported JSON snapshots, searchable text, timestamps and branches                                                          | Owner locally; an explicitly authorized matching-provider client can read eligible sources. Foreign-provider access is denied. |
| Provider working memory     | Agent-authored facts, decisions and projects, evidence links, draft revisions and job progress                             | Owner and matching-provider curators. It is private until eligible publication.                                                |
| Shared memory               | Current eligible contributions released after owner review or a separately enabled narrow rule, plus approved manual notes | Authorized clients can read it. Curators revise only contributions owned by their provider; every revision is re-evaluated.    |
| Briefs and selected context | Owner-written specifications/handoffs assembled from eligible memory; exact Markdown exports                               | Owner controls composition, review and delivery. Source restrictions continue to apply.                                        |

These are logical layers inside the existing encrypted database, not separate cloud accounts. Facts, decisions and projects remain the supported note types; broader knowledge fits those notes or freeform briefs. Shared does not mean public or uploaded: it means eligible for an authorized retrieval while the app is unlocked and agent sharing is on, or for a separately reviewed export.

### Archive: evidence, not instructions

Reimports retain unique JSON snapshots and alternate text branches, preserve manual privacy and set the active path from the newest imported snapshot. “Whole story” means material available in the official export. Images, audio, attachments, tool outputs and reasoning channels are not separately imported/rendered. Keep original exports when those assets matter.

Private or detected-sensitive source content is excluded from agent access without an override. Clearing a restriction locally never gives a foreign provider access. Imported instructions are untrusted source text, not permission to run commands or change sharing.

### Working memory: autonomous drafting under a boundary

An authorized curator can read allowed source chunks, create drafts, revise its provider’s contributions and resume a saved batch. It can consolidate repeated information and flag uncertainty; the model’s factual accuracy is not guaranteed by the database. Contributions retain origin, source versions, author/client attribution and revision history. Session summaries are labelled agent assertions and do not pretend the full chat was imported.

Writes use retry keys and expected revisions. Stale updates and fabricated/foreign source references fail. Existing evidence cannot be silently dropped. Jobs record progress; they do not start a model or prove every important idea was extracted. Multiple authorized clients of one provider share that provider’s workspace, with individual author IDs retained in history.

### Shared memory: publication is a separate decision

The default is private drafts and owner review of a complete selected batch. The owner can edit, resolve flags, restore earlier text as a new held revision or withdraw a contribution. Manual notes retain their existing separate save/review workflow.

Optional automatic-publication rules are off by default, separated by provider and archive/session origin, and apply only to future eligible project/decision writes. Facts and recognized URLs/instruction-like text require review. Sensitive content, stale/missing evidence and flagged conflicts stay withheld. Automatic releases are labelled **automatic policy**, never described as human-reviewed. A project or decision is not inherently harmless; classification and detection are imperfect.

Agent edits revoke the previous release before the new revision is evaluated. Source changes/restrictions block derived delivery, including briefs and selected exports. Other providers cannot overwrite a contribution or settle its conflict by changing foreign private notes. The receiving agent must treat all shared text as reference data, never executable authority.

### Briefs: useful specifications and handoffs

The owner assembles selected memory and original writing into freeform Markdown. Suggested structure: problem, user, decisions, requirements, exclusions, acceptance criteria and unresolved questions. A brief records the versions of its supporting notes. Changes, withdrawal or source restrictions block delivery until the evidence is eligible and reviewed again. Agents do not edit briefs in this version.

## Everyday workflows

1. **Offline use:** create a separate vault password, import a ChatGPT ZIP, search, manually save/review notes and build briefs. No cloud curator is needed.
2. **Optional archive curation:** authorize a matching-provider curator, explicitly permit source access and writing, then ask it to process an eligible archive manifest in resumable batches. Review the resulting collection or separately enable a narrow publication rule.
3. **Session handoff:** ask an already connected matching-provider agent to save useful context from the current session. It creates/revises its own session-origin contribution; shared delivery still follows owner policy.
4. **Cross-agent continuity:** another authorized agent retrieves eligible shared facts, decisions, projects or topic context. It receives no foreign raw excerpts or private draft/history metadata.
5. **Deliberate export:** choose notes/briefs in Share context, review the complete Markdown and copy/save exactly that selection. A plaintext copy is outside vault encryption.
6. **Control and recovery:** revoke clients, withdraw contributions, lock, back up and use reviewed restore/password change. These remain owner operations.

## Access, privacy and costs

MCP is a tool connection, not a model or subscription. Default clients are shared-only. Provider-specific raw access and curation require additional owner grants. Provider identity is bound to the credential, not proved by model attestation: never give a Codex credential to Grok or reuse it across providers. Agents cannot change their grants or publication rules, unlock the vault, clear privacy flags or access the key.

Pre-Drink runs no cloud model and sends no archive to a provider itself. If the owner chooses a cloud-backed curator, the permitted context it receives may be processed by that provider and may consume its subscription/API allowance. Returning an archive to its originating provider is still a new processing event. Consent is explicit at registration; the app does not meter or guarantee external costs. Local model support is future work, not a claim that current cloud-agent inference is offline.

Agent sharing starts off and resets on lock/restart. A live client grant covers eligible shared notes/briefs broadly; a topic preview does not reduce it to one project. Per-project grants are future work. Locking or revocation stops future retrieval, not copies already received. Same-account malware, an unlocked compromised machine and receiving providers are outside vault encryption’s protection.

## Persistence and migration

SQLCipher encrypts archive, memory, drafts, revision history, job progress, search indexes and backups; Argon2id derives keys from a separate vault password. There is no reset or hidden recovery key. Use maintained cryptographic libraries and never promise absolute security.

Schema 4 extends the existing store, preserves manual notes/approvals and downgrades broad legacy chat grants to shared-only. Schema 5 adds Claude to the provider constraints and retains existing v4 grants, policies and curation history; Claude’s automation starts off. Schema 6 completes the saved-job constraint migration for both v4 and v5 vaults, so Claude can start resumable jobs after an upgrade. New scoped registrations are required for source access/curation. In-app restore/password change preserves curated data/history, removes client registrations, resets automatic-publication rules and leaves sharing off. Old copies retain old passwords and may retain old grants/rules; see [RECOVERY](RECOVERY.md).

## Acceptance and remaining work

Implemented and deterministically tested: scoped source reads and writes, retry/revision protection, private workspaces, provenance, resumable/incremental manifests, batch release, narrow optional automation, history/withdrawal and source-aware shared delivery. Fictional native checks cover review, editing, held undo, workspace separation and shared preview; compiled MCP tests exercise actual protocol writing. Earlier Codex/Grok model sessions tested retrieval, not this new model-driven curation workflow.

Still open: a fresh fictional model-led write/resume/quality run; representative owner-local export and upgrade acceptance when the requested ZIP arrives; broader scale, accessibility and filesystem failure coverage; fresh independent review; signed/notarized distribution and clean installation. See [VALIDATION](VALIDATION.md) for precise evidence, [AGENT_CURATION](AGENT_CURATION.md) for the contract and [EXECUTION_PLAN](EXECUTION_PLAN.md) for order.

## Future ideas and exclusions

Later candidates: Gemini/Copilot and other validated official export adapters; explicitly scoped cross-provider delegation; an on-device curator; per-project grants; note/brief full-text search and database pagination; tagging and branch visualization; richer supersession/conflict handling; reusable export presets. Neither a local nor cloud agent receives broader access implicitly. The owner may deliberately choose an advanced cross-provider delegate; this is a planned option, not implemented permission today.

No cloud sync, mandatory model subscription, live capture, semantic embeddings, mobile, teams, monetization or demand-testing work is in the current plan. Automatic drafting through an explicitly authorized external agent is now in scope; an always-running background extraction service is not.

## Distribution and portfolio

A simple landing page will explain provider-private histories, optional curation, controlled shared memory and offline use. Download remains disabled until a verified Developer ID signed/notarized artifact exists and the owner chooses publication. Apple enrollment stays at the end. Users run independent local vaults; no hosted vault or account is required. The demo uses fictional data and shows exact outgoing context, a curator revision returning to private state and a denied foreign-source request. Source/docs remain MIT-licensed and private until launch; no public-ready or independent-audit claim is implied by passing tests.

## Optional cross-provider delegation — approved future direction

The default remains provider-separated curation. The owner may optionally choose one curator, such as Claude, to process selected histories from several providers. This is independent of whether the model runs locally or in the cloud. Cloud use is never mandatory.

This advanced mode is not implemented yet. It requires a dedicated owner-issued grant listing the permitted archives, an explicit destination/provider disclosure, revocation and a separate publication choice. Default to private drafts. Preserve source origin and curator identity as different fields; Claude curating Grok evidence must never relabel it as Claude history. Do not let the delegate overwrite foreign contributions under the current provider-ownership rule: define a dedicated delegated workspace and revision authority before implementation. Existing clients, upgrades and imports never acquire this permission automatically.
