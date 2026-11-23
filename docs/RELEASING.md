# macOS distribution

**Pre-release visibility:** Keep the repository private and Pages unpublished until the owner explicitly chooses to launch. A successful build or UI change does not authorize making either public.

Local/ad-hoc builds do not establish a verified publisher identity. Developer ID signing associates the app with an Apple identity and lets macOS detect modification. Notarization adds Apple's automated distribution check and ticket. Neither replaces vault encryption or a security audit. References: [Apple Developer ID](https://developer.apple.com/developer-id/), [Tauri signing](https://v2.tauri.app/distribute/sign/macos/).

Developer ID signing/notarization have not been completed for this version. Do not label local artifacts as signed public releases.

## Free local preflight

From a clean checkout, run `bash scripts/release-macos.sh --check-only` to install/cache locked dependencies and execute the complete local verification/advisory gate. It needs no Apple credentials and exits before app bundling, signing or notarization. Registry/advisory requests are free and contain package metadata, never vault data. For already-installed dependencies and current audit evidence, `npm run verify:offline` is the network-free routine. The manual-only GitHub workflow now uses this same verification gate; do not dispatch it under the owner's no-paid-GitHub constraint.

## Prepared release workflow

The maintainer needs Apple Developer Program membership, an installed **Developer ID Application** certificate/private key, and a local `notarytool` Keychain profile. Never store credentials in this repository or share them in chat.

Set `APPLE_SIGNING_IDENTITY` to the installed identity and `PRE_DRINK_NOTARY_PROFILE` to the Keychain profile name, then run:

```sh
bash scripts/release-macos.sh
```

Install `cargo-audit` 0.23.0 with `cargo install cargo-audit --version 0.23.0 --locked` before running the script.

The script installs/caches locked dependencies and calls `npm run verify:local` (all vulnerability severities, both Mac target graphs, Python/frontend/Rust tests, source guard and Clippy). Any failed prerequisite or check stops distribution. It then builds with hardened runtime, verifies signatures, submits to Apple, requires Accepted status, staples the ticket, and checks Gatekeeper. Successful output is `artifacts/release/Pre-Drink.zip`, with `SHA256SUMS` and `SOURCE_COMMIT`. The latter must identify a clean reviewed commit; run from a clean checkout. A checksum detects corruption, not a trustworthy publisher by itself. Notarization uploads the application bundle, never the user's vault.

Review current dependency advisories and complete real-export/native-drag/large-library validation before distribution. A configured minimum OS is not evidence of testing every macOS version or Intel hardware. Never tell users to disable Gatekeeper to compensate for an unsigned release.

## Landing page and download switch

The static landing page lives in `docs/index.html` and is suitable for GitHub Pages from `main:/docs`. It has no analytics, remote fonts, signup form or client script. The hosting provider still receives ordinary web requests; this is separate from the offline app.

After a signed/notarized release passes the remaining validation gates, publish its archive and SHA-256 checksum as GitHub release assets. Replace the disabled download control with that verified release asset URL, state the supported architecture/version, and update availability text and the validation record. Never point the public button to a local ad-hoc bundle or call an unsigned artifact notarized.

## Manual updates (v1)

The app makes no automatic update requests. Once public releases exist, users can explicitly visit the repository’s Releases page, read the release notes and supported architecture, and download a newer signed/notarized ZIP plus its checksum. Verify the checksum and publisher signature/Gatekeeper result before opening. Never bypass a macOS warning. Make an encrypted backup, quit Pre-Drink, replace the app in its stable install location, and reopen the existing vault. Follow any migration notes. Older versions must refuse newer schema versions; do not downgrade a migrated vault without a compatible backup.

An automatic updater is deferred until its signing keys, verification, rollback policy and opt-in network behavior are designed and tested. Do not enable a download button until the signed artifact and its checks are actually verified.

## Curation release gate and upgrade notes

Schema 4 is an additive migration. Release notes must state that existing manual notes/approvals remain, broad old raw-chat grants become shared-only, and source/curation access requires a fresh provider-bound registration. Backup before upgrading; do not downgrade a migrated vault. In-app restore/password change resets curator grants and automatic-publication policies. No personal data or generated credentials belong in release evidence.

Advertise recognized ChatGPT/Grok/Claude ZIP contracts separately from MCP-client compatibility. Fictional native/protocol writing checks do not establish model-led quality, web-chat connectivity or compatibility with every provider export variant. Codex and Grok each have a bounded three-session fictional write/resume result in CURATION_ACCEPTANCE. Complete representative local-export and broader quality checks before general compatibility claims; Claude model-led writing remains separate. Cloud curation remains optional; offline manual use requires no account, model or API key. A local-model runner is planned, not bundled.

The current schema is 9: it includes Claude scopes, durable owner holds, targeted privacy upgrades, encrypted resumable import preparation, and persistent per-client pauses with encrypted activity history. See RECOVERY and IMPORTS for upgrade and pending-import retention boundaries; older versions must refuse this schema. Advertise the recognized Grok/Claude export contracts with their remaining representative-export gate. Gemini/Copilot and optional cross-provider delegates are requested future options, not current supported features. Never use a name in a support matrix as a substitute for parser/client evidence.

## Local compiler compatibility

On macOS 27 with the pinned Rust 1.93 toolchain, a release rebuild exposed an unloadable `cssparser-macros` library. Direct loading reported a misaligned LINKEDIT string pool, matching [Rust issue 157750](https://github.com/rust-lang/rust/issues/157750). Cleaning the dependency cache did not fix it. The release profile now explicitly keeps host build dependencies unstripped (`profile.release.build-override.strip = "none"`). This affects compiler/build libraries, not runtime optimization, password derivation, encryption or Apple trust settings. Retest this workaround when upgrading the compiler; do not mask failures by weakening signing or reverting the security patch.
