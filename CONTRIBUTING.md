# Contributing

Pre-Drink is an early source-only macOS project. Small, focused fixes with fictional reproductions are welcome. Discuss changes to storage, provider scopes, approval policy, or the product model before implementing them.

## Protect private material

Never commit exports, vault databases, passwords, API keys, MCP credentials, real chat excerpts, or screenshots containing personal content. Use the synthetic fixtures under `src-tauri/tests/fixtures` and the generators in `scripts`. Local vault data and diagnostics belong outside the repository. The tracked-file guard is a backstop, not proof that a change contains no private information.

Report sensitive security issues privately using GitHub's **Report a vulnerability** option if available. Do not post credentials or personal exports in public issues. A sanitized description should be enough to begin a discussion.

## Local checks

Install the toolchain listed in the README, run `npm ci`, and cache the locked Rust dependencies with `cargo fetch --manifest-path src-tauri/Cargo.toml --locked`.

- `npm run check` runs the frontend build, startup/reading checks, and core Rust tests.
- `npm run format:check` checks formatting; `npm run format` applies it.
- `npm run verify:local` runs the complete local gate, including fresh free advisory queries. Follow [dependency setup](docs/DEPENDENCY_CHECKS.md) first. Stage new source files so the source fingerprint includes them.
- `npm run hooks:install` installs the optional exact-source verification pre-push guard.

Do not run Tauri builds and Cargo tests concurrently in the same target directory. GitHub Actions remains manual-only; do not introduce automatic hosted builds or paid dependencies without maintainer agreement.

## Review expectations

Explain the user-visible problem, the resulting behavior, validation performed, and remaining limits. Changes to approval, privacy, recovery, or provider scopes need regression coverage of the boundary they affect. Keep backend enforcement authoritative; a UI label or prompt is not an access-control rule.

Preserve MIT attribution and third-party notices. Update the relevant maintained documentation when behavior changes. Do not include generated app bundles, DMGs, archive exports, or dependency directories in a pull request.
