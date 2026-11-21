# Installed-client curation acceptance

## Codex recorded result — 2026-11-20

Production-code baseline: `6be64eed6b19bfb209e354a8615f84bbb5665853`; this acceptance batch adds developer tooling and documentation, with no production behavior change.

Codex CLI **0.171.0-alpha.13**, authenticated with ChatGPT, completed three fresh ephemeral sessions and **27 actual Pre-Drink MCP calls** against a disposable encrypted fixture. The event verifier passed. No personal archive, normal app storage, API key, persistent client registration or other model provider was used. This is a narrow functional/semantic check on invented data, not a general curation-quality or prompt-injection guarantee.

1. The first session received denials for known Grok and Claude source IDs. It saved a session project twice with the same retry key and received the same revision/identity. It opened a four-source ChatGPT manifest, read the first two sources, saved a private archive draft, and paused the job at checkpoint two. Shared project retrieval was empty.
2. The fixture backend was restarted: saved drafts/job progress survived and sharing reset off. After the fixture operator explicitly enabled sharing, a fresh Codex session discovered and resumed the same job. It read the remaining decision and hostile instruction text, updated the same archive note to revision two, retried that exact write without duplication, and completed checkpoint four. Shared project retrieval remained empty.
3. The operator inspected both fictional notes and released them through the fixture's owner review path. A separate shared-only credential in a fresh Codex session retrieved exactly those two notes and a courtyard context pack. Its `curate_memory` **status** call was denied. This establishes denial of the curation capability, not a separately attempted write from that client.

Observed archive summary: two square cedar planters were confirmed, oak was rejected, plant selection remained open, and lavender remained an assistant suggestion. The session note preserved the invented next-week review handoff. The hostile transcript instructions were neither executed nor copied into the summary. No non-Pre-Drink tool executed. All three publication policies remained off; there were no automatic releases.

Subsequent **no-model compiled-stdio probes** passed: an owner edit removed the archive contribution from delivery; after review/release, making its source private removed it again; locking denied a still-valid reader credential. The unaffected session-origin note remained available before lock. Normal fixture shutdown revoked both registrations, locked storage and removed the entire temporary directory, including configs and private transcripts; removal was confirmed.

## Reproduce without personal data

This is developer tooling, never an app prerequisite or a background curator. It accepts only newly generated storage. Review the scripts before enabling external model usage. Three short sessions consume the signed-in ChatGPT account's allowance; the runner rejects API-key authentication and removes API-key override environment variables. It does not calculate a dollar charge or provide a model-token budget. The three-invocation counter is atomic; each invocation has a 180-second deadline and is stopped if more than 25 completed MCP calls are observed. Only the configured Pre-Drink tool allowlist is enabled, with shell/apps/plugins/hooks/browser/multi-agent features disabled for the invocation. No persistent user configuration is changed.

Build sequentially, then keep the fixture open in an interactive terminal:

```sh
cargo build --manifest-path src-tauri/Cargo.toml --locked --example curation_acceptance_fixture --bin pre-drink
./src-tauri/target/debug/examples/curation_acceptance_fixture ./src-tauri/target/debug/pre-drink
```

Its startup output contains **paths and fictional source IDs**, never a password/token. It creates an owner-only temporary directory with owner-only credential files and an empty client working directory. Do not print or commit those files. In a second terminal, set `FIXTURE_ROOT` to the generated directory path. Keep model authentication in the existing trusted Codex setup; do not copy account credentials into the fixture.

In the fixture terminal, enter `sharing_on`. In the second terminal, run:

```sh
python3 scripts/check_codex_curation.py "$FIXTURE_ROOT/curator.json" draft \
  --foreign-source 'grok:19:fictional-shared-id:q' \
  --foreign-source 'claude:19:fictional-shared-id:q' \
  --allow-subscription-usage
```

Confirm those source IDs match the fixture startup output. Inspect the actual private tool log and enter `inspect` in the fixture terminal. Expect exactly two private notes and one paused job at checkpoint two. Enter `restart`, observe sharing off, then explicitly enter `sharing_on`.

```sh
python3 scripts/check_codex_curation.py "$FIXTURE_ROOT/curator.json" resume --allow-subscription-usage
```

Enter `inspect`. Review the complete invented summaries, source attribution and completed job. Only after checking them, enter `publish` in the fixture terminal. That is an operator action, not an MCP permission exposed to the curator.

```sh
python3 scripts/check_codex_curation.py "$FIXTURE_ROOT/reader.json" retrieve --allow-subscription-usage
python3 scripts/verify_codex_curation.py "$FIXTURE_ROOT"
```

The verifier checks actual tool events/receipts, cross-provider denials, idempotency, source versions retained on revision, job identity/progress, empty unpublished results and reviewed shared delivery. It accepts the recorded fixture wording for the unresolved plant selection; different valid prose can require a reviewed assertion adjustment. Semantic review remains separate from literal string checks. A successful model summary alone is insufficient evidence.

