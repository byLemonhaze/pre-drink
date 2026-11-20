# Export compatibility fixtures

These files contain invented conversations, not redacted personal exports. Do not replace them with user data. Generated ZIPs, databases, screenshots and raw logs stay outside Git, even while this repository is private. The repository guard still rejects files named `conversations.json`; the deliberately named `*.input.json` files are source fixtures, packaged into ZIP entries only during tests.

## Contract layout

`<provider>/<contract-id>/manifest.json` identifies the supported shape, revision, provenance, cases, preserved fields and known omissions. A contract ID is an internal test label. It is not an official provider format version unless the manifest explicitly cites one. The current `chatgpt/mapping-v1` examples describe the existing parser; they do not prove compatibility with every current ChatGPT account/export variant. Claude and Grok now have separate synthetic observed-format contracts. None substitutes for representative owner-local export acceptance.

Each case has an input JSON array and a separately reviewed expected normalization. Compare IDs, roles, content, parent/branch links, active flags, numeric timestamps, sensitivity, order and omission warnings. The test also checks that original JSON values, including unknown fields and reader omissions, remain in the archive. JSON whitespace/key order is not preserved byte-for-byte. Timestamp fields in expected files use floating-point notation to match the normalized numeric type.

| Case            | What it protects                                                                                                                                               |
| --------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `branched`      | Preferred provider ID, punctuation, fractional timestamps, depth-first ordering, alternate branches and unknown raw fields                                     |
| `mixed-content` | UTF-8 ID length, mixed text/image references, nearest retained parent across omitted nodes, tool/system/hidden/analysis/commentary omissions and text fallback |
| `fallback`      | Missing title, nonnumeric timestamps, invalid current node, deterministic tied-leaf fallback and harmless epoch text                                           |

`tests/import_contract.rs` also packages nested, split and traversal-shaped ZIP entry names without extraction; exercises repeat import/search and inactive-branch privacy; rejects later malformed/duplicate records without partial writes; and checks the declared uncompressed size bound without allocating a huge payload.

## Run and extend

```sh
cargo test --manifest-path src-tauri/Cargo.toml --locked --test import_contract
python3 scripts/create_import_fixtures.py --output /tmp/pre-drink-import-fixtures
```

Use a fresh output folder. The generator refuses existing ZIP names. It creates a valid split/nested archive and an archive whose first entry is valid but whose second entry is malformed. Only fictional data is used.

Native acceptance in a separate disposable vault:

1. Import `fictional-chatgpt.zip` with the native picker: expect 2 new conversations, 6 text messages, and notes explaining 7 omitted messages.
2. Search `mahogany`: the reading-nook conversation matches an inactive branch. Enable retained branches to see the matching reply and its follow-up. Tab/Space must operate the checkbox.
3. Reimport the same ZIP: expect 0 new / 2 refreshed, with no increase in conversations/messages.
4. Import `fictional-invalid.zip`: expect an invalid-JSON error, no new conversation, and all existing data preserved.
5. Mark the reading-nook conversation private and search again: normal search has no matching row. Cancel a new picker operation and verify counts remain unchanged. Lock the vault.
6. Actual Finder-to-app drag and drop needs its own verification; picker evidence is not drop evidence.

Before changing a parser, add the smallest fictional case that reproduces the format difference and review the expected result independently. Append compatible examples/revision notes; use a new contract folder for a materially different layout. Keep old contracts and rejection tests. A representative official export may be inspected locally under the project's private-data rules, but never committed or sent to an agent provider merely to create fixtures.
