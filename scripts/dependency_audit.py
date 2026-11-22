"""Free advisory checks and a short-lived, lockfile-bound offline receipt.

These developer checks send package metadata to npm/RustSec, never vault data.
The local receipt is a guard against stale checks, not a signed attestation.
"""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time

POLICY = 1
MAX_AGE = 24 * 60 * 60
AUDITOR_VERSION = '0.23.0'
LOCKFILES = ('package-lock.json', 'src-tauri/Cargo.lock')


class AuditError(RuntimeError):
    pass


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def lock_hashes(root):
    return {name: digest(root / name) for name in LOCKFILES}


def assess(npm, rust):
    """Reject failures, incomplete reports and every reported vulnerability."""
    try:
        npm_counts = npm['metadata']['vulnerabilities']
        if (npm.get('error') or npm['auditReportVersion'] != 2
                or any(npm_counts[name] != 0 for name in
                       ('info', 'low', 'moderate', 'high', 'critical', 'total'))
                or npm['vulnerabilities'] != {}):
            raise AuditError('npm audit reported vulnerabilities or an incomplete result.')
        vulnerabilities = rust['vulnerabilities']
        if (rust.get('error') or vulnerabilities['found'] is not False
                or vulnerabilities['count'] != 0 or vulnerabilities['list'] != []):
            raise AuditError('Rust audit reported vulnerabilities or an incomplete result.')
        database = rust['database']
        if not database['last-commit'] or not database['last-updated']:
            raise AuditError('Rust audit did not identify its advisory database.')
        warnings = sorted(
            f"{kind}: {item['advisory']['id']} ({item['package']['name']})"
            for kind, items in rust['warnings'].items() for item in items
        )
        return {'rustsec_commit': database['last-commit'], 'warnings': warnings}
    except (KeyError, TypeError, AttributeError) as error:
        raise AuditError('Dependency audit returned an incomplete report.') from error


def refresh(root):
    root = Path(root)
    directory = root / 'artifacts'
    directory.mkdir(exist_ok=True)
    receipt = directory / 'dependency-audit.json'
    receipt.unlink(missing_ok=True)
    before = lock_hashes(root)
    auditor = shutil.which('cargo-audit') or str(directory / 'audit-tools/bin/cargo-audit')
    try:
        version = subprocess.check_output([auditor, '--version'], text=True).strip()
    except (OSError, subprocess.CalledProcessError) as error:
        raise AuditError(
            'Install the pinned free audit tool first: cargo install cargo-audit '
            '--version 0.23.0 --locked --root artifacts/audit-tools'
        ) from error
    if version != f'cargo-audit {AUDITOR_VERSION}':
        raise AuditError(f'Expected cargo-audit {AUDITOR_VERSION}; found {version}.')
    reports = {}
    commands = {
        'npm': ['npm', 'audit', '--json', '--audit-level=low'],
        'rust': [auditor, 'audit', '--file', 'src-tauri/Cargo.lock', '--json'],
    }
    for name, command in commands.items():
        print(f'Checking current {name} advisories (free package-metadata query).', flush=True)
        try:
            result = subprocess.run(command, cwd=root, capture_output=True, text=True,
                                    timeout=180, env={**os.environ, 'CARGO_NET_OFFLINE': 'false'})
        except (OSError, subprocess.TimeoutExpired) as error:
            raise AuditError(f'{name} advisory query failed; no audit receipt written.') from error
        path = directory / f'dependency-audit-{name}.json'
        path.write_text(result.stdout)
        if result.returncode:
            raise AuditError(f'{name} audit failed. Inspect {path.name}; no audit receipt written.')
        try:
            reports[name] = json.loads(result.stdout)
        except ValueError as error:
            raise AuditError(f'{name} audit returned invalid JSON.') from error
    summary = assess(reports['npm'], reports['rust'])
    if before != lock_hashes(root):
        raise AuditError('Lockfiles changed during the audit; run it again.')
    saved = {
        'policy': POLICY, 'checked_at': time.time(), 'lockfiles': before,
        'auditor': version,
        'reports': {name: digest(directory / f'dependency-audit-{name}.json') for name in commands},
        **summary,
    }
    receipt.write_text(json.dumps(saved, indent=2) + '\n')
    return validate(root)


def validate(root):
    """Offline and pre-push checks must have recent, unchanged audit evidence."""
    root = Path(root)
    directory = root / 'artifacts'
    try:
        saved = json.loads((directory / 'dependency-audit.json').read_text())
        age = time.time() - saved['checked_at']
        if (saved['policy'] != POLICY or not 0 <= age <= MAX_AGE
                or saved['lockfiles'] != lock_hashes(root)
                or saved['auditor'] != f'cargo-audit {AUDITOR_VERSION}'):
            raise AuditError('Dependency audit is stale or does not match these lockfiles.')
        reports = {}
        for name in ('npm', 'rust'):
            path = directory / f'dependency-audit-{name}.json'
            if saved['reports'][name] != digest(path):
                raise AuditError('Dependency audit report changed after verification.')
            reports[name] = json.loads(path.read_text())
        summary = assess(reports['npm'], reports['rust'])
        if any(saved[key] != value for key, value in summary.items()):
            raise AuditError('Dependency audit summary does not match its reports.')
    except (OSError, ValueError, KeyError, TypeError) as error:
        raise AuditError('Dependency audit evidence is missing or invalid.') from error
    print('Current lockfiles passed the vulnerability audit (receipt valid for 24 hours).')
    for warning in summary['warnings']:
        print(f'Outstanding dependency warning: {warning}')
    return saved
