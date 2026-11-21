# Installed-client acceptance

This document separates the acceptance procedure from results for each client. Automated stdio checks do not by themselves establish third-party client compatibility. Native setup screens and native lifecycle events need evidence beyond the backend fixture.

## Client results (2026-11-20)

| Client            | Result   | Scope and remaining limits                                                                                                                                                                                   |
| ----------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Codex CLI 0.169.1 | Passed   | Eight phases, ten real MCP calls through the production stdio bridge and fictional encrypted fixture; actual tool events independently checked. Desktop configuration UI and native app restart remain open. |
| Grok Build 1.0.41 | Passed   | Eight phases, ten real MCP calls across three disposable fixtures after owner sign-in. Four built-in tool-discovery calls also ran. Native setup/restart and Grok web/Bot remain unverified.                 |
| Claude Code       | Deferred | Owner chose Codex/Grok instead. No Claude model calls made.                                                                                                                                                  |

The Codex phases were sharing-off denial; approved facts and brief retrieval plus memory-only chat denial; approval invalidation on edit; source privacy exclusion; lock with a still-valid credential; unlock retaining sharing-off; revocation with sharing enabled; and backend-session replacement retaining revocation. Draft/private/sensitive fictional markers never appeared in successful facts/context responses. All ten completed tool events belonged to Pre-Drink; no other tools executed.

Codex used existing ChatGPT authentication, no API key, an empty working folder, `--ignore-user-config`, `--ephemeral`, read-only command restrictions, shell/apps/multi-agent tools disabled and a three-tool Pre-Drink allowlist. Runtime `env_vars` forwarded the temporary credentials without placing their values in process arguments. These were per-invocation settings, not persistent owner configuration changes.

Grok passed the same eight policy phases using the production bridge: sharing off; approved facts/context plus memory-only chat denial; edit invalidation; private-source exclusion; lock; unlock retaining sharing-off; revocation; and revocation after backend-session replacement. Assertions inspected actual ACP tool results, not the model's summary. All ten MCP calls targeted the three requested Pre-Drink tools; the only other executed tool was Grok's built-in discovery (four calls). No shell, file, web, unrelated connector or subagent call executed.

The Grok runs used existing account authentication, an empty working folder, isolated temporary configuration, disabled compatibility connectors and no API key. Credentials traveled through the local ACP session setup; no persistent MCP registration was added. Fictional vault/configuration files were removed after revocation and lock. Cloud-client history can retain content it already received; these checks used invented content only.