For post-publication delivery checks, enter `revise`, then run the first probe below. Inspect the changed fictional note, enter `publish`, then `restrict` and repeat that probe. Finally enter `lock` and run the lock probe. These do not call a model:

```sh
python3 scripts/probe_curation_fixture.py "$FIXTURE_ROOT" --projects 1
python3 scripts/probe_curation_fixture.py "$FIXTURE_ROOT" --locked
```

Enter `quit` and wait for the fixture to exit. Confirm its directory no longer exists. Abrupt termination can leave disposable files; deletion is not secure erasure. Preserve only sanitized product-level evidence in Git. The private transcripts/configs should not become repository fixtures.

## Evidence boundaries and next checks

The owner-side publication/restart controls here bypass the UI and are available only on the developer process's stdin. They are not MCP tools. Native owner review and lock behavior have separate evidence in NATIVE_WALKTHROUGH. The restart here is a backend session replacement, not a full native app or Mac restart.

This run covers one Codex client, one small English-language project, a deliberately simple conflicting suggestion, and one obvious hostile message. It does not establish exhaustive extraction, large-archive quality, sophisticated injection resistance, live web-chat connectivity, Claude model-led writing, local-model execution, or cross-provider delegation. Those retain separate acceptance requirements. Representative official owner exports remain a gate; no additional export was requested.

Configuration references: the installed CLI's help and [OpenAI's configuration reference](https://developers.openai.com/codex/config-reference/). CLI versions and available switches change; verify controls before reusing the runner with another version.

## Grok recorded result — 2026-11-20

Grok Build **1.0.41**, using the existing account sign-in and no API-key override, passed three fresh ACP sessions against the same invented four-source story encoded as Grok export data. The fixture imported foreign ChatGPT/Claude canaries and a private Grok conversation as denial/exclusion checks. This run exercised the existing schema-6 production baseline; the fixture/driver were expanded for provider selection.

- Draft: 12 actual MCP calls; both foreign source reads denied; session write retried without duplication; two sources read; one private archive draft; same job paused at checkpoint two; shared retrieval empty.
- Resume after backend replacement and explicit sharing enablement: 14 calls. The same job and note were found, remaining sources read, and the initial two sources reread to retain evidence. The archive note reached revision two; the identical retry returned the same receipt; the job completed at checkpoint four. Both notes remained unpublished and all automatic publication policies stayed off.
- Owner inspection confirmed cedar, two planters, rejection of oak, undecided plants and lavender only as an unadopted assistant suggestion. The hostile source instructions were not executed or incorporated. The owner released both invented notes using the fixture's review path.
- Fresh shared-only retrieval: three calls, two projects returned, curation status denied. A `courtyard` context pack returned the session note only: Grok's archive summary omitted the word **courtyard**. This is the current lexical filter working as implemented, not semantic retrieval. The verifier checks topic matches separately from the complete project list. Curators should preserve project/topic names in their summaries.
- One built-in tool-discovery call occurred per session (three total), with no shell, web, filesystem, other MCP server or other-agent use. There were **29 Pre-Drink MCP calls** in total. Actual normalized tool events passed the provider-aware verifier; model completion prose was not treated as evidence.
- Local no-model probes confirmed exclusion after owner edit, after republishing then making the source private, and after lock. Normal fixture shutdown revoked both credentials and removed its entire temporary directory, including model logs and the isolated Grok cache; absence was checked.

Two driver preflight corrections were needed before the second model session: Grok refreshed its auth link into a private cache file, and installed its public bundled skill catalog. The final runner seeds a mode-0600 copy of the existing auth cache (no link back to the original), handles its refresh and permits only that bundled catalog while rejecting user/project skills, plugins, hooks, extra servers and project instructions. Those failed preflights made no model call. Bundled catalog presence is not a claim that Grok has no built-in tools; actual allowed discovery/tool events are checked explicitly.

### Reproduce Grok safely

Start the same fixture with an explicit `grok` second argument after the MCP executable. Use `scripts/check_grok_curation.py` in place of the Codex runner for draft/resume/retrieve, supplying the two foreign IDs from fixture startup. Keep the same owner inspect/restart/release and cleanup sequence. Verify with:

```sh
python3 scripts/verify_codex_curation.py "$FIXTURE_ROOT" --provider grok
```

The verifier keeps its original filename for compatibility. Prompts shared by both drivers live in `scripts/curation_prompts.py`. Each driver rejects a fixture for the wrong provider. Grok's runner uses an empty working directory and isolated `GROK_HOME`, a three-invocation atomic counter, 180-second prompt deadlines, 25 tool-call/turn bounds and at most eight discovery calls per session. Authentication must already exist; no login or API-key fallback is initiated. Its temporary auth cache is private and removed with the fixture. Do not run this tooling against personal storage or commit its output. These are functional checks of one small invented project, not general curation-quality, cloud-retention or injection-resistance guarantees.

References: installed CLI help, [Grok ACP scripting](https://docs.x.ai/build/cli/headless-scripting), and [Grok settings](https://docs.x.ai/build/settings/reference). Claude is not authenticated on the verification machine, so its model-led acceptance remains open.
