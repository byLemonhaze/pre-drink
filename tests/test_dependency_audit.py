"""Audit gate tests use fictional reports, disposable files and no network."""
import copy
import json
from pathlib import Path
import runpy
import subprocess
import tempfile
import time
import unittest
from unittest.mock import patch

SOURCE = Path(__file__).resolve().parents[1]


class DependencyAudit(unittest.TestCase):
    def setUp(self):
        self.state = runpy.run_path(str(SOURCE / 'scripts/dependency_audit.py'))
        self.refresh = self.state['refresh']
        self.validate = self.state['validate']
        self.error = self.state['AuditError']
        self.real_target_packages = self.refresh.__globals__['target_packages']
        self.refresh.__globals__['target_packages'] = lambda root, target: {('pre-drink', '0.2.0')}
        self.tmp = tempfile.TemporaryDirectory(prefix='pre-drink-audit-test-')
        self.root = Path(self.tmp.name)
        (self.root / 'src-tauri').mkdir()
        for name in self.state['LOCKFILES']:
            (self.root / name).write_text('fictional lockfile')
        self.npm = {'auditReportVersion': 2, 'vulnerabilities': {}, 'metadata': {
            'vulnerabilities': dict.fromkeys(('info', 'low', 'moderate', 'high', 'critical', 'total'), 0)}}
        self.rust = {'vulnerabilities': {'found': False, 'count': 0, 'list': []},
                     'database': {'last-commit': 'fictional-commit', 'last-updated': 'fictional-date'},
                     'warnings': {'unmaintained': [{'advisory': {'id': 'FICTIONAL-WARNING'},
                                                   'package': {'name': 'fictional-package', 'version': '1.0.0'}}]}}
        self.calls = []

    def tearDown(self):
        self.tmp.cleanup()

    def result(self, command, **kwargs):
        self.calls.append(command)
        report = self.npm if command[0] == 'npm' else self.rust
        return subprocess.CompletedProcess(command, 0, json.dumps(report), '')

    def refresh_fixture(self, run=None):
        with patch('subprocess.check_output', return_value='cargo-audit 0.23.0\n'), \
                patch('subprocess.run', side_effect=run or self.result), \
                patch('builtins.print'):
            return self.refresh(self.root)

    def rewrite_receipt(self, **changes):
        path = self.root / 'artifacts/dependency-audit.json'
        data = json.loads(path.read_text())
        data.update(changes)
        path.write_text(json.dumps(data))

    def test_clean_reports_allow_offline_reuse_and_preserve_warnings(self):
        saved = self.refresh_fixture()
        self.assertEqual(len(self.calls), 2)
        with patch('subprocess.run', side_effect=AssertionError('Offline means no queries')), patch('builtins.print'):
            self.assertEqual(self.validate(self.root), saved)
        self.assertIn('FICTIONAL-WARNING', saved['warnings'][0])
        self.assertIn('--audit-level=low', self.calls[0])

    def test_missing_expired_and_future_evidence_are_blocked(self):
        with self.assertRaises(self.error):
            self.validate(self.root)
        for checked_at in (time.time() - 86401, time.time() + 60):
            self.refresh_fixture()
            self.rewrite_receipt(checked_at=checked_at)
            with self.assertRaises(self.error):
                self.validate(self.root)

    def test_lockfile_report_and_policy_changes_are_blocked(self):
        for name in self.state['LOCKFILES']:
            self.refresh_fixture()
            (self.root / name).write_text('different fictional dependency')
            with self.assertRaises(self.error):
                self.validate(self.root)
        self.refresh_fixture()
        (self.root / 'artifacts/dependency-audit-npm.json').write_text('{}')
        with self.assertRaises(self.error):
            self.validate(self.root)
        self.refresh_fixture()
        self.rewrite_receipt(policy=0)
        with self.assertRaises(self.error):
            self.validate(self.root)

    def test_failed_refresh_invalidates_previous_success(self):
        for failure in ('exit', 'json', 'timeout'):
            self.refresh_fixture()
            def failed(command, **kwargs):
                if failure == 'timeout':
                    raise subprocess.TimeoutExpired(command, 180)
                return subprocess.CompletedProcess(command, 1 if failure == 'exit' else 0, 'invalid', '')
            with self.assertRaises(self.error):
                self.refresh_fixture(failed)
            with self.assertRaises(self.error):
                self.validate(self.root)

    def test_reported_vulnerabilities_cannot_pass_even_with_zero_exit_status(self):
        for kind in ('npm', 'rust'):
            npm, rust = copy.deepcopy(self.npm), copy.deepcopy(self.rust)
            if kind == 'npm':
                self.npm['metadata']['vulnerabilities'].update(low=1, total=1)
            else:
                self.rust['vulnerabilities'].update(found=True, count=1, list=[{}])
            with self.assertRaises(self.error):
                self.refresh_fixture()
            self.npm, self.rust = npm, rust

    def test_incomplete_reports_and_wrong_auditor_are_blocked(self):
        self.rust = {}
        with self.assertRaises(self.error):
            self.refresh_fixture()
        with patch('subprocess.check_output', return_value='cargo-audit 0.1.0'), self.assertRaises(self.error):
            self.refresh(self.root)

    def test_lockfile_change_during_query_is_blocked(self):
        def changed(command, **kwargs):
            (self.root / 'package-lock.json').write_text('changed during audit')
            return self.result(command, **kwargs)
        with self.assertRaises(self.error):
            self.refresh_fixture(changed)

    def test_unsound_warning_blocks_only_when_its_affected_version_is_in_mac_graph(self):
        self.rust['warnings']['unsound'] = [{'advisory': {'id': 'FICTIONAL-UNSOUND'},
                                            'package': {'name': 'fictional-package', 'version': '1.0.0'}}]
        self.refresh_fixture()
        self.refresh.__globals__['target_packages'] = lambda root, target: {('fictional-package', '2.0.0')}
        self.refresh_fixture()
        # This also exercises revalidation of a previously successful receipt.
        self.refresh.__globals__['target_packages'] = lambda root, target: {('fictional-package', '1.0.0')}
        with self.assertRaises(self.error):
            self.validate(self.root)
        with self.assertRaises(self.error):
            self.refresh_fixture()
        self.assertFalse((self.root / 'artifacts/dependency-audit.json').exists())

    def test_target_graph_parser_and_failed_resolution_do_not_claim_absence(self):
        valid = 'pre-drink v0.2.0 (/fictional/source)\nfictional-package v1.0.0\nfictional-package v1.0.0 (*)\n'
        result = subprocess.CompletedProcess([], 0, valid, '')
        with patch('subprocess.run', return_value=result) as run:
            packages = self.real_target_packages(self.root, 'aarch64-apple-darwin')
            self.assertEqual(packages, {('pre-drink', '0.2.0'), ('fictional-package', '1.0.0')})
            self.assertIn('--offline', run.call_args.args[0])
            self.assertIn('normal,build', run.call_args.args[0])
        for status, text in ((1, valid), (0, ''), (0, 'unexpected output'), (0, 'fictional-package v1.0.0')):
            with patch('subprocess.run', return_value=subprocess.CompletedProcess([], status, text, '')), self.assertRaises(self.error):
                self.real_target_packages(self.root, 'aarch64-apple-darwin')


if __name__ == '__main__':
    unittest.main()
