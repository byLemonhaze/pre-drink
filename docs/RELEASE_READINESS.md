# Release readiness — 2026-11-22

**Ready for continued owner-only local use.** The supplied separate-agent review at `abd9bea` establishes no new blocking code defect. Core local acceptance and the current release preflight have passed. **Not yet ready for outside distribution:** Apple Developer ID signing, notarization and a clean downloaded-install check remain last. The repository and landing page stay private/unpublished until the owner chooses launch.

This is an early personal tool, not a professionally security-audited product. Keep original exports and verified encrypted backups. A test count and an agent review are bounded evidence, not a safety certification.

## Fixed gates

| Gate                                         | Current state                                                                                                                                                                                                    | Next action                                                                                                                       |
| -------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| Local release preflight                      | Passed from clean `6b13364`, runtime `1dc3cc0`: 219 tests, refreshed advisories, both Mac graphs, build/format/source guard, provider fixtures and Clippy. Check-only exited before Apple operations.            | Repeat only after implementation/dependency changes or expired advisory evidence at release.                                      |
| Core fresh-user acceptance                   | Passed: owner-created fictional vault, import/search, edited draft, approval, exact outgoing preview/save and lock/reopen. Subsequent privacy, credential setup and agent controls have focused native evidence. | No new full walkthrough round required without a relevant change or defect.                                                       |
| Current implementation separate agent review | Received for `abd9bea` on Nov 22 at 11:04 PM ET. Recent fixes confirmed, one low-priority logging limitation identified, no new blocking code defect. Linux checks only; local Mac evidence is separate.         | Gate satisfied for this implementation. See REVIEW_RESPONSE for scope and corrections; this is not a professional audit.          |
| Apple distribution                           | Not started; deliberately deferred. Local apps have ad-hoc signatures only.                                                                                                                                      | Owner enrollment decision, Developer ID build, notarization, ticket/signature/checksum verification and downloaded clean install. |
| Owner launch decision                        | Pending. Passing checks does not authorize invitations, public visibility or release publication.                                                                                                                | Choose intended audience and release claims after the distribution checks.                                                        |

## Limits to disclose

- **Provider imports:** one representative ChatGPT export passed isolated backend and native post-import checks. Grok/Claude adapters have fictional contracts and isolation tests; treat them as experimental until representative official exports pass. No Gemini/Copilot support is claimed.
- **Accessibility:** keyboard/focus/layout checks and spoken password/unlock labels have evidence. Automatic saving/completion speech remains unconfirmed; the owner deferred the listening check. Verify before promising those announcements or relying on them for a participant. No full accessibility-conformance claim.
- **Dependencies:** the SDK's unmaintained Chrono dependency remains active on macOS. Two other warnings are absent from both resolved Mac normal/build graphs. All three remain visible; refreshed audits found no vulnerability findings, not a warning-free graph.
- **Activity history (R10-1):** encrypted bounded metadata, not a tamper-proof audit. Authenticated clients can roll older events out of the shared cap; invalid/stale/revoked-token attempts are not recorded. No client can call the owner clear-history command. Per-client retention and aggregate denials are optional follow-up work.
- **Curation:** Codex and Grok have bounded fictional client runs; broad semantic quality, actual Claude client use and arbitrary web-chat integrations remain unverified. Cloud curation is optional and manual local use needs no API credits.
- **Platform:** native acceptance is on Apple Silicon. A configured minimum macOS version is not proof of every supported OS, Intel hardware or physical power-loss scenario.

## Optional improvements, not new blockers

Keychain-backed client storage, versioned KDF metadata, per-client activity retention/denial metrics, single-revoke confirmation and onboarding/navigation polish remain backlog items. Manual updates are documented; an automatic updater is deferred. No optional item silently reopens completed acceptance or justifies spending on GitHub.

The next required work for public download is the owner-deferred Apple stage, followed by artifact acceptance and launch approval. Representative-provider and human accessibility checks can be added when their inputs are available; limit claims until then. New reproducible blocking defects still require fixes and focused revalidation.

## Evidence

- [Separate review dispositions](REVIEW_RESPONSE.md): review at `abd9bea`, optional findings, source corrections and validation limits.
- [Agent-control review](AGENT_CONTROLS_REVIEW.md): two reproduced defects fixed at `1dc3cc0` and later native acceptance.
- [Validation](VALIDATION.md) and [native walkthrough](NATIVE_WALKTHROUGH.md): chronological test/build/native records.
- [Dependency checks](DEPENDENCY_CHECKS.md), [provider contracts](PROVIDERS.md) and [curation acceptance](CURATION_ACCEPTANCE.md): detailed limitations.
- [Release procedure](RELEASING.md): free check-only preparation, final Apple operations and manual updates.

No paid GitHub usage, cloud processing of personal archives, public publication or Apple account/payment action is authorized by this document.
