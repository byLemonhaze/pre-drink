"""Generate a synthetic ChatGPT export ZIP for local smoke testing. No real chat data."""
import json
from pathlib import Path
from zipfile import ZipFile, ZIP_DEFLATED

root = Path(__file__).resolve().parent.parent
output = root / "artifacts" / "sample-chatgpt-export.zip"
output.parent.mkdir(exist_ok=True)

def conversation(identifier, title, question, answer, alternate=None):
    mapping = {
        "root": {"parent": None, "message": None},
        "user": {"parent": "root", "message": {"author": {"role": "user"}, "content": {"content_type": "text", "parts": [question]}, "create_time": 1795000000}},
        "assistant": {"parent": "user", "message": {"author": {"role": "assistant"}, "content": {"content_type": "text", "parts": [answer]}, "create_time": 1795000010}},
    }
    if alternate:
        mapping["alternate"] = {"parent": "user", "message": {"author": {"role": "assistant"}, "content": {"parts": [alternate]}, "create_time": 1795000005}}
    return {"id": identifier, "title": title, "create_time": 1795000000, "update_time": 1795000010, "current_node": "assistant", "mapping": mapping}

data = [
    conversation("sample-1", "A quieter kind of workspace", "Help me design a calm reading nook.", "## Make room for a slower thought\n\nStart with a **walnut bookshelf**, a warm lamp, and a comfortable chair.\n\n- Keep the surface clear.\n- Choose one book for the week.\n- Let natural light do most of the work.\n\n> A good space makes it easier to begin.", "A mahogany bookshelf could anchor the room."),
    conversation("sample-2", "The idea behind Pre-Drink", "What should this little app stand for?", "Your AI history, sober and ready.\n\nA local home for scattered thoughts. Start with a searchable library; build toward a deliberate memory layer."),
    conversation("sample-sensitive", "A contact to remember (privacy example)", "My email is sample@example.org.", "This synthetic conversation should be hidden by the default privacy filter."),
]
with ZipFile(output, "w", ZIP_DEFLATED) as archive:
    archive.writestr("conversations.json", json.dumps(data))
print(output)
