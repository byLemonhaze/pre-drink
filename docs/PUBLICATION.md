# Source-only publication checks

Prepared November 27, 2026. Source publication is separate from a downloadable Mac release.

## Presentation

- README starts with a work-in-progress/build-from-source banner, describes known limitations, and links to the MIT license.
- Screenshots use the restored cream/orange native interface and fictional content only. The cabinet redesign was withdrawn; its unrelated stable-ID source-link fix remains.
- No prebuilt app, DMG, ZIP download, GitHub Release, or Pages download site is published.
- Public documentation is organized around usage, architecture, privacy boundaries, reproducible checks, and outstanding work. Internal execution logs, review-response scratchpads, and unfinished landing assets were removed from the current tree.

## History review

The pre-publication baseline through `286835e` contains 64 reachable commits and 984 unique historical blobs. All fetched branches/tags were included; there were no additional remote branches or pull requests.

- Official Gitleaks 8.30.1, verified against its published download checksum, scanned the complete reachable history with built-in rules and inline suppressions disabled: **zero findings**.
- A separate local content inventory checked all historical blobs for personal export/vault references, known personal test material, local home paths, private keys, and unexpected email metadata: **zero findings**. Commit emails use GitHub no-reply addresses. Content email candidates are reserved fictional addresses or an icon filename.
- Historical binary files were app icons only; no archive ZIP, database, credential file, or screenshot was present.
- All 33 existing Actions run-log archives also passed the local secret scan. Targeted log review found only generic hosted-runner home paths, not personal machine paths. Commit-message marker checks passed.
- Existing GitHub releases, artifacts, issues, pull requests, and Pages configuration were checked separately. Publication does not start Actions or upload local build artifacts.

Scanner output and operational reports remain outside Git. No real data is uploaded to a scanner service; scans run locally. Historical engineering notes contain generic implementation/test evidence, not personal transcripts. History is retained rather than rewritten solely to hide the withdrawn UI experiment.

Reproduce the main history check with a locally installed Gitleaks:

```sh
gitleaks git . --log-opts='--all --full-history' --ignore-gitleaks-allow --redact
```

Automated scans can miss private content. The result is bounded evidence combined with manual review, not a promise that a scanner recognizes every possible secret.

## Current verification

On November 27, the restored-interface publication tree passed `npm run verify:local`: **227 tests** (178 Rust, 28 frontend, 21 Python), production frontend build, formatting, tracked-file guard, fictional provider-quality fixtures, and Clippy with warnings denied. The personal-export harness remains intentionally ignored in routine tests; no personal vault was opened during publication preparation.

Fresh npm and RustSec checks reported **zero vulnerability findings**. Three informational warnings remain: Chrono is an active Mac dependency marked unmaintained; proc-macro-error and GLib warnings are in the cross-platform lockfile but absent from both resolved Mac normal/build dependency graphs. See [dependency checks](DEPENDENCY_CHECKS.md).

A fresh native build of the restored interface was opened against separate fictional encrypted storage. Archive reading, memory listing, exact selected-context preview, and lock were observed. All four screenshots, including the locked-vault opening image, were visually inspected; their metadata contains only image dimensions/resolution and the generic screenshot label. No context was copied, saved, or sent to an agent.

Remaining distribution, accessibility, platform, and provider checks are listed in [release readiness](RELEASE_READINESS.md). Source publication does not close them.
