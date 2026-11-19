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

The script installs locked frontend dependencies, builds, tests, checks Rust warnings, builds with hardened runtime, verifies signatures, submits to Apple, requires Accepted status, staples the ticket, and checks Gatekeeper. Successful output is `artifacts/release/Pre-Drink.zip`. Notarization uploads the application bundle, never the user's vault.

Review current dependency advisories and complete real-export/native-drag/large-library validation before distribution. A configured minimum OS is not evidence of testing every macOS version or Intel hardware. Never tell users to disable Gatekeeper to compensate for an unsigned release.

## Landing page and download switch

The static landing page lives in `docs/index.html` and is suitable for GitHub Pages from `main:/docs`. It has no analytics, remote fonts, signup form or client script. The hosting provider still receives ordinary web requests; this is separate from the offline app.

After a signed/notarized release passes the remaining validation gates, publish its archive and SHA-256 checksum as GitHub release assets. Replace the disabled download control with that verified release asset URL, state the supported architecture/version, and update availability text and the validation record. Never point the public button to a local ad-hoc bundle or call an unsigned artifact notarized.
