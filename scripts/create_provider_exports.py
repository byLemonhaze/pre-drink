"""Package invented provider fixtures for native import acceptance; no user data."""
from pathlib import Path
from zipfile import ZIP_DEFLATED, ZipFile

root = Path(__file__).resolve().parents[1]
out = root / "artifacts" / "provider-fixtures"
out.mkdir(parents=True, exist_ok=True)
for provider, contract, entry in [
    ("grok", "account-v1", "ttl/30d/export_data/fictional/prod-grok-backend.json"),
    ("claude", "messages-v1", "conversations.json"),
]:
    fixture = root / "src-tauri" / "tests" / "fixtures" / provider / contract / "garden.input.json"
    target = out / f"fictional-{provider}-export.zip"
    with ZipFile(target, "w", ZIP_DEFLATED) as archive:
        archive.writestr(entry, fixture.read_bytes())
    print(target)
