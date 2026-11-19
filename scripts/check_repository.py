#!/usr/bin/env python3
"""Reject user-data/credential paths and common secret patterns in tracked files.

This is a guardrail, not a substitute for manually reviewing staged content.
The scan reports filenames/rule names only, never matching secret values.
"""
import pathlib
import re
import subprocess
import sys

paths = subprocess.check_output(['git', 'ls-files', '-z']).decode().split('\0')
forbidden_suffixes = {'.zip', '.db', '.sqlite', '.sqlite3', '.pem', '.key', '.p12', '.pfx', '.log', '.partial'}
forbidden_dirs = {'exports', 'backups', 'vaults', 'private', 'local', 'node_modules', 'target', 'artifacts', '.codex', '.claude', '.cursor', '.agents'}
patterns = {
    'private key': re.compile(rb'-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----'),
    'OpenAI-style credential': re.compile(rb'\bsk-(?:proj-|svcacct-)?[A-Za-z0-9_-]{24,}'),
    'GitHub credential': re.compile(rb'\bgh[pousr]_[A-Za-z0-9]{30,}'),
    'GitHub fine-grained credential': re.compile(rb'github_pat_[A-Za-z0-9_]{40,}'),
    'AWS access key': re.compile(rb'\b(?:AKIA|ASIA)[A-Z0-9]{16}\b'),
    'local home path': re.compile(rb'/' + rb'Users/[^/\s]+/'),
}
failed = []
for name in filter(None, paths):
    path = pathlib.Path(name)
    if path.suffix.lower() in forbidden_suffixes or forbidden_dirs.intersection(path.parts) or path.name in {'.mcp.json', 'mcp.json', 'chat.html'} or path.name.startswith('.env') or re.fullmatch(r'conversations(?:-\d+)?\.json', path.name):
        failed.append((name, 'forbidden data/credential path'))
        continue
    raw = path.read_bytes()
    for rule, pattern in patterns.items():
        if pattern.search(raw):
            failed.append((name, rule))
for name, rule in failed:
    print(f'{name}: {rule}', file=sys.stderr)
if failed:
    sys.exit(1)
print('Tracked-file guard passed. Manual content review remains required.')
