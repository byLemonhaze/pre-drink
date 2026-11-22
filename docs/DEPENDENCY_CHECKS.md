# Free local dependency checks

The application stays local/offline by default. These are **developer** checks: npm receives package/version metadata and cargo-audit refreshes the public RustSec database. They do not read vaults, call models, dispatch GitHub Actions or incur paid service usage.

## Setup and routine

Install the pinned free auditor once from the public Rust registry:

```sh
cargo install cargo-audit --version 0.23.0 --locked --root artifacts/audit-tools
```

An installed `cargo-audit` on PATH is accepted only at the pinned version. Build dependencies must already be installed/cached. Stage new source files before full verification so they enter the source fingerprint.

- `npm run verify:local`: refresh both advisory sources, then run the existing local functional/build/source checks. Functional Cargo commands remain offline.
- `npm run audit:local`: refresh advisory evidence only; this does **not** replace full source verification.
- `npm run verify:offline`: run full checks without advisory network requests, requiring a successful matching audit from the last 24 hours.
- Installed pre-push hook: require clean exact verified sources **and** recent matching audit evidence, without network requests. Refresh an expired audit before retrying. Documentation-only changes still receive formatting/source-guard checks.

Vulnerability findings at any severity, nonzero audit exit, invalid/incomplete JSON, missing/changed reports, changed lockfiles, a failed refresh, future/expired timestamps and changed check policy block acceptance. Receipts and reports live in ignored `artifacts/`. They bind both lockfiles, report hashes, the auditor version and RustSec database commit. A receipt can become stale before a newly published advisory is known; the 24-hour cache is a bounded compromise, not continuous monitoring. Hooks are optional local guardrails, not tamper-proof attestations; do not use `--no-verify` as a substitute.

## Nov 22 findings

The supplied review's `url` issue was confirmed against [RUSTSEC-2026-0592](https://rustsec.org/advisories/RUSTSEC-2026-0592.html). The lockfile now uses 2.5.9, with its required `form_urlencoded` 1.2.3 update. Both public audit queries passed after this update.

Three Rust warnings remain visible, not ignored or marked fixed:

- `chrono`: [RUSTSEC-2026-0543](https://rustsec.org/advisories/RUSTSEC-2026-0543.html), unmaintained. It is used directly and transitively; removing only the direct use would not clear the full graph.
- `proc-macro-error`: [RUSTSEC-2024-0370](https://rustsec.org/advisories/RUSTSEC-2024-0370.html), unmaintained, in the cross-platform lockfile.
- `glib`: [RUSTSEC-2024-0429](https://rustsec.org/advisories/RUSTSEC-2024-0429.html), an unsoundness warning in the cross-platform lockfile. This must be revisited before expanding platform support.

The audit gate rejects vulnerabilities and reports informational warnings separately. It does not establish that every dependency is maintained or safe. Upstream replacement/platform follow-up remains open; no allowlist was added.
