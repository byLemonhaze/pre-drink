# Release readiness

**Source-only work in progress.** The repository is suitable for code review, contributions, and developer-run local builds. It does not offer a ready-to-download Mac release.

| Area                | Evidence / remaining limit                                                                                                                                                        |
| ------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Core implementation | Local tests cover imports, encryption/recovery, approval invalidation, provider isolation, and MCP boundaries. See VALIDATION for dated results.                                  |
| Native workflows    | Fictional Mac walkthroughs cover import, search, memory review, selected context, recovery, locking, and per-client controls. Later changes require relevant rechecks.            |
| Real exports        | One representative ChatGPT export passed isolated backend/native checks. Representative Claude/Grok exports remain unverified.                                                    |
| Accessibility       | Keyboard/focus checks and some spoken labels have evidence. Automatic import-status/completion announcements and broader enlarged-text coverage remain open.                      |
| Dependencies        | Fresh advisories are required for the release commit. Three known warnings remain documented in DEPENDENCY_CHECKS.                                                                |
| Independent review  | AI-assisted reviews informed development. No professional security audit or cryptographic certification is claimed; later changes are not automatically covered by older reviews. |
| Apple distribution  | Developer ID signing, notarization, stapling, Gatekeeper and clean downloaded-install verification remain incomplete. Local ad-hoc bundles are developer artifacts only.          |
| Platform matrix     | Native evidence is from Apple Silicon; configured minimum macOS and Intel dependency graphs do not establish broad hardware/OS acceptance.                                        |

Keep original exports and verified encrypted backups. Sensitive-content detection and model curation can make mistakes. Review the exact context you release, and read SECURITY for the threat model and limits.

Source publication does not authorize uploading unsigned binaries, enabling a download landing page, or claiming general-release readiness. See RELEASING for the later distribution process.
