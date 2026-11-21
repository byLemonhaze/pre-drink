#!/usr/bin/env python3
"""Opt-in installed Codex acceptance against curation_acceptance_fixture only.

Uses existing ChatGPT authentication, never an API key. At most three model
invocations per disposable fixture, 180 seconds each. Private transcripts are
kept inside that fixture directory, never printed or written into this repo.
"""
import argparse
from curation_prompts import common, prompts
import fcntl
import json
import os
from pathlib import Path
import selectors
import signal
import stat
import subprocess
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("config", type=Path)
parser.add_argument("phase", choices=["draft", "resume", "retrieve"])
parser.add_argument("--foreign-source", action="append", default=[])
parser.add_argument("--allow-subscription-usage", action="store_true")
args = parser.parse_args()
if not args.allow_subscription_usage:
    parser.error("Explicit --allow-subscription-usage is required; this can consume ChatGPT allowance.")
config_path = args.config.resolve(strict=True)
root = config_path.parent
if (root.parent != Path("/private/tmp") or not root.name.startswith("pd-curator-client-")
        or config_path.name not in {"curator.json", "reader.json"}):
    parser.error("Only the disposable writing-acceptance fixture is accepted.")
for path in [root, config_path]:
    if path.stat().st_uid != os.getuid() or stat.S_IMODE(path.stat().st_mode) & 0o077:
        parser.error("Fixture and configuration must be owner-only.")
if (args.phase == "retrieve") != (config_path.name == "reader.json"):
    parser.error("Retrieval must use reader.json; writing must use curator.json.")
config = json.loads(config_path.read_text())
if config.get("fixture_provider") != "chatgpt":
    parser.error("Codex acceptance requires the ChatGPT-scoped fixture.")
if set(config["env"]) != {"PRE_DRINK_SOCKET", "PRE_DRINK_CLIENT_ID", "PRE_DRINK_CLIENT_TOKEN"}:
    parser.error("Unexpected fixture configuration.")
if Path(config["env"]["PRE_DRINK_SOCKET"]).resolve() != root / "agents.sock":
    parser.error("Socket must belong to this fixture.")
if config["args"] != ["--mcp"]:
    parser.error("Unexpected bridge arguments.")
env = {k: v for k, v in os.environ.items()
       if k not in {"OPENAI_API_KEY", "CODEX_API_KEY", "OPENAI_BASE_URL"}
       and not k.startswith("PRE_DRINK_")}
status = subprocess.run(["codex", "login", "status"], env=env, capture_output=True, text=True, timeout=15)
if status.returncode or "Logged in using ChatGPT" not in status.stdout + status.stderr:
    parser.error("ChatGPT sign-in required; API-key authentication is not accepted.")
count_path = root / "model-invocations"
prompt = common + prompts[args.phase]
if args.phase == "draft":
    if len(args.foreign_source) != 2:
        parser.error("Supply the two foreign source IDs from the fixture startup response.")
    prompt += "\nForeign IDs for explicit denial checks: " + json.dumps(args.foreign_source)

command = ["codex", "exec", "--ignore-user-config", "--ephemeral", "--skip-git-repo-check",
           "--json", "--color", "never", "-s", "read-only", "-C", str(root / "work")]
overrides = {
    "forced_login_method": "chatgpt", "approval_policy": "never", "web_search": "disabled",
    "project_doc_max_bytes": 0, "features.shell_tool": False, "features.apps": False,
    "features.plugins": False, "features.hooks": False, "features.multi_agent": False,
    "features.multi_agent_v2": False, "features.browser_use": False,
    "features.computer_use": False, "features.in_app_browser": False,
    "features.memories": False, "features.artifact": False,
    "mcp_servers.pre-drink.command": config["command"],
    "mcp_servers.pre-drink.args": config["args"],
    "mcp_servers.pre-drink.env_vars": list(config["env"]),
    "mcp_servers.pre-drink.enabled_tools": ["curate_memory", "get_projects", "get_context_pack"],
    "mcp_servers.pre-drink.required": True,
}
for key, value in overrides.items():
    command += ["-c", key + "=" + json.dumps(value)]
command += ["-"]
env.update(config["env"])
# Keep each attempt accountable, including interrupted or failed invocations.
fd = os.open(count_path, os.O_RDWR | os.O_CREAT, 0o600)
with os.fdopen(fd, "r+") as f:
    fcntl.flock(f, fcntl.LOCK_EX)
    count = int(f.read() or "0")
    if count >= 3:
        parser.error("Three-invocation limit reached for this fixture. No model call made.")
    f.seek(0)
    f.write(str(count + 1))
    f.truncate()
    f.flush()
log_path = root / (args.phase + ".jsonl")
fd = os.open(log_path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
calls, events, usage = [], [], None
with os.fdopen(fd, "wb") as log:
    process = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                               stderr=log, env=env, start_new_session=True)
    selector = selectors.DefaultSelector()
    try:
        process.stdin.write(prompt.encode())
        process.stdin.close()
        selector.register(process.stdout, selectors.EVENT_READ)
        deadline = time.monotonic() + 180
        pending = b""
        while True:
            if time.monotonic() > deadline:
                raise RuntimeError("Acceptance call exceeded its 180-second limit")
            ready = selector.select(1)
            if not ready:
                continue
            chunk = os.read(process.stdout.fileno(), 65536)
            if not chunk:
                break
            log.write(chunk)
            log.flush()
            pending += chunk
            while b"\n" in pending:
                line, pending = pending.split(b"\n", 1)
                try:
                    event = json.loads(line)
                except (ValueError, UnicodeDecodeError):
                    continue
                events.append(event)
                if event.get("type") == "turn.completed":
                    usage = event.get("usage")
                item = event.get("item", {})
                if event.get("type") == "item.completed" and item.get("type") == "mcp_tool_call":
                    if item.get("server") != "pre-drink":
                        raise RuntimeError("Unexpected MCP server called")
                    calls.append(item)
                    if len(calls) > 25:
                        raise RuntimeError("Acceptance exceeded its 25-tool-call limit")
                if event.get("type") == "item.completed" and item.get("type") not in {
                        "agent_message", "reasoning", "mcp_tool_call"}:
                    raise RuntimeError("Unexpected non-vault tool or item executed")
        code = process.wait(timeout=5)
        print(json.dumps({"phase": args.phase, "exit_code": code, "completed_mcp_calls": len(calls),
                          "tools": [c.get("tool") for c in calls], "usage": usage,
                          "private_log": str(log_path)}))
        if code or not calls or usage is None:
            raise RuntimeError("Client acceptance did not complete; inspect the private fixture log")
    finally:
        selector.close()
        if process.poll() is None:
            os.killpg(process.pid, signal.SIGTERM)
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait(timeout=5)
