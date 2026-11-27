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

- `chrono`: [RUSTSEC-2026-0543](https://rustsec.org/advisories/RUSTSEC-2026-0543.html), unmaintained. Our direct use has been replaced by `time`; the mandatory MCP SDK/schema dependency still keeps it in the build.
- `proc-macro-error`: [RUSTSEC-2024-0370](https://rustsec.org/advisories/RUSTSEC-2024-0370.html), unmaintained, in the cross-platform lockfile.
- `glib`: [RUSTSEC-2024-0429](https://rustsec.org/advisories/RUSTSEC-2024-0429.html), an unsoundness warning in the cross-platform lockfile. This must be revisited before expanding platform support.

The audit gate rejects vulnerabilities and reports informational warnings separately. It does not establish that every dependency is maintained or safe. Upstream replacement/platform follow-up remains open; no allowlist was added.

## Mac impact and upgrade assessment — Nov 22

On November 22, the official registry reported that the locked `rmcp` 3.11.0, Tauri 2.12.7 and dialog plugin 2.8.4 were the latest stable versions at that check. Tauri 3 prereleases are not a drop-in security patch. The current supported dependency stack cannot eliminate all three warnings with a compatible stable update. No library was forked, advisory suppressed or prerelease substituted to produce a green report.

| Warning                                | Apple Silicon and Intel Mac dependency graphs                            | Resolution path                                                                                                                                                                                                                          |
| -------------------------------------- | ------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `chrono` 0.4.45, unmaintained          | Present: mandatory MCP SDK dependency and SDK-enabled schema integration | Direct date parsing now uses `time` with before/after timestamp-contract regressions. The remaining fix requires a supported MCP SDK release that no longer depends on Chrono. No fixed Chrono version exists.                           |
| `proc-macro-error` 1.0.4, unmaintained | Absent from normal/build dependencies for both Mac targets               | Pulled through the Linux GTK macro chain. Requires compatible upstream GTK/Tauri changes. Remains in the cross-platform lockfile and audit output.                                                                                       |
| `glib` 0.18.5, unsound iterator        | Absent from normal/build dependencies for both Mac targets               | RustSec's fix is >=0.20; Tauri 2's GTK 0.18 stack still selects the older family. Independently forcing a second newer GLib version would not replace the affected one. Resolve the supported framework chain before Linux distribution. |

Audit policy 2 now resolves both Mac dependency graphs locally with locked/offline Cargo, including host build dependencies. Every validation—including cached evidence before push—prints each advisory's target presence. A known-unsound package/version active in either graph blocks acceptance; failed/empty/unrecognized graph results cannot be treated as absence. This does not hide unmaintained warnings or claim that absent Linux dependencies were patched. Registry downloads must be cached separately if Cargo needs them.

These are dependency-graph checks, not Intel hardware/build acceptance or a proof that maintained libraries have no flaws. Chrono remains an active maintenance risk; no existing RustSec vulnerability finding is being reclassified as harmless. Original advisories and unresolved warnings stay visible above.

## Direct replacement and consistent release requirements — Nov 22

Official registry metadata was rechecked on November 22 for the versions above; recheck before future dependency changes. Replaced the app's direct Chrono requirement with `time` 0.3.55 (`std` and `parsing`, no clock/network feature). That version was already in the lockfile; the lockfile change moves only the root dependency edge. No transitive library was downgraded or forked.

The same fictional timestamp contract passed against the old parser and replacement: offsets, millisecond flooring including pre-epoch fractions, arbitrary fractional precision, upper/lowercase separators, the earlier accepted space separator and leap-second representation, Mongo dates, missing/invalid dates and complete raw JSON retention. Both Claude/Grok adapters are exercised. The remaining Chrono tree is MCP SDK → Chrono and SDK-enabled schema integration → Chrono. All three upstream warnings remain visible; this is partial replacement, not a warning-free claim.

Local verification, the manual-only GitHub workflow and the future release script now invoke the same `verify:local` gate. The workflow still has no push/pull-request trigger and was not dispatched. Release setup caches locked dependencies before offline functional checks. `bash scripts/release-macos.sh --check-only` requires a clean source checkout and runs developer checks without Apple credentials, signing, bundling or notarization—even if Apple credentials happen to be configured. It still uses free registries/advisory requests; use `verify:offline` when recent evidence and dependencies are already cached and no network is desired. Six mocked-command regressions check routing, missing prerequisites, dirty sources and failed gates without contacting Apple or any registry.

## Nov 24 — Patched cookie advisory

A fresh refresh correctly blocked on [RUSTSEC-2026-0594](https://rustsec.org/advisories/RUSTSEC-2026-0594.html), published Nov 23: cookie's private encrypted-cookie nonces can repeat after specific fork-without-exec behaviour. The advisory identifies 0.18.3 as patched. Updated the transitive lockfile entry from 0.18.2 to 0.18.3, with no other dependency version change.

The inspected `aarch64-apple-darwin` feature graph enabled cookie's default feature, not its private-cookie encryption feature, and application sources contained no PrivateJar/fork calls. This narrows the observed application's exposure; the vulnerable version was still removed, rather than waived. SQLCipher/Argon2 vault encryption and credential formats were not changed.

The full fresh-advisory local verifier passed after the patch, including 226 tests, both Mac dependency graphs and Clippy. Rust/npm audits reported no vulnerability findings; the existing Chrono, proc-macro-error and glib warnings remain visible as described above. This result is dated evidence, not a promise against future advisories.
