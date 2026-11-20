# Product brief

## Identity and purpose

**Pre-Drink — Your AI history, sober and ready.**

A free, open-source, macOS-first local app that turns official AI chat exports into a private searchable archive and a curated knowledge layer other agents can query. The archive preserves evidence; reviewed memory and useful handoffs are the product.

The current implementation supports ChatGPT only. Claude and Grok are scheduled after the dependable ChatGPT, selective-sharing and recovery milestones; see EXECUTION_PLAN. Official exports are a product decision: no scraping, browser credentials, userscripts or live capture. Users import a newer export to refresh history. Exports may omit material unavailable from the provider, and media support is separate from text parsing.

## Core principles

1. Local by default: no application cloud sync, telemetry, hosted model, remote fonts, or required account.
2. Preserve source material: curation does not rewrite original chat text.
3. Deliberate memory: people decide what becomes durable knowledge.
4. Sharing is an explicit permission boundary, not an automatic consequence of import.
5. Encryption covers actual storage, search indexes and backups, rather than hiding rows only in the interface.
6. Use maintained cryptographic implementations; never promise absolute or permanent security.
7. Build working native flows with persistent data. Describe remaining release gates honestly.

## Three-layer information model

### Archive: the whole available story

Stores official conversation JSON, searchable user/assistant text, timestamps, parent relationships and branches. Reimports retain earlier unique JSON snapshots and text branches, and preserve manual privacy flags. The newest imported snapshot defines the active path. It is evidence, including uncertainty and discarded ideas—not automatically trusted instruction.

The phrase “whole story” means the conversation material present in imported exports. This version does not separately import/render images, audio, attachments, tool outputs or reasoning channels. Keep original exports where those matter.

### Memory: reviewed knowledge

A user selects a message and saves a rewritten **fact**, **decision**, or **project** note. Notes can also be authored independently. They can be edited, deleted, marked private, or explicitly approved. Saving always returns the note to draft. A separate review displays the saved version; approval is bound to that exact version and rejected if anything changes. Source-linked notes keep provenance and a route back to the conversation. Source restrictions propagate; approval never overrides privacy.

Example using invented project data: a long discussion about an offline notebook becomes “The first release stores notes locally and supports one owner.” The archive remains unchanged. The note is editable and unapproved until its owner reviews it.

### Briefs: usable specifications and handoffs

A brief combines selected memories and original writing into editable Markdown. Suggested structures: problem, intended user, decisions, requirements, exclusions, acceptance criteria, unresolved questions. The current editor is freeform, not an automatic summarizer.

Adding memory records its reviewed version. Editing, deleting, restricting or unapproving that source blocks the brief from agents until eligible and reviewed again. A deleted source permanently blocks that provenance chain; create a new reviewed brief if needed. This prevents a previously approved document from silently outliving its evidence.

Approval permits eligible clients to retrieve a brief. It does not publish it, make a network request or enable sharing. Current client grants cover all eligible approved notes/briefs; per-project and per-brief grants are future work.

## Primary workflows

1. **Initialize:** create a separate strong vault password, understand that it cannot be reset, then import an official ZIP.
2. **Find:** search across active and alternative text branches; explicitly enable restricted local review when needed.
3. **Curate:** keep a useful message as a note; rewrite, classify, review and approve it.
4. **Synthesize:** assemble a specification from reviewed notes and edit the narrative.
5. **Share deliberately:** authorize a named client, choose its scope, inspect context, and separately enable sharing.
6. **Revoke and lock:** stop future access, lock the library, and retain encrypted backups.

## MCP and costs

MCP is the standard tool interface between an assistant and this vault. An agent can retrieve facts, decisions, projects, topic context or—only with additional permission—safe conversation matches. Agents cannot approve/edit memory, decrypt the database independently, or undo privacy flags.

No model is needed for import, manual editing, encryption, search or context assembly; these incur no model API charges. The assistant consuming context may use subscription allowances or separately billed API tokens. OpenAI's ChatGPT and API billing are separate: [official billing explanation](https://help.openai.com/en/articles/9039756-managing-billing-for-chatgpt-and-the-api-platform).

Potential assistance later: an on-device model suggests clearer notes for human approval, or an explicitly enabled cloud provider assists selected text. Neither exists in this version. Cloud assistance must disclose data flow and estimated usage before sending content. The product is free and open source. Core local import, curation and controlled sharing must not require a subscription or hosted service. Optional third-party model usage could still incur that provider’s charges; it is not part of the current product.

## Privacy expectations

New imports are not shared automatically. Agent scope defaults to approved memory/briefs. Raw conversations require a separate grant. Private/sensitive sources and derived content are excluded in the backend. Sharing starts off and resets on lock/restart. Authorization warns that a cloud client can send received context to its provider; a local connection does not make that provider local.

The repository contains source, generic product documents and synthetic fixtures only. No real exports, vaults, personal conversation transcripts, client credentials, signing keys or private screenshots belong in source control—even in a private repository.

## Acceptance and release status

