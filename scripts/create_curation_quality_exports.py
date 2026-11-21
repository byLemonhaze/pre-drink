"""Package the versioned fictional quality corpus; never reads owner exports.

Only sources go into ZIPs. The separate expected/must_not rubric must stay out
of the curator prompt to avoid teaching the answer. Outputs are ignored locally.
"""
import argparse
import json
from pathlib import Path
from zipfile import ZipFile, ZIP_DEFLATED

root = Path(__file__).resolve().parents[1]
corpus = json.loads((root / "src-tauri/tests/fixtures/curation/quality-v1/cases.json").read_text())
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--output", type=Path, default=root / "artifacts/curation-quality-v1")
out = parser.parse_args().output
out.mkdir(parents=True, exist_ok=True)
for provider in ("chatgpt", "grok", "claude"):
    records = []
    for case in corpus["cases"]:
        cid = "fictional-quality-" + case["id"]
        messages = [(f"{cid}-{i}", role, text) for i, (role, text) in enumerate(case["messages"])]
        if provider == "chatgpt":
            records.append({"id": cid, "title": case["title"], "current_node": messages[-1][0], "mapping": {
                mid: {"parent": messages[i-1][0] if i else None, "message": {"author": {"role": role}, "content": {"parts": [text]}}}
                for i, (mid, role, text) in enumerate(messages)
            }})
        elif provider == "claude":
            records.append({"uuid": cid, "name": case["title"], "chat_messages": [
                {"uuid": mid, "sender": "human" if role == "user" else "assistant", "text": text}
                for mid, role, text in messages
            ]})
        else:
            records.append({"conversation": {"id": cid, "title": case["title"]}, "responses": [
                {"response": {"_id": mid, "sender": "human" if role == "user" else "assistant", "message": text,
                              "parent_response_id": messages[i-1][0] if i else None}}
                for i, (mid, role, text) in enumerate(messages)
            ]})
    target = out / f"fictional-{provider}-quality.zip"
    # Refuse overwrite so an old acceptance input cannot silently change.
    with ZipFile(target, "x", ZIP_DEFLATED) as archive:
        entry = "prod-grok-backend.json" if provider == "grok" else "conversations.json"
        archive.writestr(entry, json.dumps({"conversations": records} if provider == "grok" else records, ensure_ascii=False))
    print(f"Packaged fictional {provider} quality input")
