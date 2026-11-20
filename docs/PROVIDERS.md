# Official export providers and compatibility

The importer reads recognized JSON entries directly from a ZIP. It never extracts paths, scrapes a live service, uses browser credentials or modifies the original. One ZIP is parsed before its database transaction; malformed records, duplicate identities and cycles reject that ZIP. Successful earlier ZIPs in a multi-file batch remain if a later ZIP fails.

## Current adapters

| Provider | Recognized entry                                                       | Accepted conversation contract                                                                                  | Verification                                                                                                           |
| -------- | ---------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| ChatGPT  | `conversations.json` or `conversations-N.json`, including nested paths | Array with IDs and `mapping` nodes                                                                              | Versioned fictional fixtures, native picker/drop/reimport; representative owner export pending                         |
| Claude   | Same conversation filenames, detected by record structure              | Array with `uuid`, `name`, `chat_messages`; message UUID/sender and text blocks or legacy text                  | Versioned fictional fixtures, native picker/reimport/search and scope regressions; representative owner export pending |
| Grok     | `prod-grok-backend.json`, including nested account-export paths        | Object with `conversations`; each has `conversation` metadata and `responses`, wrapped or flat response records | Versioned fictional fixtures, native picker/reimport/search and scope regressions; representative owner export pending |

These are observed internal contracts, not official provider schema versions or a promise to accept every historical/future layout. Empty conversation arrays are valid. Unknown/ambiguous structures fail with an error; a filename alone is not trusted as a provider label. Claude is recognized by `chat_messages`, ChatGPT by `mapping`; a record containing both is rejected. Grok requires the account-export envelope. Live-capture JSON and loose JSON uploads are outside this importer.

The same 256 MiB cumulative uncompressed recognized-JSON limit and 100,000-entry ZIP limit apply to all providers. Declared and actually decompressed sizes are checked. Nested paths are used for matching only; traversal-like entry names cannot write files. Other account-export files are not imported. No compressed nested ZIP recursion is supported.

## What is preserved

Provider-qualified conversation/message identities prevent collisions even when two products use the same source ID. Conversation JSON is retained intact as a JSON value, including unknown fields, while text normalization feeds search and the reader. New imports retain previous unique snapshots and missing older message branches, preserve manual privacy and invalidate changed source evidence. The parser sets the stored provider explicitly; every curation route uses that stored origin.

Claude imports `human` and `assistant` text. A nonempty content-block array contributes only `type: text` blocks; thinking/tools/images do not become reader text. Legacy `text` is used when blocks are absent, null or empty. Grok imports `human` and case-insensitive `assistant` senders and their `message` string; unknown senders are omitted. Grok `_id` or `response_id` and Claude `uuid` are required; duplicate/missing response IDs fail. Original omitted fields remain inside the encrypted conversation snapshot, not in shared context.

ISO-8601 offsets normalize to Unix seconds. Grok Mongo extended-JSON dates (`$date` / `$numberLong` milliseconds, numeric milliseconds or ISO dates) are supported. Missing/unrecognized dates remain absent, never replaced with the current time. No attachment/media download, reasoning rendering or citation fetching occurs.

Available parent links are preserved and checked for cycles. Without any parent links, the export’s supplied array order becomes a linear conversation. When a graph has no provider-confirmed current leaf, the latest timestamp/ID leaf is an explicitly warned display fallback. It is not proof of which branch the user last selected. Alternate branches remain accessible in the owner reader and subject to privacy checks.

## Requesting original exports

- ChatGPT: use the provider’s account data export, then keep the downloaded ZIP intact. The already requested owner export remains pending; do not submit duplicate requests as part of verification.
- Claude: [Anthropic’s export instructions](https://support.claude.com/en/articles/9450526-export-your-claude-data) describe account data download through privacy settings. Import the original conversation-containing ZIP, not a public share link.
- Grok: [the provider’s data-rights FAQ](https://x.ai/legal/faq#how-do-i-submit-a-data-subject-rights-request) directs account data download through Settings/Data Controls. Use the ZIP containing `prod-grok-backend.json`; X account archives and standalone Grok exports are not assumed interchangeable.

Export controls can change. Do not ask users to paste exports or passwords into chat/issues. Representative acceptance uses an explicitly authorized isolated encrypted local copy, recording only generic pass/fail evidence in the repository.

## Provider permissions

Codex registrations bind ChatGPT, Grok registrations bind Grok, and Claude local-MCP registrations bind Claude. Shared-only clients have no raw-source access. Curation is an additional explicit permission and does not enable automatic publication or sharing. Existing manual local features remain usable offline with no curator.

Schema 5 extends the schema-4 provider constraints to Claude while retaining existing grants, policies, notes and child records. It creates a default-off Claude publication policy. Older v3 grants still undergo the v4 shared-only downgrade; no migration assigns broader access. The current backend has no cross-provider delegation override. The owner has approved that as a later advanced option, with explicit selected-source grants and independent publication consent; see AGENT_CURATION.

## Additional providers

| Candidate         | Available export evidence                                                                                                                         | Next requirement before support                                                                                                                                        |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Gemini            | [Google documents Takeout download](https://support.google.com/gemini/answer/16920332)                                                            | Inspect representative official conversation/activity structure and omissions; create fictional versioned fixtures; define identity/thread semantics and curator scope |
| Microsoft Copilot | [Microsoft documents activity export](https://support.microsoft.com/en-us/privacy/manage-your-copilot-activity-history-in-the-privacy-dashboard.) | Distinguish consumer from Microsoft 365 formats; inspect actual export contract, response completeness and stable IDs; add fixtures and scope tests                    |
| Other products    | Evaluate official export availability individually                                                                                                | Same parser, privacy, provenance, recovery and native acceptance gates; no live scraping fallback                                                                      |

Download availability alone does not establish a reliable conversation parser. Gemini and Copilot are requested candidates, not implemented choices in the app. Keep them out of supported-provider claims until their contracts and tests exist. Cloud/local model choice and access scope are independent; a future local runner must have the same explicit source permissions.

## Fixture and review references

Synthetic contracts live under `src-tauri/tests/fixtures/<provider>/<contract-id>/`, with a manifest identifying provenance, omissions and version. Grok field understanding also references the [published account-export parser](https://github.com/NinZine/chat-converter/blob/master/grok_parser.py); Claude field understanding references [Loominary’s parser](https://github.com/Laumss/loominary/blob/main/src/utils/fileParser/claudeParser.js). Pre-Drink’s adapters are independently written Rust; no live-capture implementation or third-party code was copied.
