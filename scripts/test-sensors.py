#!/usr/bin/env python3
"""Offline regression fixtures. Run real entrypoints in isolated miniature repos."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent


class Sensors(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        shutil.copytree(ROOT / 'scripts', self.root / 'scripts', ignore=shutil.ignore_patterns('__pycache__'))
        for name in ('web/pkg', 'src/core', 'src/utils', 'src/render', 'src/ui', 'bin'):
            (self.root / name).mkdir(parents=True)
        for name in ('pnpm-lock.yaml', 'web/pnpm-lock.yaml'):
            (self.root / name).write_text('lockfileVersion: 9\n')
        (self.root / 'src/core/mod.rs').write_text('pub struct Grid;\n')
        self.env = dict(os.environ, PATH=f'{self.root / "bin"}:/usr/bin:/bin')

    def npm_audit(self):
        # Literal command lists keep fixtures free of shell/PATH taint; the
        # temp repo is selected via cwd only.
        return subprocess.run(['bash', 'scripts/npm-audit.sh'],
                              cwd=self.root, env=self.env, text=True, capture_output=True)

    def architecture(self):
        return subprocess.run(['bash', 'scripts/check-architecture.sh'],
                              cwd=self.root, env=self.env, text=True, capture_output=True)

    def loc(self):
        return subprocess.run(['bash', 'scripts/check-loc.sh'],
                              cwd=self.root, env=self.env, text=True, capture_output=True)

    def bandit(self, payload, status=0, version='bandit 1.9.4'):
        (self.root / 'bandit-payload').write_text(payload)
        (self.root / 'bandit-status').write_text(str(status))
        (self.root / 'bandit-version').write_text(version)
        executable = self.root / 'bin/bandit'
        executable.write_text('''#!/usr/bin/python3
import pathlib, sys
cwd = pathlib.Path.cwd()
if '--version' in sys.argv:
    sys.stdout.write((cwd / 'bandit-version').read_text() + '\\n')
    sys.exit(0)
with (cwd / 'bandit-calls').open('a') as log:
    log.write(' '.join(sys.argv[1:]) + '\\n')
sys.stdout.write((cwd / 'bandit-payload').read_text())
sys.exit(int((cwd / 'bandit-status').read_text()))
''')
        executable.chmod(0o755)

    def run_bandit(self):
        return subprocess.run(['python3', 'scripts/bandit-check.py'],
                              cwd=self.root, env=self.env, text=True, capture_output=True)

    def actionlint(self, payload, status=0, version='1.6.26'):
        (self.root / 'actionlint-payload').write_text(payload)
        (self.root / 'actionlint-status').write_text(str(status))
        (self.root / 'actionlint-version').write_text(version)
        executable = self.root / 'bin/actionlint'
        executable.write_text('''#!/usr/bin/python3
import pathlib, sys
cwd = pathlib.Path.cwd()
if '--version' in sys.argv:
    sys.stdout.write((cwd / 'actionlint-version').read_text() + '\\n')
    sys.exit(0)
with (cwd / 'actionlint-calls').open('a') as log:
    log.write(' '.join(sys.argv[1:]) + '\\n')
sys.stdout.write((cwd / 'actionlint-payload').read_text())
sys.exit(int((cwd / 'actionlint-status').read_text()))
''')
        executable.chmod(0o755)

    def run_actionlint(self):
        return subprocess.run(['python3', 'scripts/actionlint-check.py'],
                              cwd=self.root, env=self.env, text=True, capture_output=True)

    @staticmethod
    def actionlint_finding(kind='shellcheck', filepath='.github/workflows/x.yml',
                           line=1, column=1, message='finding'):
        return {'message': message, 'filepath': filepath, 'line': line,
                'column': column, 'kind': kind, 'snippet': 'x', 'end_column': column + 1}

    @staticmethod
    def bandit_report(*severities, errors=()):
        results = [{'filename': 'scripts/x.py', 'line_number': index + 1,
                    'test_id': 'B603', 'issue_severity': severity,
                    'issue_confidence': 'HIGH', 'issue_text': 'finding'}
                   for index, severity in enumerate(severities)]
        return {'errors': list(errors), 'generated_at': 'now', 'metrics': {}, 'results': results}

    def assert_status(self, result, expected):
        self.assertEqual(result.returncode, expected, result.stdout + result.stderr)

    def pnpm(self, root, web=None, root_rc=0, web_rc=None):
        responses = {'root': [root, root_rc], 'web': [web if web is not None else root,
                                                   root_rc if web_rc is None else web_rc]}
        (self.root / 'responses.json').write_text(json.dumps(responses))
        executable = self.root / 'bin/pnpm'
        executable.write_text('''#!/usr/bin/python3
import json, pathlib, sys
cwd = pathlib.Path.cwd()
root = cwd.parent if cwd.name == 'web' else cwd
label = 'web' if cwd.name == 'web' else 'root'
with (root / 'calls').open('a') as log:
    log.write(label + ' ' + ' '.join(sys.argv[1:]) + '\\n')
response, status = json.loads((root / 'responses.json').read_text())[label]
print(json.dumps(response))
sys.exit(status)
''')
        executable.chmod(0o755)

    @staticmethod
    def audit(high=0):
        return {'advisories': {'123': {'title': 'network registry.npmjs.org exposure'}} if high else {},
                'metadata': {'vulnerabilities': {'info': 0, 'low': 0, 'moderate': 0,
                                                 'high': high, 'critical': 0}}}

    def test_audit_clean_requires_both_and_json(self):
        self.pnpm(self.audit())
        self.assert_status(self.npm_audit(), 0)
        calls = (self.root / 'calls').read_text().splitlines()
        self.assertEqual(len(calls), 2)
        self.assertTrue(all('--json' in call for call in calls))

    def test_audit_high_network_prose_is_not_offline(self):
        self.pnpm(self.audit(1), root_rc=1)
        self.assert_status(self.npm_audit(), 1)

    def test_audit_registry_failure_is_unverified(self):
        self.pnpm({'error': {'code': 'ENOTFOUND'}}, root_rc=1)
        self.assert_status(self.npm_audit(), 2)

    def test_audit_partial_is_unverified(self):
        self.pnpm(self.audit(), {'error': {'code': 'ETIMEDOUT'}}, web_rc=1)
        self.assert_status(self.npm_audit(), 2)
        self.assertEqual(len((self.root / 'calls').read_text().splitlines()), 2)

    def test_audit_missing_tool_is_unverified(self):
        self.assert_status(self.npm_audit(), 2)

    def test_audit_missing_lockfile_is_unverified(self):
        self.pnpm(self.audit())
        for name in ('web/pnpm-lock.yaml', 'pnpm-lock.yaml'):
            with self.subTest(name=name):
                (self.root / name).unlink()
                self.assert_status(self.npm_audit(), 2)
                (self.root / name).write_text('lockfileVersion: 9\n')

    def test_audit_malformed_response_is_unverified(self):
        self.pnpm({'metadata': {'vulnerabilities': {}}})
        self.assert_status(self.npm_audit(), 2)

    def artifact(self, data):
        if data is not None:
            (self.root / 'web/pkg/ascii_canvas_bg.wasm').write_bytes(data)
        return subprocess.run(['node', 'scripts/check-artifact.mjs'],
                              cwd=self.root, text=True, capture_output=True)

    def test_artifact_missing(self):
        self.assertNotEqual(self.artifact(None).returncode, 0)

    def test_artifact_invalid(self):
        for data in (b'', b'not wasm', b'\0asm\1\0\0\0\xff'):
            with self.subTest(data=data):
                self.assertNotEqual(self.artifact(data).returncode, 0)

    def test_artifact_valid(self):
        self.assert_status(self.artifact(b'\0asm\1\0\0\0'), 0)

    def test_artifact_oversize(self):
        # Valid module with one large custom section (empty name + payload).
        result = self.artifact(b'\0asm\1\0\0\0\0\x80\x80\x60' + bytes(1572864))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('exceeds', result.stdout + result.stderr)

    def test_architecture_allowed(self):
        (self.root / 'src/core/mod.rs').write_text('// crate::wasm::Bad\nconst S: &str = "crate::ui::Bad";\n')
        self.assert_status(self.architecture(), 0)

    def test_architecture_forbidden_paths(self):
        for code in ('use crate::wasm::Editor;', 'use crate::{core::Grid, wasm::Editor};',
                     'use crate::{core::Grid,\n render::{Font, Pixel}};',
                     'fn f() { crate::ui::Toolbar::new(); }',
                     'use super::super::wasm::Editor;', 'use ::web_sys::Window;'):
            with self.subTest(code=code):
                (self.root / 'src/core/mod.rs').write_text(code)
                self.assert_status(self.architecture(), 1)

    def test_architecture_utils_core(self):
        (self.root / 'src/utils/mod.rs').write_text('use crate::core::Grid;')
        self.assert_status(self.architecture(), 1)

    def test_architecture_scan_error(self):
        (self.root / 'src/core/mod.rs').write_bytes(b'\xff')
        self.assert_status(self.architecture(), 2)

    def test_architecture_unterminated_input_is_unverified(self):
        (self.root / 'src/core/mod.rs').write_text('/* unterminated')
        self.assert_status(self.architecture(), 2)

    def test_architecture_missing_tree(self):
        shutil.rmtree(self.root / 'src')
        self.assert_status(self.architecture(), 2)

    def test_loc_clean_and_web_limit(self):
        self.assert_status(self.loc(), 0)
        (self.root / 'web/large.ts').write_text('\n' * 501)
        self.assert_status(self.loc(), 1)

    def test_loc_budget_and_growth(self):
        (self.root / 'web/large.ts').write_text('\n' * 501)
        (self.root / '.loc-allowlist').write_text('web/large.ts=501\n')
        self.assert_status(self.loc(), 0)
        (self.root / 'web/large.ts').write_text('\n' * 502)
        self.assert_status(self.loc(), 1)

    def test_loc_stale_and_test_exclusion(self):
        (self.root / 'web/large.test.ts').write_text('\n' * 501)
        (self.root / '.loc-allowlist').write_text('web/missing.ts=501\n')
        result = self.loc()
        self.assert_status(result, 0)
        self.assertIn('STALE', result.stdout)

    def test_bandit_clean_requires_json_and_recursive(self):
        self.bandit(json.dumps(self.bandit_report()))
        self.assert_status(self.run_bandit(), 0)
        calls = (self.root / 'bandit-calls').read_text().splitlines()
        self.assertEqual(len(calls), 1)
        self.assertIn('-f json', calls[0])
        self.assertIn('-r', calls[0])

    def test_bandit_high_fails(self):
        self.bandit(json.dumps(self.bandit_report('HIGH')), status=1)
        result = self.run_bandit()
        self.assert_status(result, 1)
        self.assertIn('1 finding(s) at or above HIGH', result.stdout)

    def test_bandit_critical_fails(self):
        self.bandit(json.dumps(self.bandit_report('CRITICAL')), status=1)
        self.assert_status(self.run_bandit(), 1)

    def test_bandit_advisories_below_threshold_pass(self):
        self.bandit(json.dumps(self.bandit_report('LOW', 'MEDIUM', 'MEDIUM')), status=1)
        result = self.run_bandit()
        self.assert_status(result, 0)
        self.assertIn('3 below threshold', result.stdout)

    def test_bandit_missing_tool_is_unverified(self):
        result = self.run_bandit()
        self.assert_status(result, 2)
        self.assertIn('not found on PATH', result.stderr)

    def test_bandit_scanner_errors_are_unverified(self):
        self.bandit(json.dumps(self.bandit_report('HIGH', errors=[{'message': 'bad file'}])),
                    status=1)
        self.assert_status(self.run_bandit(), 2)

    def test_bandit_unknown_severity_is_unverified(self):
        self.bandit(json.dumps(self.bandit_report('BOGUS')), status=1)
        self.assert_status(self.run_bandit(), 2)

    def test_bandit_prose_output_is_unverified(self):
        self.bandit('Traceback (most recent call last): ...', status=1)
        self.assert_status(self.run_bandit(), 2)

    def test_bandit_stale_version_is_unverified(self):
        # An installed bandit that is not the CI-pinned 1.9.4 is a different
        # sensor and cannot report PASS for the required check (L-005/L-026).
        self.bandit(json.dumps(self.bandit_report()), version='bandit 1.7.0')
        result = self.run_bandit()
        self.assert_status(result, 2)
        self.assertIn('does not match the pinned bandit', result.stderr)

    def test_bandit_status_findings_mismatch_is_unverified(self):
        # Findings without the expected non-zero exit, or a clean-looking
        # report accompanying a failed run, both prove nothing.
        for severities, status in ((('HIGH',), 0), ((), 1)):
            with self.subTest(severities=severities, status=status):
                self.bandit(json.dumps(self.bandit_report(*severities)), status=status)
                self.assert_status(self.run_bandit(), 2)

    def test_actionlint_clean_requires_json_and_format_flag(self):
        self.actionlint('[]')
        result = self.run_actionlint()
        self.assert_status(result, 0)
        self.assertIn('[PASS]', result.stdout)
        calls = (self.root / 'actionlint-calls').read_text()
        self.assertIn('-format', calls)
        self.assertIn('{{ json . }}', calls)

    def test_actionlint_findings_fail(self):
        finding = {'message': 'bad', 'filepath': 'x.yml', 'line': 1, 'column': 1,
                   'kind': 'shellcheck', 'snippet': 'x'}
        self.actionlint(json.dumps([finding]), status=1)
        result = self.run_actionlint()
        self.assert_status(result, 1)
        self.assertIn('1 finding(s)', result.stdout)
        self.assertIn('x.yml:1:1', result.stdout)

    def test_actionlint_missing_tool_is_unverified(self):
        result = self.run_actionlint()
        self.assert_status(result, 2)
        self.assertIn('not found on PATH', result.stderr)

    def test_actionlint_stale_version_is_unverified(self):
        # A different actionlint than CI ran is a different sensor (L-005/L-027).
        self.actionlint('[]', version='1.7.7')
        result = self.run_actionlint()
        self.assert_status(result, 2)
        self.assertIn('does not match the pinned actionlint', result.stderr)

    def test_actionlint_prose_output_is_unverified(self):
        self.actionlint('Traceback (most recent call last): ...', status=1)
        self.assert_status(self.run_actionlint(), 2)

    def test_actionlint_non_array_payload_is_unverified(self):
        # Real actionlint emits `[]` or an array; an object proves a fork.
        self.actionlint(json.dumps({'findings': []}), status=0)
        self.assert_status(self.run_actionlint(), 2)

    def test_actionlint_finding_missing_field_is_unverified(self):
        bad = {'message': 'oops', 'filepath': 'x.yml'}
        self.actionlint(json.dumps([bad]), status=1)
        result = self.run_actionlint()
        self.assert_status(result, 2)
        self.assertIn('missing', result.stderr)

    def test_actionlint_malformed_finding_entry_is_unverified(self):
        self.actionlint(json.dumps(['not-a-dict']), status=1)
        self.assert_status(self.run_actionlint(), 2)

    def test_actionlint_status_findings_mismatch_is_unverified(self):
        # Both directions: findings without the expected non-zero exit, or a
        # clean-looking payload accompanying a failed run.
        finding = {'message': 'bad', 'filepath': 'x.yml', 'line': 1, 'column': 1,
                   'kind': 'shellcheck', 'snippet': 'x'}
        for payload, status in ((json.dumps([finding]), 0), ('[]', 1)):
            with self.subTest(status=status):
                self.actionlint(payload, status=status)
                self.assert_status(self.run_actionlint(), 2)


if __name__ == '__main__':
    unittest.main()
