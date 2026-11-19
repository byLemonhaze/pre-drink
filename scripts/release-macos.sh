#!/bin/bash
set -euo pipefail

# Run from the repository root after configuring your own Developer ID and
# notarytool Keychain profile. Never paste signing credentials into this file.
: "${APPLE_SIGNING_IDENTITY:?Set your Developer ID Application signing identity}"
: "${PRE_DRINK_NOTARY_PROFILE:?Set an existing notarytool Keychain profile name}"
case "$APPLE_SIGNING_IDENTITY" in
  "Developer ID Application:"*) ;;
  *) echo "Public releases require a Developer ID Application identity." >&2; exit 1 ;;
esac

if [[ -n "$(git status --porcelain)" ]]; then
  echo "Release from a clean reviewed commit so SOURCE_COMMIT identifies the exact source." >&2
  exit 1
fi

npm ci
npm run format:check
npm run build
npm audit --audit-level=high
cargo audit --file src-tauri/Cargo.lock
python3 scripts/check_repository.py
cargo test --manifest-path src-tauri/Cargo.toml --locked
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
npm run tauri -- build -- --locked

app_path="src-tauri/target/release/bundle/macos/Pre-Drink.app"
release_dir="artifacts/release"
mkdir -p "$release_dir"
codesign --verify --deep --strict --verbose=2 "$app_path"
ditto -c -k --keepParent "$app_path" "$release_dir/Pre-Drink-for-notarization.zip"
xcrun notarytool submit "$release_dir/Pre-Drink-for-notarization.zip" \
  --keychain-profile "$PRE_DRINK_NOTARY_PROFILE" --wait --output-format json \
  > "$release_dir/notarization.json"
python3 - "$release_dir/notarization.json" <<'PY'
import json, sys
result = json.load(open(sys.argv[1]))
if result.get("status") != "Accepted":
    raise SystemExit("Apple did not accept this build. Inspect the notarization result before distributing.")
PY
xcrun stapler staple "$app_path"
xcrun stapler validate "$app_path"
spctl --assess --type execute --verbose=2 "$app_path"
ditto -c -k --keepParent "$app_path" "$release_dir/Pre-Drink.zip"
(cd "$release_dir" && shasum -a 256 Pre-Drink.zip > SHA256SUMS)
git rev-parse HEAD > "$release_dir/SOURCE_COMMIT"
echo "Signed, notarized, and verified: $release_dir/Pre-Drink.zip"
