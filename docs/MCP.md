# Agent access through MCP

Model Context Protocol is a standard way for an AI application to discover tools and request context from another program. Pre-Drink is the server; your assistant is the client. MCP is not a model, subscription, or cloud store. Cloud curation is optional: all manual local workflows work without a connected agent. Pre-Drink has no bundled local-model runner. [Official architecture](https://modelcontextprotocol.io/docs/learn/architecture).

## Connect

1. Put the app in a stable location before generating its configuration: the executable path is embedded.
2. Open and unlock the vault.
3. Review and approve eligible memories and briefs.
4. In **Agent access**, choose Codex, Grok Build, Claude (local MCP), or Other client (JSON), name the client, leave source/curation access off unless needed; selecting Codex binds ChatGPT, Grok binds Grok and Claude binds Claude, and acknowledge the provider-processing disclosure.
5. Save the generated configuration in that agent's **local** MCP settings. Its secret token is shown once. Never commit it or paste it into a conversation.
6. Enable sharing and keep Pre-Drink open/unlocked.
7. Ask the agent to use `get_context_pack` with a narrow topic.

The app generates TOML for Codex/Grok Build or common `mcpServers` JSON for other local clients. Every format carries the same absolute executable path, `args: ["--mcp"]`, and `PRE_DRINK_SOCKET`, `PRE_DRINK_CLIENT_ID`, `PRE_DRINK_CLIENT_TOKEN` environment entries. Tokens are never placed in command arguments. No vault password belongs in any client configuration. Creating settings does not turn sharing on or modify another application’s files.

The stdio handshake, discovery and calls are integration tested. Codex CLI 0.169.1 has also passed eight fictional-data acceptance phases (ten actual MCP calls), including approval/source restrictions, memory-only scope, lock, unlock and persisted revocation. The app’s fictional Codex/Grok configuration screens have native evidence; this does not establish setup in every external desktop client. No client is automatically given access.

See [installed-client acceptance](MCP_ACCEPTANCE.md) for setup, per-client status and privacy/lifecycle evidence. Actual Claude client execution remains untested; generated Claude JSON registration is deterministically tested. Grok Build 1.0.41 also passed eight fictional-data phases and ten actual MCP calls after owner sign-in.

## Client configuration formats

Choose **Codex** to generate `[mcp_servers.pre-drink]` and its environment table. Add both sections to private `~/.codex/config.toml`, or `CODEX_HOME/config.toml` when using a custom home. Choose **Grok Build** for the equivalent sections in private `~/.grok/config.toml`, or `GROK_HOME/config.toml`. Keep unrelated settings; replace existing sections with the same server name rather than adding duplicate TOML tables. Restart the client to load the updated settings. Choose **Other client (JSON)** to merge a `pre-drink` entry into that client's existing `mcpServers` object.

Keep the settings file readable/writable only by your Mac account and outside project repositories. The token is shown on creation; hiding it does not revoke the client. If lost, revoke that registration and create a replacement. Do not put token values in shell arguments, prompts or repository configuration. The original acceptance runs used temporary per-invocation settings, not persistent owner configuration changes. See [Codex MCP configuration](https://developers.openai.com/codex/mcp).

The serializer preserves spaces, quotes, backslashes and Unicode in paths. Non-absolute paths, invalid Unicode and control characters fail without leaving a registration. Grok paths containing `${` are refused because the client interprets environment expressions in settings; use a stable location without that sequence. The TOML serializer was already present in the locked dependency graph and is now a direct dependency.

Grok Build supports command-based local stdio servers, as described in its [MCP guide](https://docs.x.ai/build/features/mcp-servers). This is distinct from [Grok web custom connectors](https://docs.x.ai/grok/connectors), which require a publicly reachable server. Pre-Drink does not expose a public endpoint or create tunnels. The Grok Build checks establish local stdio tool-result compatibility for the tested version; they do not establish Grok Bot/web compatibility. Client-specific configuration generation is implemented. Native configuration-screen checks are recorded in NATIVE_WALKTHROUGH; import support is separately defined and tested in PROVIDERS. Codex's new three-session fictional writing acceptance is recorded in CURATION_ACCEPTANCE; Grok Build model-led writing remains open.

## Tools

| Tool                                 | Permission              | Result                                                                                         |
| ------------------------------------ | ----------------------- | ---------------------------------------------------------------------------------------------- |
| `get_facts()`                        | Approved memory         | Up to 50 recently updated eligible facts                                                       |
| `get_decisions()`                    | Approved memory         | Up to 50 eligible decisions                                                                    |
| `get_projects()`                     | Approved memory         | Up to 50 eligible project notes                                                                |
| `get_context_pack(topic)`            | Approved memory/briefs  | Up to 3 briefs and 30 notes in bounded Markdown; avoids repeating notes represented by a brief |
| `search_conversations(query, limit)` | Matching provider scope | Up to 50 safe conversation matches with previews                                               |
| `mark_private(conversation_id)`      | Matching provider scope | Makes a conversation private; cannot undo privacy                                              |

Memory list responses contain `id`, `kind`, `content` and `updated_at`. Released agent contributions also carry `curator_provider`, `curator_revision`, `origin` and `publication_basis`. Raw source IDs/titles and private session labels are excluded.

All MCP context packs now contain shared memory/briefs only. Provider-scoped search and curation are the exclusive raw-history routes. Local topic preview uses the same shared-only pack. Memory/brief topic matching is literal case-insensitive substring matching, not semantic search. Chat search uses literal FTS terms. A topic is not an authorization boundary. All tools enforce privacy and source eligibility; approval does not bypass those filters.

## Control and cost

Sharing off, locking, or quitting stops future calls. Unlocking does not re-enable sharing. Revoke credentials individually; change scope or replace lost credentials by revoking and creating a new client. Returned content cannot be recalled.

The local server makes no AI calls and has no per-use charge. Your assistant may consume normal subscription allowances or API tokens when processing context. Cloud assistants may transmit it to their providers. For shared context to remain entirely local, use a local client/model.

## Troubleshooting

- Locked/sharing off: unlock in the app and enable sharing.
- Denied/revoked: create a new configuration; never insert the vault password.
- Memory-only restriction: retain it or explicitly authorize a separate chat-enabled client.
- No matches: check approval, privacy, source eligibility and literal topic text.
- Connection unavailable: GUI must run and executable/socket paths must match. Regenerate after moving the app.
- Result too large: narrow the topic. The shared fact/decision/project lists do not paginate; private source/note/job-source actions have their documented continuations.

## Selected exports versus live grants

**Share context** is an owner-only selection → full Markdown preview → copy/save workflow. Its saved-version and session checks apply at delivery. It does not issue a client credential, call a provider or narrow existing MCP access. The original six tools plus the scoped `curate_memory` tool are available; clients cannot invoke `prepare_share`, `copy_share` or `save_share`. Agent access's topic preview remains an illustration of a dynamic query, not an access-control boundary. See [SHARING](SHARING.md).

## Agent writing and curation

Enable the curation checkbox when registering Codex, Grok or Claude. This grants its provider's safe source access and private working-memory writes; it does not turn sharing on or authorize automatic publication. Generic JSON registrations stay shared-only. Existing schema-3 broad chat grants are downgraded to shared-only on upgrade.

The seventh tool, `curate_memory`, takes an `operation` object containing a tagged action, for example `{"operation":{"action":"status"}}`:

| Action         | Purpose                                                                                  |
| -------------- | ---------------------------------------------------------------------------------------- |
| `status`       | Bound provider and current owner policy                                                  |
| `list_sources` | Safe own-provider source metadata; `after` ID and up to 50 records                       |
| `read_source`  | Own-provider message in 4,000-character chunks, evidence version and continuation offset |
| `list_notes`   | Own eligible private notes, bounded response and continuation ID                         |
| `write`        | Create or revise a contribution with retry key and expected revision                     |
| `history`      | Up to 20 recent eligible revisions, within the response budget                           |
| `undo`         | Copy prior text to a new held revision, preserving all evidence                          |
| `jobs`         | Latest 30 own-provider job checkpoints                                                   |
| `job_sources`  | Stable job manifest, 30 entries after an ordinal; changed/restricted entries withheld    |
| `save_job`     | Retry-safe creation or revision-checked checkpoint/status update                         |

For session handoff, call `status`, inspect existing own notes to avoid semantic duplicates, and call `write`. New input has `id: null`, `expected_revision: 0`, a unique `request_key`, `kind` (`fact`, `decision`, `project`), `content`, `origin: "session"`, a nonempty `session_label`, `sources: []`, and a boolean `conflict`. Updates supply the saved ID, current revision, same origin and every retained source reference. A session label is an agent assertion, not verified imported evidence.

For bulk history, create a job using `save_job` with `id: null`, `expected_revision: 0`, `checkpoint: 0`, short non-sensitive `label`, `status: "running"` and `request_key`. Read its `job_sources`, then each available `read_source`, checking versions. Store durable high-signal notes using `origin: "archive"` and `sources: [{message_id, version}]`. Save/retry contributions before advancing the completed-source ordinal in `checkpoint`. Pause/resume with expected job revisions. Completing requires reaching the manifest length. New jobs include new/changed/unacknowledged evidence; no background inference or automatic spending occurs.

Use **Agent access → Agent curation** to review/edit/withdraw contributions, inspect history and progress, or release up to 30 selected notes after an exact preview. Automatic sharing is separately opt-in for future archive/session projects and decisions; facts, links and instruction-like content wait for review. Sensitive, stale-source and unresolved-conflict content stays withheld. Auto-released content is explicitly labelled, never presented as individually owner-reviewed.

All operations use the existing sharing/lock/revocation boundary. Client names and claimed model identities cannot broaden a credential. Do not reuse provider credentials across models. See [AGENT_CURATION](AGENT_CURATION.md) for the full contract and limitations.

## Claude and future delegates

Choose **Claude (local MCP)** for `mcpServers` JSON and a Claude-bound grant. Merge the entry into a compatible local Claude client’s private settings; never upload it to a chat or repository. This does not create web connectivity or establish tested Claude model execution. Its source/write scope is Claude only; shared summaries are available under the same delivery rules.

An optional cross-provider delegate is planned, not implemented. No current format, client name or `curate_memory` argument grants it. The owner must separately select source archives and acknowledge the model destination when that feature exists. Cloud curation is always optional.
