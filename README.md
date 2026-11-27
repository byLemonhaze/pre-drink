# Pre-Drink

> **Work in progress — build from source for now.** This repository shares the implementation, not a ready-to-download Mac app. A Developer ID signed and notarized Mac release will come later. No prebuilt app or DMG is published here.

**Your AI history, sober and ready.**

Pre-Drink is a free, MIT-licensed macOS app for turning AI chat exports into a local memory vault. Keep original conversations, curate useful facts and decisions, assemble briefs, and control the context your other agents can read.

Built with **Rust, Tauri 2, React, TypeScript, and SQLCipher/SQLite FTS5**.

## A look inside

These screenshots show the current cream/orange interface with **fictional garden data only**. They are native app captures, not design mockups. The interface is still evolving.

![The locked vault, with no history exposed](docs/screenshots/locked.png)

![Search and read an imported conversation](docs/screenshots/archive.png)

![Review and edit structured memories](docs/screenshots/memory.png)

![Inspect the exact context before exporting it](docs/screenshots/context-preview.png)

## What works today

- **Official ZIP imports:** recognized ChatGPT, Claude, and Grok formats; preserved text branches and snapshots; encrypted pause/resume checkpoints.
- **A local archive:** full-text search, Markdown reading, privacy flags, and source links.
- **Curated memory:** editable facts, decisions, projects, and briefs. Manual edits return to draft and require a new review before sharing.
- **Controlled context:** preview selected outgoing Markdown before copying or saving it. Private, sensitive, unapproved, and source-blocked items are excluded.
- **Optional agent curation through local MCP:** provider-scoped working memories feed a shared layer. Matching-provider curators can draft and revise their own contributions; other providers cannot read their raw history. Shared-only access is the default.
- **Encrypted local storage and recovery:** a separate vault password, encrypted backups, restore, and automatic locking. Locking stops agent access.

Import, search, manual editing, and backups work offline without a model or API key. Cloud curation is optional: an authorized cloud agent can send its permitted context to its provider and use that provider's subscription or API allowance. Pre-Drink does not run a model itself. Local-model execution is planned.

## Build from source

Requirements: macOS 12+, Xcode command-line tools, Node.js 22.12+ or a compatible newer release, and Rust 1.93+. Native verification has used Apple Silicon; Intel hardware and the full macOS version range are not yet validated.

```sh
git clone https://github.com/byLemonhaze/pre-drink.git
cd pre-drink
npm ci
npm run tauri -- dev
```

For an optimized **local development build**:

```sh
npm run tauri -- build
```

The resulting local bundle is not a Developer ID signed/notarized distribution. Do not disable Gatekeeper or treat it as a public release. See [build and release guidance](docs/RELEASING.md).

Choose a unique vault passphrase of at least 14 characters. There is no default password, password reset, or hidden recovery key. Keep original exports and verified encrypted backups. Start with fictional data if you are evaluating the project.

`npm run dev` alone previews the interface; vault operations require the native app. Follow the [quickstart](docs/QUICKSTART.md) for the first useful memory and optional client setup.

## Status and limits

This is an early working personal tool, **not a professionally security-audited product**. Encryption does not protect an unlocked vault from malware, and sensitive-content detection has false positives and false negatives. Read the [security model and limits](docs/SECURITY.md).

One representative ChatGPT export has passed isolated backend and native post-import checks. Claude and Grok adapters have versioned fictional contracts; representative official exports remain unverified. Attachments, audio, images, tool results, and reasoning channels are not separately imported/rendered. No Gemini/Copilot imports, cloud sync, or live capture are provided.

Signing/notarization, downloaded-install checks, broader platform/provider coverage, and remaining accessibility checks are still open. See [release readiness](docs/RELEASE_READINESS.md) and the [roadmap](docs/ROADMAP.md). Test results are dated evidence, not a safety guarantee.

## Development and verification

```sh
npm run check
npm run format:check
```

The full local gate adds dependency advisories, Python tooling checks, provider fixtures, and Clippy:

```sh
npm run verify:local
```

See [dependency-check setup](docs/DEPENDENCY_CHECKS.md) and [validation evidence](docs/VALIDATION.md) for prerequisites, test scope, and known warnings. Run Cargo tests and Tauri builds sequentially. GitHub Actions is manual-only; normal pushes do not start hosted builds.

## Documentation and contributing

Start with the [documentation index](docs/README.md), [architecture](docs/ARCHITECTURE.md), [MCP contract](docs/MCP.md), or [contributing guide](CONTRIBUTING.md).

Never upload real chat exports, vaults, passwords, client credentials, or unredacted diagnostics to an issue or pull request. Reproduce problems with fictional data.

Pre-Drink is an **independent implementation, not a Loominary fork**. Loominary informed the export and branch model; no source, branding, or live-capture code was copied. See [provenance](docs/PROVENANCE.md). Original project code is available under the [MIT license](LICENSE).
