# Validation evidence and limits

Test results describe a particular source snapshot and environment. They are not a security certification, a guarantee against data loss, or proof that every provider export works.

## Reproduce the checks

Follow DEPENDENCY_CHECKS for the pinned auditor and cached dependency prerequisites. Run checks sequentially:

```sh
npm run verify:local
```

The gate runs formatting, the tracked-file guard, a production frontend build, startup/reading tests, Python developer-tool tests, Rust unit/integration tests, fictional provider-quality fixtures, Clippy, and fresh npm/RustSec advisory checks. It resolves normal/build dependency graphs for both Apple Silicon and Intel Mac targets. Functional Cargo checks run offline after dependencies are cached. No model, personal vault, or GitHub Actions run is needed.

## Automated coverage

| Area                 | Representative checks                                                                                                                                                                        |
| -------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Import               | Versioned ChatGPT/Claude/Grok fixtures, malformed ZIPs, limits, branch retention, idempotence, atomic rollback, encrypted pause/resume and upgrade checkpoints.                              |
| Storage/recovery     | Wrong passwords, encryption, tampering, truncation, verified backups, empty-storage restore, password change, interrupted generation switches, disk/permission failure simulation and retry. |
| Privacy and approval | Private/sensitive sources, edit/reimport invalidation, stale reviews, source deletion, blocked briefs, and delivery-time rechecks.                                                           |
| Agent boundaries     | Provider isolation, shared-only defaults, credential validation, pause/revoke, scoped curation, revision/history rules, lock/expiry and bounded access logging.                              |
| UI/data access       | Bounded message/library rendering, inert Markdown, theme contrast, stale-request handling, modal acknowledgement, and exact-ID source lookup independent of title or first-page position.    |
| Developer tooling    | Advisory/receipt validation, release routing, source guards, fixture generators and no-Apple check-only behavior.                                                                            |

The November 27 restored-interface publication tree passed **227 tests** (178 Rust, 28 frontend, 21 Python), the production build, fixture checks, Clippy, and fresh dependency advisories. See [publication checks](PUBLICATION.md) for exact scope and remaining warnings. The withdrawn cabinet interface is not the current product.

## Native and real-data evidence

Fictional Apple Silicon walkthroughs have covered fresh-vault import/search, memory edit/review, exact selected context preview/save, lock/reopen, encrypted recovery/password change, native file dialogs, Mac lock/sleep, large-library reading, provider workspace controls, and per-client confirmation/cancellation. Some checks included direct user participation, particularly credential changes and spoken VoiceOver labels. These observations do not establish all possible failure modes or OS versions.

One representative official ChatGPT export passed isolated backend checks and native post-import search/reading/lock checks. Its contents, filenames, passwords, and raw diagnostics are not repository fixtures. Representative official Claude/Grok export checks remain open; their synthetic contracts do not substitute for those checks.

Codex and Grok Build have exercised bounded fictional MCP read/write flows. See MCP_ACCEPTANCE and CURATION_ACCEPTANCE for scope. Claude client execution and arbitrary web-chat integrations are not established by the protocol tests.

## Remaining verification

- Human confirmation of automatic import-status/completion speech and broader enlarged-text coverage.
- Representative Claude/Grok export compatibility, wider archive structures and curation-quality evaluation.
- Intel hardware, broader macOS versions, and physical power-loss scenarios.
- Developer ID signing/notarization and a clean downloaded-install walkthrough for the exact release artifact.
- A new independent review of later changes; previous AI-assisted reviews are snapshot evidence and not a professional security audit.

Sensitive-content detection and cloud models can make mistakes. Keep original exports and verified encrypted backups, and review the context you share.
