"""Exercise release routing with fake commands; never call Apple or registries."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / 'scripts/release-macos.sh'


class ReleaseGate(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='pre-drink-release-test-')
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.bin = self.root / 'bin'
        self.bin.mkdir()
        self.log = self.root / 'commands'
        fake = '''#!/bin/sh
name=${0##*/}
printf '%s %s\\n' "$name" "$*" >> "$FICTIONAL_COMMAND_LOG"
if [ "$name" = git ] && [ "$1" = status ] && [ "${FICTIONAL_DIRTY:-}" = yes ]; then
  printf ' M fictional-source\\n'
fi
if [ "$name" = npm ] && [ "$*" = "${FICTIONAL_FAIL_NPM:-}" ]; then
  exit 7
fi
if [ "$name" = cargo ] && [ "${FICTIONAL_FAIL_CARGO:-}" = yes ]; then
  exit 7
fi
case "$name" in
  codesign|xcrun|spctl|ditto) exit 99 ;;
esac
exit 0
'''
        for name in ('git', 'npm', 'cargo', 'codesign', 'xcrun', 'spctl', 'ditto'):
            path = self.bin / name
            path.write_text(fake)
            path.chmod(0o700)
        self.env = {k: v for k, v in os.environ.items()
                    if not k.startswith(('APPLE_', 'TAURI_SIGNING_'))
                    and k != 'PRE_DRINK_NOTARY_PROFILE'}
        self.env.update(PATH=f'{self.bin}:/usr/bin:/bin', FICTIONAL_COMMAND_LOG=str(self.log))

    def run_script(self, *args):
        result = subprocess.run(['/bin/bash', str(SCRIPT), *args], cwd=self.root,
                                env=self.env, capture_output=True, text=True, timeout=10)
        calls = self.log.read_text().splitlines() if self.log.exists() else []
        return result, calls

    def assert_no_distribution(self, calls):
        self.assertFalse(any(line.startswith(('codesign ', 'xcrun ', 'spctl ', 'ditto '))
                             or line.startswith('npm run tauri') for line in calls))

    def test_check_only_uses_complete_gate_without_apple_credentials(self):
        result, calls = self.run_script('--check-only')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(calls, ['git status --porcelain', 'npm ci', 'cargo fetch --manifest-path src-tauri/Cargo.toml --locked', 'npm run verify:local'])
        self.assertIn('No Apple signing or notarization', result.stdout)
        self.assert_no_distribution(calls)

    def test_check_only_never_reaches_apple_even_when_credentials_are_configured(self):
        self.env.update(APPLE_SIGNING_IDENTITY='Developer ID Application: Fictional',
                        PRE_DRINK_NOTARY_PROFILE='fictional profile')
        result, calls = self.run_script('--check-only')
        self.assertEqual(result.returncode, 0)
        self.assert_no_distribution(calls)

    def test_dependency_or_full_gate_failure_stops_distribution(self):
        self.env.update(APPLE_SIGNING_IDENTITY='Developer ID Application: Fictional',
                        PRE_DRINK_NOTARY_PROFILE='fictional profile')
        for failed in ('ci', 'run verify:local'):
            for args in ((), ('--check-only',)):
                with self.subTest(failed=failed, args=args):
                    self.log.unlink(missing_ok=True)
                    self.env['FICTIONAL_FAIL_NPM'] = failed
                    result, calls = self.run_script(*args)
                    self.assertEqual(result.returncode, 7)
                    self.assert_no_distribution(calls)
                    if failed == 'ci':
                        self.assertNotIn('npm run verify:local', calls)

    def test_missing_locked_dependency_stops_before_verification_and_distribution(self):
        self.env['FICTIONAL_FAIL_CARGO'] = 'yes'
        result, calls = self.run_script('--check-only')
        self.assertEqual(result.returncode, 7)
        self.assertNotIn('npm run verify:local', calls)
        self.assert_no_distribution(calls)

    def test_dirty_tree_and_unknown_arguments_stop_before_installation(self):
        result, calls = self.run_script('--unexpected')
        self.assertEqual(result.returncode, 2)
        self.assertEqual(calls, [])
        self.env['FICTIONAL_DIRTY'] = 'yes'
        result, calls = self.run_script('--check-only')
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(calls, ['git status --porcelain'])

    def test_normal_release_still_requires_developer_id_configuration(self):
        result, calls = self.run_script()
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(calls, [])
        self.env.update(APPLE_SIGNING_IDENTITY='-', PRE_DRINK_NOTARY_PROFILE='fictional profile')
        result, calls = self.run_script()
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(calls, [])


if __name__ == '__main__':
    unittest.main()
