#!/usr/bin/env python3
"""Package reviewed fictional parser fixtures for native import acceptance checks."""
import argparse
import os
from pathlib import Path
from zipfile import ZipFile, ZIP_DEFLATED


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True,
                        help="Local folder for generated fictional ZIPs; never commit it")
    args = parser.parse_args()
    fixtures = Path(__file__).resolve().parent.parent / "src-tauri/tests/fixtures/chatgpt/mapping-v1"
    args.output.mkdir(mode=0o700, parents=True, exist_ok=True)
    archives = {
        "fictional-chatgpt.zip": [
            ("conversations-000.json", (fixtures / "branched.input.json").read_bytes()),
            ("nested/conversations-001.json", (fixtures / "mixed-content.input.json").read_bytes()),
        ],
        "fictional-invalid.zip": [
            ("conversations-000.json", (fixtures / "fallback.input.json").read_bytes()),
            ("conversations-001.json", b"invalid JSON"),
        ],
    }
    for name, entries in archives.items():
        path = args.output / name
        try:
            fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        except FileExistsError:
            parser.error(f"{name} already exists; choose a fresh output folder")
        with os.fdopen(fd, "wb") as output:
            with ZipFile(output, "w", ZIP_DEFLATED) as archive:
                for entry, content in entries:
                    archive.writestr(entry, content)
        print(path)


if __name__ == "__main__":
    main()