The earlier authentication block was resolved when the owner completed `grok login --oauth`. Initialization then advertised `cached_token` and `grok.com`, and cached authentication succeeded. The [Grok ACP guide](https://docs.x.ai/build/cli/headless-scripting) describes that flow; inspect the advertised methods for the installed client. One local test-driver attempt redundantly unlocked an already-unlocked restarted fixture and stopped before making its final MCP call. Correcting that sequence and rerunning the single restart case passed; no product change was required.

## Generated settings acceptance

The app now generates Codex/Grok Build TOML and generic JSON directly. Unit tests parse each format and compare exact executable/socket paths (including quotes, spaces, backslashes and Unicode), argument separation and credential values. Issued credentials obey memory-only versus chat scope and revocation in every format; unsupported paths roll back registration.

Read-only checks against installed Codex CLI 0.169.1 loaded generated TOML and confirmed the exact command, arguments and environment. Grok Build 1.0.41 discovered its generated stdio entry and exact executable path; its inspection output does not expose all environment values, so those are validated by the parser tests. Both checks used fresh temporary homes and an empty working directory; no model call or persistent owner setting changed. This does not close native connection-screen verification.

## Reproducible backend fixture

When desktop automation is unavailable, a developer-only example can host the actual encrypted vault, broker and policy code with invented content:

```sh
cargo build --manifest-path src-tauri/Cargo.toml --locked --example mcp_acceptance_fixture --bin pre-drink
./src-tauri/target/debug/examples/mcp_acceptance_fixture ./src-tauri/target/debug/pre-drink
```

It creates fresh temporary storage and a random password that is never printed. It accepts no existing vault or export. Its first output gives a configuration-file path and an empty client working folder; the folder is owner-only (0700) and configuration file owner read/write (0600). Start the client in that working folder using the file path. Do not put the generated file in Git or display its token.

The fixture terminal accepts `sharing_on`, `sharing_off`, `edit`, `restrict`, `lock`, `unlock`, `revoke`, `restart` and `quit`. These are developer-process stdin commands, not MCP tools or app features. `edit` changes the approved standalone fact; `restrict` marks the invented source conversation private. `restart` replaces/reopens the backend session, not the native app or operating system. `quit` revokes access, locks and removes this fixture's temporary files on normal exit. Abrupt termination can leave temporary files; filesystem deletion is not secure erasure.

This harness bypasses UI setup and therefore cannot verify the app's client-registration screens, clipboard configuration flow or native lifecycle events. Results apply only to the actual client tested; see the status table above. Review any cloud-client cost before invocation; no model calls are part of starting the fixture.

## Safe setup

- Use a debug build with separate disposable fictional storage. Confirm **Separate test storage** before unlocking. Never connect a personal vault for this check.
- Create fictional notes containing distinct markers: `garden-approved`, `garden-draft` and `garden-private`. Approve only the first; leave the second unapproved and the third private. Create and approve a fictional brief containing `garden-brief`.
- Register a dedicated memory-only client through **Agent access** after reviewing the disclosure. Leave chat access off. Sharing starts off.
- Save the generated JSON in a private local folder outside every repository. Restrict the folder to the owner and the file to owner read/write. The token is a credential: never paste it into a prompt, command-line argument, screenshot, report or Git. Do not use a project `.mcp.json`.
- Use a separate empty working folder for the client. Do not load the product source tree, personal files or existing agent sessions. A cloud-backed client may send the fictional tool results to its provider and consume its normal allowance.

## Claude Code launch

The installed Claude Code 2.1.112 help was inspected for these flags. This verifies available syntax only; it does not verify a connection or successful tool use.

Reference: [Claude Code MCP documentation](https://code.claude.com/docs/en/mcp). Recheck the installed CLI help if its version changes.

From the empty working folder, launch an interactive session with the configuration file's absolute path:

```sh
claude --mcp-config /absolute/local/path/pre-drink-client.json \
  --strict-mcp-config --tools "" --disable-slash-commands --no-chrome
```

Replace the example path locally; it is not a real configuration. `--strict-mcp-config` limits MCP servers to the supplied configuration. `--tools ""` disables built-in tools; it does not narrow the Pre-Drink credential's permissions. Keep ordinary permission prompts enabled. Do not use a permission-bypass mode. Review local client settings/hooks before launch; these flags alone are not a general-purpose isolation boundary.

Do not place the JSON itself in `--mcp-config`: use its file path to keep the token out of process arguments. No persistent client registration in Claude Code is required for this procedure.

## Acceptance matrix

Use the actual tool result as evidence, not just the model's paraphrase. Ask the client to call the named tool and report the returned result without filling gaps from prior messages. Discovery may succeed while the vault is locked: authorization is enforced when a tool is called.

| Check              | Action                                                                                          | Required result                                             |
| ------------------ | ----------------------------------------------------------------------------------------------- | ----------------------------------------------------------- |
| Discovery          | Inspect the connected server's tools                                                            | Six documented Pre-Drink tools; no vault password requested |
| Sharing off        | Call `get_facts` before enabling sharing                                                        | Tool error; no note content                                 |
| Approved content   | Enable sharing, then call `get_facts` and `get_context_pack` with topic `garden`                | Approved note/brief present; draft/private markers absent   |
| Memory-only scope  | Call `search_conversations` with query `garden`                                                 | Permission denial, even though discovery lists the tool     |
| Edit invalidation  | Edit the approved note in Pre-Drink without approving its new version; call `get_facts` again   | Edited note absent until separately reviewed and approved   |
| Source restriction | With a separate approved source-derived note, mark its source conversation private; query again | Derived note excluded                                       |
| Lock               | Lock Pre-Drink, then call `get_facts`                                                           | Tool error; no content                                      |
| Unlock default     | Unlock, then call again without enabling sharing                                                | Still denied                                                |
| Revocation         | Enable sharing, revoke this test client, then call again                                        | Denied with the old configuration                           |
| Restart            | Quit/reopen/unlock Pre-Drink                                                                    | Sharing remains off; revoked credential remains invalid     |

Changes only stop future retrieval. A client's earlier responses or chat history may still contain previously returned fictional content. Seeing that old text in the client is not evidence that a new request succeeded.

## Close and record

Revoke the dedicated test credential, disable sharing, lock the fictional vault and exit the client. Remove its temporary configuration according to the local test-data cleanup policy; revocation is the access-control step. Do not change or remove existing client configurations.

Record the source commit, native build identity, macOS/client versions, each pass/fail and any sanitized error. Keep raw client transcripts, configuration, local paths and screenshots out of Git. A discovered server or successful handshake alone does not close this matrix. Report unresolved cases explicitly.

## Provider curation acceptance status

The historical model-client retrieval runs above predate agent writing. The current compiled stdio suite additionally exercises the eight-tool handshake and permission-only connection probe, `curate_memory` status/write, identical retry receipts, owner release, attributed shared retrieval and rejection after lock. Backend regressions cover provider separation, evidence, revision conflicts, incremental manifests and lifecycle resets; the native fictional walkthrough covers owner review/edit/history/withdrawal. See VALIDATION for exact counts.

Codex CLI 0.171.0-alpha.13 now passed three fresh ChatGPT-authenticated sessions and 27 actual MCP calls covering session/archive writes, retries, backend restart/job resume, reviewed release and shared-only retrieval. Cross-provider raw reads and shared-reader curation were denied. No-model follow-up probes verified exclusion after edit/source restriction and on lock. The complete procedure, semantic observations and limits are in [CURATION_ACCEPTANCE](CURATION_ACCEPTANCE.md). This supersedes the earlier writing-only-unverified status for this bounded Codex fixture, not the historical client/version results above.

Grok Build 1.0.41 also passed three fresh provider-matched sessions with 29 actual MCP calls and three tool-discovery calls. It resumed the same job, retained evidence on revision, kept drafts private, and retrieved the two reviewed notes with a shared-only credential. Foreign ChatGPT/Claude source reads and shared-reader curation were denied. Edit, source-privacy and lock probes passed locally. See CURATION_ACCEPTANCE for the lexical context-pack limitation and exact boundaries.

Claude model-led writing, broad curation quality and representative official exports remain separate gates. This does not connect this chat or an arbitrary web session to the app. Manual offline use does not depend on any cloud acceptance run.
