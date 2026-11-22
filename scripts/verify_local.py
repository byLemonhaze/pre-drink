#!/usr/bin/env python3
"""Local checks plus current advisories; never starts Actions or models."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
from dependency_audit import AuditError, refresh as refresh_audit, validate as validate_audit

ROOT = Path(__file__).resolve().parents[1]
os.chdir(ROOT)
RECEIPT = ROOT / 'artifacts/local-verification.json'


def output(*args):
    return subprocess.check_output(args, text=True).strip()


def fingerprint():
    digest = hashlib.sha256()
    documentation = hashlib.sha256()
    paths = subprocess.check_output(['git', 'ls-files', '-z']).split(b'\0')
    for raw in sorted(filter(None, paths)):
        path = Path(os.fsdecode(raw))
        target = documentation if path.suffix == '.md' and (path.name == 'README.md' or path.parts[0] == 'docs') else digest
        target.update(raw + b'\0')
        target.update(path.read_bytes())
        target.update(b'\0' + str(path.stat().st_mode & 0o777).encode() + b'\0')
    tools = {tool: output(tool, '--version') for tool in ('node', 'npm', 'cargo', 'rustc', 'python3')}
    digest.update(json.dumps(tools, sort_keys=True).encode())
    return {'sources_sha256': digest.hexdigest(), 'documentation_sha256': documentation.hexdigest(), 'tools': tools}


def run(*args):
    print('Checking:', ' '.join(args), flush=True)
    subprocess.run(args, check=True, env={**os.environ, 'CARGO_NET_OFFLINE': 'true'})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pre-push', action='store_true')
    parser.add_argument('--audit-only', action='store_true')
    parser.add_argument('--offline', action='store_true')
    args = parser.parse_args()
    if args.audit_only:
        if args.pre_push or args.offline:
            parser.error('--audit-only cannot be combined with --pre-push or --offline')
        refresh_audit(ROOT)
        return
    if args.pre_push:
        head = output('git', 'rev-parse', 'HEAD')
        for line in sys.stdin:
            _, local_sha, _, _ = line.split()
            if set(local_sha) == {'0'}:
                continue
            if local_sha != head:
                sys.exit('Push blocked: check out and verify the exact commit being pushed.')
        if output('git', 'status', '--porcelain'):
            sys.exit('Push blocked: commit or set aside source changes, then run npm run verify:local.')
    elif subprocess.check_output(['git', 'ls-files', '--others', '--exclude-standard', '-z']):
        sys.exit('Stage new source files before verification so the receipt covers them.')
    before = fingerprint()
    # Functional checks run offline. Advisory refresh is explicit network I/O;
    # pre-push/offline reuse only recent evidence for the exact lockfiles.
    if args.pre_push or args.offline:
        validate_audit(ROOT)
    else:
        RECEIPT.unlink(missing_ok=True)
        refresh_audit(ROOT)
    if args.pre_push:
        try:
            saved = json.loads(RECEIPT.read_text())
            if saved.get('sources_sha256') == before['sources_sha256'] and saved.get('tools') == before['tools']:
                if saved.get('documentation_sha256') != before['documentation_sha256']:
                    run('npm', 'run', 'format:check')
                    run('python3', 'scripts/check_repository.py')
                    if before != fingerprint():
                        sys.exit('Sources changed during documentation checks; push blocked.')
                    RECEIPT.write_text(json.dumps(before, indent=2) + '\n')
                print('Exact-source local verification receipt matched. No remote checks started.')
                return
        except (OSError, ValueError):
            pass
        sys.exit('Push blocked: run npm run verify:local for these sources first (no paid CI).')
    RECEIPT.unlink(missing_ok=True)
    run('python3', 'scripts/check_repository.py')
    run('npm', 'run', 'format:check')
    run('npm', 'run', 'build')
    run('npm', 'run', 'test:startup')
    run('npm', 'run', 'test:reading')
    run('python3', '-m', 'unittest', 'discover', '-s', 'tests', '-p', 'test_*.py')
    run('cargo', 'test', '--manifest-path', 'src-tauri/Cargo.toml', '--locked', '--offline')
    with tempfile.TemporaryDirectory(prefix='pre-drink-fictional-check-') as directory:
        run('python3', 'scripts/create_curation_quality_exports.py', '--output', directory)
        run('cargo', 'run', '--manifest-path', 'src-tauri/Cargo.toml', '--locked', '--offline', '--example', 'verify_quality_exports', '--', directory)
    run('cargo', 'clippy', '--manifest-path', 'src-tauri/Cargo.toml', '--locked', '--offline', '--all-targets', '--', '-D', 'warnings')
    if before != fingerprint():
        sys.exit('Sources changed during verification; no receipt written. Run verification again.')
    validate_audit(ROOT)
    RECEIPT.parent.mkdir(exist_ok=True)
    RECEIPT.write_text(json.dumps(before, indent=2) + '\n')
    print('Local verification passed. Receipt contains hashes and tool versions only; no vault data.')
    print('Dependency advisories passed; outstanding warnings are listed above. Native walkthrough remains separate.')


if __name__ == '__main__':
    try:
        main()
    except subprocess.CalledProcessError as error:
        sys.exit(error.returncode)
    except AuditError as error:
        sys.exit(f'Verification blocked: {error} Run npm run audit:local, then retry.')
