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


if __name__ == '__main__':
    unittest.main()
