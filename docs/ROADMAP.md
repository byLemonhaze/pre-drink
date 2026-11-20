# Roadmap

The detailed agreed order and exit criteria are in [EXECUTION_PLAN](EXECUTION_PLAN.md). Completed work and open verification are in [WORK_LOG](WORK_LOG.md); checks are in [VALIDATION](VALIDATION.md).

## Current implementation

ChatGPT, Claude and Grok official ZIP adapters with versioned fictional contracts; branch/snapshot archive; FTS5; separate-password SQLCipher vault; editable memory and briefs; source-aware sharing rules; selected Markdown preview/copy/file export; local authenticated MCP; encrypted backups; manual/idle/macOS lifecycle locking. Manual saves produce drafts and require a separate saved-version review. Provider curators now have private drafts, revisions, resumable source manifests and batch release; narrow automatic publication is separately opt-in. Cloud curation is optional; all manual local workflows remain available without a model. No local-model runner is implemented. The native interface has cream/orange light mode by default and optional charcoal/orange dark mode; static landing assets remain unpublished.

This is a working private pre-release. Basic native review, sharing, recovery and fictional import workflows have evidence. Codex CLI and Grok Build have passed the fictional MCP client policy matrix. Native Finder drop/reimport, password change, Codex/Grok configuration screens and sharing-off after restart are now verified with fictional data. Representative real exports, full keyboard/lifecycle coverage and distribution gates remain open. Grok/Claude adapters and matching scopes are now implemented; representative exports are still unverified; MCP-client compatibility is tracked separately in MCP_ACCEPTANCE.

## Agreed execution order

1. **M1, in progress:** dependable ChatGPT workflow; contextual sensitivity detection and safe reclassification; build/vault identity; native approval, isolated upgrade, real export and MCP checks.
2. **M2, implemented; basic native flow verified:** make exact sharing preview the centerpiece; selective copy and Markdown export with privacy/staleness enforcement.
3. **M3, implemented; validation in progress:** reviewed in-app restore/password change, preserved encrypted generations, client reset, conversation deletion and explicit raw-snapshot cleanup. Disposable recovery and lifecycle regression suites pass; native and durability limits remain documented.
4. **Provider curation, implemented; acceptance in progress:** scoped archive curation and explicit end-of-chat memory writes, durable/incremental jobs, revisions and reviewed/policy-controlled shared collections. Real-export and model-quality acceptance remain; see [AGENT_CURATION](AGENT_CURATION.md).
5. **M4, implementation expanded:** ChatGPT/Grok/Claude adapters and curator roles; finish native and representative-format acceptance. Gemini, Copilot and additional official adapters follow validated structures, not name-only UI support.
6. **M5, planned:** signed/notarized release, clean installation test, fictional-data demo, strong README and owner-approved public launch.

Apple enrollment is deferred to the end at the owner’s request. Enrollment is not confirmed. No account/legal/payment action is automatic.

## Retained backlog

Per-project/brief agent grants; direct brief retrieval; note/brief FTS and database pagination (bounded UI pages implemented); tagging/branch views; encrypted access metadata and revoke-all; signed-app Keychain client integration; broader backup-filesystem support; richer supersession/conflict discovery; local-model execution with explicitly defined provider isolation. Private draft workspaces and revision history are implemented, not backlog items. These require explicit scheduling and must not silently expand a milestone.

No cloud sync, live capture, mandatory paid model dependency, semantic embeddings, mobile, teams, monetization or demand-testing work in this plan.

Keep the repository private and public download disabled until the owner explicitly chooses publication after the release gates pass.

Optional future mode: explicitly authorize a selected curator to process multiple chosen source archives, with distinct delegated attribution and independent publication. Default isolation stays unchanged. This is not implemented; see AGENT_CURATION. Gemini/Copilot format discovery is documented in PROVIDERS.
