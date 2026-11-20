# macOS distribution

**Pre-release visibility:** Keep the repository private and Pages unpublished until the owner explicitly chooses to launch. A successful build or UI change does not authorize making either public.

Local/ad-hoc builds do not establish a verified publisher identity. Developer ID signing associates the app with an Apple identity and lets macOS detect modification. Notarization adds Apple's automated distribution check and ticket. Neither replaces vault encryption or a security audit. References: [Apple Developer ID](https://developer.apple.com/developer-id/), [Tauri signing](https://v2.tauri.app/distribute/sign/macos/).

Developer ID signing/notarization have not been completed for this version. Do not label local artifacts as signed public releases.

## Prepared release workflow

The maintainer needs Apple Developer Program membership, an installed **Developer ID Application** certificate/private key, and a local `notarytool` Keychain profile. Never store credentials in this repository or share them in chat.

Set `APPLE_SIGNING_IDENTITY` to the installed identity and `PRE_DRINK_NOTARY_PROFILE` to the Keychain profile name, then run:

```sh
bash scripts/release-macos.sh
```

Install `cargo-audit` 0.23.0 with `cargo install cargo-audit --version 0.23.0 --locked` before running the script.

The script installs locked frontend dependencies, checks formatting, builds, tests and checks Rust warnings with locked dependencies, runs npm/cargo security audits and the repository guard, builds with hardened runtime, verifies signatures, submits to Apple, requires Accepted status, staples the ticket, and checks Gatekeeper. Successful output is `artifacts/release/Pre-Drink.zip`, with `SHA256SUMS` and `SOURCE_COMMIT`. The latter must identify a clean reviewed commit; run from a clean checkout. A checksum detects corruption, not a trustworthy publisher by itself. Notarization uploads the application bundle, never the user's vault.

Review current dependency advisories and complete real-export/native-drag/large-library validation before distribution. A configured minimum OS is not evidence of testing every macOS version or Intel hardware. Never tell users to disable Gatekeeper to compensate for an unsigned release.

## Landing page and download switch

The static landing page lives in `docs/index.html` and is suitable for GitHub Pages from `main:/docs`. It has no analytics, remote fonts, signup form or client script. The hosting provider still receives ordinary web requests; this is separate from the offline app.

After a signed/notarized release passes the remaining validation gates, publish its archive and SHA-256 checksum as GitHub release assets. Replace the disabled download control with that verified release asset URL, state the supported architecture/version, and update availability text and the validation record. Never point the public button to a local ad-hoc bundle or call an unsigned artifact notarized.

## Manual updates (v1)

The app makes no automatic update requests. Once public releases exist, users can explicitly visit the repository’s Releases page, read the release notes and supported architecture, and download a newer signed/notarized ZIP plus its checksum. Verify the checksum and publisher signature/Gatekeeper result before opening. Never bypass a macOS warning. Make an encrypted backup, quit Pre-Drink, replace the app in its stable install location, and reopen the existing vault. Follow any migration notes. Older versions must refuse newer schema versions; do not downgrade a migrated vault without a compatible backup.

An automatic updater is deferred until its signing keys, verification, rollback policy and opt-in network behavior are designed and tested. Do not enable a download button until the signed artifact and its checks are actually verified.

## Curation release gate and upgrade notes

Schema 4 is an additive migration. Release notes must state that existing manual notes/approvals remain, broad old raw-chat grants become shared-only, and source/curation access requires a fresh provider-bound registration. Backup before upgrading; do not downgrade a migrated vault. In-app restore/password change resets curator grants and automatic-publication policies. No personal data or generated credentials belong in release evidence.

Advertise ChatGPT ZIP import separately from Codex/Grok MCP roles. Fictional native/protocol writing checks do not establish model-led quality, web-chat connectivity or Grok export support. Complete the fresh fictional model-led session/resume test and representative local-export gate before making those claims. Cloud curation remains optional; offline manual use requires no account, model or API key. A local-model runner is planned, not bundled.
