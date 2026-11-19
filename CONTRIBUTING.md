# Contributing

Read docs/AGENT_REVIEW.md before changing the vault or sharing boundaries. Keep ChatGPT-only scope unless the owner expands it. Preserve local-first behavior, explicit approvals, provenance and encryption format compatibility.

Use synthetic fixtures only. Never commit exports, databases, backups, credentials, local MCP configurations, private screenshots or personal transcripts. Inspect the exact staged file set and scan it before every push. A private repository is not a data-storage exception.

Run formatting, frontend build, Rust tests and Clippy sequentially before a code change is considered ready. Add meaningful regression cases for encryption/privacy/import failures; avoid tests that merely mirror presentation code. Update product/security/recovery documentation when behavior changes.

Do not add telemetry, cloud AI calls, automatic fact extraction, new data sharing, or recovery keys without an explicit product decision. Do not claim an independent audit or signing step happened unless it did. Original contributions are covered by the repository’s MIT license; preserve third-party notices.
