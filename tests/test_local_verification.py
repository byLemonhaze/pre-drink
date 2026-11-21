"""Exercise the local pre-push gate using disposable repositories, never remotes."""
import contextlib
import io
import json
import os
from pathlib import Path
import runpy
import subprocess
import tempfile
import unittest
from unittest.mock import patch

SOURCE = Path(__file__).resolve().parents[1]


class VerificationGuard(unittest.TestCase):
    def setUp(self):
        self.previous = Path.cwd()
        namespace = runpy.run_path(str(SOURCE / 'scripts/verify_local.py'))
        self.main = namespace['main']
        self.state = self.main.__globals__
        self.tmp = tempfile.TemporaryDirectory(prefix='pre-drink-hook-test-')
        self.root = Path(self.tmp.name)
        os.chdir(self.root)
        self.git('init', '-q')
        self.git('config', 'user.name', 'Fictional verification')
        self.git('config', 'user.email', 'fictional@example.invalid')
        (self.root / 'code.txt').write_text('fictional source\n')
        (self.root / 'README.md').write_text('Fictional documentation\n')
        self.commit()
        self.receipt = self.root / 'artifacts/local-verification.json'
        self.receipt.parent.mkdir()
        self.state['RECEIPT'] = self.receipt
        self.receipt.write_text(json.dumps(self.state['fingerprint']()))
        (self.root / '.git/info/exclude').write_text('artifacts/\n')
        self.checks = []
        self.state['run'] = lambda *args: self.checks.append(args)

    def tearDown(self):
        os.chdir(self.previous)
        self.tmp.cleanup()

    def git(self, *args):
        return subprocess.check_output(['git', *args], text=True).strip()

    def commit(self):
        self.git('add', 'code.txt', 'README.md')
        self.git('commit', '-qm', 'Fictional source checkpoint')

    def push(self, sha=None):
        sha = sha or self.git('rev-parse', 'HEAD')
        line = f'refs/heads/main {sha} refs/heads/main {"0" * 40}\n'
        with patch('sys.argv', ['verify_local.py', '--pre-push']), patch('sys.stdin', io.StringIO(line)), contextlib.redirect_stdout(io.StringIO()):
            self.main()

    def test_exact_receipt_reuses_checks(self):
        self.push()
        self.assertEqual(self.checks, [])

    def test_changed_implementation_requires_full_verification(self):
        (self.root / 'code.txt').write_text('changed fictional code\n')
        self.commit()
        with self.assertRaises(SystemExit):
            self.push()

    def test_documentation_only_runs_format_and_source_guard(self):
        (self.root / 'README.md').write_text('Updated fictional evidence\n')
        self.commit()
        self.push()
        self.assertEqual(self.checks, [('npm', 'run', 'format:check'), ('python3', 'scripts/check_repository.py')])
        self.assertEqual(json.loads(self.receipt.read_text()), self.state['fingerprint']())

    def test_dirty_tree_and_other_commits_are_rejected(self):
        with self.assertRaises(SystemExit):
            self.push('1' * 40)
        (self.root / 'untracked.txt').write_text('unverified source')
        with self.assertRaises(SystemExit):
            self.push()

    def test_missing_receipt_is_not_a_pass(self):
        self.receipt.unlink()
        with self.assertRaises(SystemExit):
            self.push()


if __name__ == '__main__':
    unittest.main()
