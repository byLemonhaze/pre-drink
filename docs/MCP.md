# Agent access through MCP

Model Context Protocol is a standard way for an AI application to discover tools and request context from another program. Pre-Drink is the server; your assistant is the client. MCP is not a model, subscription, or cloud store. [Official architecture](https://modelcontextprotocol.io/docs/learn/architecture).

## Connect

1. Put the app in a stable location before generating its configuration: the executable path is embedded.
2. Open and unlock the vault.
3. Review and approve eligible memories and briefs.
4. In **Agent access**, name a client, leave chat access off unless needed, and acknowledge the provider-processing disclosure.
5. Save the generated configuration in that agent's **local** MCP settings. Its secret token is shown once. Never commit it or paste it into a conversation.
6. Enable sharing and keep Pre-Drink open/unlocked.
7. Ask the agent to use `get_context_pack` with a narrow topic.

The generated configuration uses common `mcpServers` JSON: absolute executable path, `args: ["--mcp"]`, and `PRE_DRINK_SOCKET`, `PRE_DRINK_CLIENT_ID`, `PRE_DRINK_CLIENT_TOKEN` environment entries. Other client formats need those values translated to their own local settings. No vault password belongs in any client configuration.

The stdio handshake, discovery and calls are integration tested. Individual third-party client setup screens have not yet been verified here. No client is automatically given access.

## Tools

| Tool                                 | Permission             | Result                                                                                         |
| ------------------------------------ | ---------------------- | ---------------------------------------------------------------------------------------------- |
| `get_facts()`                        | Approved memory        | Up to 50 recently updated eligible facts                                                       |
| `get_decisions()`                    | Approved memory        | Up to 50 eligible decisions                                                                    |
| `get_projects()`                     | Approved memory        | Up to 50 eligible project notes                                                                |
| `get_context_pack(topic)`            | Approved memory/briefs | Up to 3 briefs and 30 notes in bounded Markdown; avoids repeating notes represented by a brief |
| `search_conversations(query, limit)` | Additional chat scope  | Up to 50 safe conversation matches with previews                                               |
| `mark_private(conversation_id)`      | Additional chat scope  | Makes a conversation private; cannot undo privacy                                              |

Memory list responses contain only `id`, `kind`, `content` and `updated_at`. Source conversation titles and IDs are not included. Memory-only context packs omit source-message IDs.

Chat-enabled context packs can add up to five safe excerpts. Local preview shows memory-only scope. Memory/brief topic matching is literal case-insensitive substring matching, not semantic search. Chat search uses literal FTS terms. A topic is not an authorization boundary. All tools enforce privacy and source eligibility; approval does not bypass those filters.

## Control and cost

Sharing off, locking, or quitting stops future calls. Unlocking does not re-enable sharing. Revoke credentials individually; change scope or replace lost credentials by revoking and creating a new client. Returned content cannot be recalled.

The local server makes no AI calls and has no per-use charge. Your assistant may consume normal subscription allowances or API tokens when processing context. Cloud assistants may transmit it to their providers. For shared context to remain entirely local, use a local client/model.

## Troubleshooting

- Locked/sharing off: unlock in the app and enable sharing.
- Denied/revoked: create a new configuration; never insert the vault password.
- Memory-only restriction: retain it or explicitly authorize a separate chat-enabled client.
- No matches: check approval, privacy, source eligibility and literal topic text.
- Connection unavailable: GUI must run and executable/socket paths must match. Regenerate after moving the app.
- Result too large: narrow the topic. List tools do not yet paginate.

## Selected exports versus live grants

**Share context** is an owner-only selection → full Markdown preview → copy/save workflow. Its saved-version and session checks apply at delivery. It does not issue a client credential, call a provider or narrow existing MCP access. The six MCP tools remain unchanged; clients cannot invoke `prepare_share`, `copy_share` or `save_share`. Agent access's topic preview remains an illustration of a dynamic query, not an access-control boundary. See [SHARING](SHARING.md).