Implemented: encrypted persistent storage, ChatGPT import/search, memory CRUD, curated briefs, selected Markdown preview/copy/file export, source-aware privacy, encrypted backups, reviewed in-app restore/password change, conversation deletion/snapshot cleanup, authenticated local MCP and lock lifecycle. Automated checks use synthetic fixtures; an earlier native curation flow was exercised. The current separate-review and flag-recheck screens have basic synthetic native-flow evidence; full failure, lifecycle, representative-export and installed-client checks remain open.

Public-release gates remain: representative real export validation, large-library behavior, physical drag-and-drop, individual client setup, independent security review, Developer ID signing and notarization. See the roadmap and review guide. “Real implementation” does not imply these gates have already passed.

The agreed work ahead, dependencies, evidence requirements and owner actions are recorded in [EXECUTION_PLAN](EXECUTION_PLAN.md); [WORK_LOG](WORK_LOG.md) distinguishes implemented work from verification.

## Future ideas, not current promises

- Claude/Grok adapters after representative official fixtures exist.
- Tags, better topic matching, result pagination and branch visualization.
- Per-brief/project client permissions and reusable export presets.
- Superseded decisions, revision history, contradictions and stale-memory review.
- Local drafting assistance with approval, never silent extraction.
- Guided retained-generation cleanup and broader recovery/filesystem verification.

No cloud sync, semantic/embedding search, automatic fact extraction, Gemini, mobile, live capture, or team features in the current scope.

## Public distribution

A simple landing page explains the three layers, privacy boundaries and supported platform. Its primary download becomes available only after a signed/notarized release is published. Source stays available under MIT. Users run independent local vaults; there is no shared hosted database, required account, or upload to the project repository.

## Deliberate sharing

The Share context screen assembles an explicit selection of memories and briefs, shows the entire outgoing Markdown and requires a review confirmation before copy or file save. Restricted/draft/source-blocked items are excluded and explained locally. Delivery uses the same backend assembly and revalidates the reviewed saved/source versions and unlock session. It exports no raw-chat metadata or automatic source excerpts. The owner chooses any subsequent destination app themselves. See [SHARING](SHARING.md).

This creates plaintext outside the vault; clipboard history, synced folders and receiving products may retain it. A live MCP grant is broader than a selected export and is managed separately. Changing imported source text/role now returns derived memories to draft and blocks dependent briefs until reviewed again.

## Vault lifecycle

Restore is available while locked or unlocked and does not require access to the old vault password. It inspects a standalone encrypted backup, shows its counts and impact, preserves the previous encrypted vault, and switches only after explicit review. Password change similarly verifies the current password and prepares a replacement under the new password. Both remove agent registrations and leave sharing off. Older recovery copies/backups keep their previous passwords and need separate cleanup after verification.

Conversation deletion removes the archive record, its text/search entries and raw snapshots. Derived memories remain as blocked drafts with missing-source markers; briefs cannot silently share that deleted evidence. Snapshot retention is a separate explicit cleanup of older raw JSON versions, preserving the latest imported raw version and all message branches. No deletion claims secure erasure of independent copies. See RECOVERY and SECURITY.

## Proposed later: agent workspaces and reviewed shared memory

This is a recorded product proposal, not implemented behavior or authorization to expand MCP access. Complete the current interface/verification batch first. ChatGPT remains the only supported import provider in this phase.

The owner's required boundary is strict same-provider curation: ChatGPT imports may be curated only by an explicitly authorized OpenAI/ChatGPT/Codex curator; Grok imports may be curated only by an explicitly authorized Grok curator. An agent must not read another provider's private working space or curate its source memory. Other authorized agents may read eligible owner-approved material in the shared layer, but may not overwrite another provider's source notes or shared revisions. Updates from the originating curator go through a new draft/proposal and saved-version review.

Preserve the archive as source evidence and use separate working-memory spaces before promotion into shared memory/briefs. Enforce provider provenance and registered-client permissions in the vault backend; do not trust a caller's claimed model name or rely on prompt instructions. Being from the same provider does not prove that an agent session already has the exported context, grant authentication, or bypass privacy checks. A possible local-AI curator needs a separately designed, explicitly approved access rule; no cross-provider exception is currently authorized.

Initial automation should propose changes, not silently approve them. A proposal carries its author/client, source references, prior revision and a readable diff. Shared publication needs owner approval of the saved revision. Conflicts, stale sources, deletion and new privacy restrictions must block promotion or invalidate downstream approval under the existing source-safety rules. Separate workspaces do not erase privacy inherited from sources. Untrusted imported text remains reference material, never permission to change policy or access another workspace.

A future design must define per-workspace read/write grants, explicit source access, proposal-only versus direct draft edits, audit/revision history, rollback, revocation, lock behavior and concurrent-update checks. Clients must not approve their own work, broaden their grants or cross workspace boundaries. More automatic promotion would be a separate explicit owner policy, scoped to selected content and reversible revisions; it is not a default or part of current v1.

The app and vault remain local. A cloud-backed agent can transmit the content it reads to its provider: connection review must make the exact allowed source/workspace scope and that boundary understandable. No API key, model subscription or automatic inference requirement is added by this proposal. Any future paid inference must be separately selected and authorized by the owner.
