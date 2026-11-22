#!/bin/bash
set -euo pipefail

# --check-only performs free developer checks without requiring or using Apple
# credentials. Distribution remains a separate, explicitly configured action.
check_only=false
if [[ $# -gt 1 ]] || [[ $# -eq 1 && "$1" != "--check-only" ]]; then
  echo "Usage: bash scripts/release-macos.sh [--check-only]" >&2
  exit 2
fi
if [[ $# -eq 1 ]]; then
  check_only=true
fi
if [[ "$check_only" == false ]]; then
  : "${APPLE_SIGNING_IDENTITY:?Set your Developer ID Application signing identity}"
  : "${PRE_DRINK_NOTARY_PROFILE:?Set an existing notarytool Keychain profile name}"
  case "$APPLE_SIGNING_IDENTITY" in
    "Developer ID Application:"*) ;;
    *) echo "Public releases require a Developer ID Application identity." >&2; exit 1 ;;
  esac
fi

if [[ -n "$(git status --porcelain)" ]]; then
  echo "Release from a clean reviewed commit so SOURCE_COMMIT identifies the exact source." >&2
  exit 1
fi

npm ci
cargo fetch --manifest-path src-tauri/Cargo.toml --locked
# One authoritative gate: every vulnerability severity, both Mac dependency
# graphs, Python/frontend/Rust tests, fictional imports, formatting and Clippy.
npm run verify:local
if [[ "$check_only" == true ]]; then
  echo "Local release checks passed. No Apple signing or notarization was attempted."
  exit 0
fi
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
