#!/usr/bin/env python3
"""Install this repository's guard without replacing other configured hooks."""
from pathlib import Path
import subprocess
import sys

root = Path(__file__).resolve().parents[1]
existing = subprocess.run(['git', 'config', '--get', 'core.hooksPath'], cwd=root, text=True, capture_output=True)
if existing.returncode == 0 and existing.stdout.strip() != '.githooks':
    sys.exit('Existing hooks configuration found; integrate the pre-push guard manually.')
old = Path(subprocess.check_output(['git', 'rev-parse', '--git-path', 'hooks'], cwd=root, text=True).strip())
if not old.is_absolute():
    old = root / old
if existing.returncode != 0 and old.exists() and any(p.is_file() and not p.name.endswith('.sample') for p in old.iterdir()):
    sys.exit('Existing local hooks found; integrate the pre-push guard manually.')
subprocess.run(['git', 'config', '--local', 'core.hooksPath', '.githooks'], cwd=root, check=True)
print('Repository pre-push guard installed. No account settings or remote jobs changed.')
