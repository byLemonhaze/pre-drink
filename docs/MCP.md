# Agent access through MCP

Model Context Protocol is a standard way for an AI application to discover tools and request context from another program. Pre-Drink is the server; your assistant is the client. MCP is not a model, subscription, or cloud store. Cloud curation is optional: all manual local workflows work without a connected agent. Pre-Drink has no bundled local-model runner. [Official architecture](https://modelcontextprotocol.io/docs/learn/architecture).

## Connect

1. Put the app in a stable location before generating its configuration: the executable path is embedded.
2. Open and unlock the vault.
3. Review and approve eligible memories and briefs.
4. In **Agent access → Set up a connection**, choose the application and name. Shared memory only is the default. Explicitly choose history or curation if needed; Codex binds ChatGPT, Grok binds Grok and Claude binds Claude. Curators for the same provider share one workspace and can revise each other’s contributions. Acknowledge the scope and cloud disclosure.
5. Create private settings, save a new private configuration file (or explicitly reveal it), merge its Pre-Drink sections into that client's local configuration and restart/reload it. Save does not install or connect the client. Keep the file outside repositories, synced folders and chats.
6. Continue to Verify, explicitly enable sharing and keep Pre-Drink open/unlocked. Send the displayed permission-only `connection_status` instruction to the client, then choose **Check for client request**.
7. After verification, ask for a narrow context pack or small private curation batch within the authorized scope.

Verification retains only a set of client IDs whose authenticated successful response was delivered during the current unlock. Failed authentication, denied calls and oversized/error replies do not count. It stores no request text, returned content, timestamps or tokens, and clears on lock/unlock. The observation is not a live-connection monitor or model-identity proof. It can reflect any successful tool request from that credential, not exclusively the suggested probe.

The app generates TOML for Codex/Grok Build or common `mcpServers` JSON for other local clients. Every format carries the same absolute executable path, `args: ["--mcp"]`, and `PRE_DRINK_SOCKET`, `PRE_DRINK_CLIENT_ID`, `PRE_DRINK_CLIENT_TOKEN` environment entries. Tokens are never placed in command arguments. No vault password belongs in any client configuration. Creating settings does not turn sharing on or modify another application’s files.

Settings remain in one zeroizing backend buffer until explicit save/reveal, rather than being returned to the interface on registration. Save creates a new 0600 file, refuses an existing file/symlink and refuses detected Git-checkout destinations. It does not merge another app’s settings. Delivery rechecks the current unlock, setup identity and active registration while holding the session lock; a delayed Save panel cannot deliver after vault lock/expiry/replacement or revocation. Closing setup drops unsaved settings, without revoking the client. Saved files and explicitly revealed JavaScript text are not guaranteed to disappear from memory or other applications.

The stdio handshake, discovery and calls are integration tested. Codex CLI 0.169.1 has also passed eight fictional-data acceptance phases (ten actual MCP calls), including approval/source restrictions, memory-only scope, lock, unlock and persisted revocation. The app’s fictional Codex/Grok configuration screens have native evidence; this does not establish setup in every external desktop client. No client is automatically given access.

See [installed-client acceptance](MCP_ACCEPTANCE.md) for setup, per-client status and privacy/lifecycle evidence. Actual Claude client execution remains untested; generated Claude JSON registration is deterministically tested. Grok Build 1.0.41 also passed eight fictional-data phases and ten actual MCP calls after owner sign-in.

## Client configuration formats

Choose **Codex** to generate `[mcp_servers.pre-drink]` and its environment table. Add both sections to private `~/.codex/config.toml`, or `CODEX_HOME/config.toml` when using a custom home. Choose **Grok Build** for the equivalent sections in private `~/.grok/config.toml`, or `GROK_HOME/config.toml`. Keep unrelated settings; replace existing sections with the same server name rather than adding duplicate TOML tables. Restart the client to load the updated settings. Choose **Other client (JSON)** to merge a `pre-drink` entry into that client's existing `mcpServers` object.

Keep the settings file readable/writable only by your Mac account and outside project repositories. The token is shown on creation; hiding it does not revoke the client. If lost, revoke that registration and create a replacement. Do not put token values in shell arguments, prompts or repository configuration. The original acceptance runs used temporary per-invocation settings, not persistent owner configuration changes. See [Codex MCP configuration](https://developers.openai.com/codex/mcp).

The serializer preserves spaces, quotes, backslashes and Unicode in paths. Non-absolute paths, invalid Unicode and control characters fail without leaving a registration. Grok paths containing `${` are refused because the client interprets environment expressions in settings; use a stable location without that sequence. The TOML serializer was already present in the locked dependency graph and is now a direct dependency.

Grok Build supports command-based local stdio servers, as described in its [MCP guide](https://docs.x.ai/build/features/mcp-servers). This is distinct from [Grok web custom connectors](https://docs.x.ai/grok/connectors), which require a publicly reachable server. Pre-Drink does not expose a public endpoint or create tunnels. The Grok Build checks establish local stdio tool-result compatibility for the tested version; they do not establish Grok Bot/web compatibility. Client-specific configuration generation is implemented. Native configuration-screen checks are recorded in NATIVE_WALKTHROUGH; import support is separately defined and tested in PROVIDERS. Codex and Grok Build each passed three-session fictional writing/resume/retrieval acceptance; see CURATION_ACCEPTANCE for the measured scope and remaining quality gates.

## Tools

| Tool                                 | Permission              | Result                                                                                         |
| ------------------------------------ | ----------------------- | ---------------------------------------------------------------------------------------------- |
| `curate_memory(operation)`           | Matching curator scope  | Own-provider sources, private writes/revisions and resumable jobs; see action contract below   |
| `connection_status()`                | Any valid registration  | Permissions only: connected, provider, allow_history, can_curate; no archive or memory content |
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

**Share context** is an owner-only selection → full Markdown preview → copy/save workflow. Its saved-version and session checks apply at delivery. It does not issue a client credential, call a provider or narrow existing MCP access. Eight tools are available: the original six, scoped `curate_memory`, and permission-only `connection_status`; clients cannot invoke `prepare_share`, `copy_share` or `save_share`. Agent access's topic preview remains an illustration of a dynamic query, not an access-control boundary. See [SHARING](SHARING.md).

## Agent writing and curation

Select **Curate [provider] history and memory** in the guided setup for Codex, Grok or Claude. This grants its provider's safe source access and private working-memory writes; it does not turn sharing on or authorize automatic publication. Generic JSON registrations stay shared-only. Existing schema-3 broad chat grants are downgraded to shared-only on upgrade.

The scoped tool `curate_memory` takes an `operation` object containing a tagged action, for example `{"operation":{"action":"status"}}`:

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

## Owner holds and uninterrupted work

An owner-applied hold is independent of an agent's conflict flag. Agents may keep revising their own provider's draft but cannot clear the hold or release it. The owner clears it in **Edit / resolve**, then reviews the new saved version. Existing conflict flags from older vaults migrate conservatively to owner holds. The shared layer remains controlled by the owner; longer sessions do not broaden scopes or publication policy.

**Agent access → Let your agents work** offers one hour, four hours, or until Mac lock. Starting explicitly enables sharing and keeps the vault unlocked for that period, including while idle. A timed session locks at its deadline. Agents cannot renew it. Stopping sharing cancels it; manual lock, Mac lock and sleep still lock immediately, and unlocking/restarting never resumes it. The control does not launch a model or buy usage. External clients still have their own usage costs; this project's current policy prohibits further paid calls.

## Local account verification

Before sending a client token, the stdio bridge checks that the Unix socket peer belongs to its own effective OS account. The app independently checks the connecting peer before reading the request. The parent directory must be owned by that account and private; stale cleanup refuses links and ordinary files. These checks supplement scoped bearer tokens; they neither prove the model's identity nor protect against same-account/root malware. Signed-app credential integration remains future work. No MCP tool, provider grant or publication rule is expanded by this change.
