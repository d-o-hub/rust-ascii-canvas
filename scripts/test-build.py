#!/usr/bin/env python3
"""Offline WASM freshness and nonzero test-execution fixtures."""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent


class BuildEvidence(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        shutil.copytree(ROOT / 'scripts', self.root / 'scripts', ignore=shutil.ignore_patterns('__pycache__'))
        for name in ('src', 'web/pkg', '.cargo', 'bin'):
            (self.root / name).mkdir(parents=True)
        for name in ('Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'mise.toml',
                     'package.json', 'pnpm-lock.yaml', '.cargo/config.toml', 'src/lib.rs'):
            (self.root / name).write_text('fixture\n')
        for name in ('ascii_canvas.js', 'ascii_canvas.d.ts', 'ascii_canvas_bg.wasm.d.ts'):
            (self.root / 'web/pkg' / name).write_text('generated fixture\n')
        (self.root / 'web/pkg/ascii_canvas_bg.wasm').write_bytes(b'\0asm\1\0\0\0')

    def freshness(self, *args):
        return subprocess.run(['python3', str(self.root / 'scripts/wasm-freshness.py'), *args],
                              cwd=self.root, text=True, capture_output=True)

    def stamp(self):
        digest = self.freshness('--print')
        self.assertEqual(digest.returncode, 0, digest.stderr)
        result = self.freshness('--record', digest.stdout.strip())
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_missing_stamp_rejected(self):
        self.assertNotEqual(self.freshness('--check').returncode, 0)

    def test_fresh_inputs_and_outputs(self):
        self.stamp()
        self.assertEqual(self.freshness('--check').returncode, 0)

    def test_changed_input_is_stale(self):
        self.stamp()
        (self.root / 'src/lib.rs').write_text('changed API')
        self.assertNotEqual(self.freshness('--check').returncode, 0)

    def test_changed_output_is_stale(self):
        self.stamp()
        (self.root / 'web/pkg/ascii_canvas.d.ts').write_text('old output')
        self.assertNotEqual(self.freshness('--check').returncode, 0)

    def test_missing_output_is_stale(self):
        self.stamp()
        (self.root / 'web/pkg/ascii_canvas.js').unlink()
        self.assertNotEqual(self.freshness('--check').returncode, 0)

    def test_source_change_during_build_cannot_be_stamped(self):
        digest = self.freshness('--print').stdout.strip()
        (self.root / 'src/lib.rs').write_text('changed during build')
        self.assertNotEqual(self.freshness('--record', digest).returncode, 0)

    def test_ensure_rebuilds_stale_inputs(self):
        (self.root / 'scripts/build-wasm.sh').write_text('''#!/bin/sh
set -e
echo rebuilt > build-called
digest=$(python3 scripts/wasm-freshness.py --print)
python3 scripts/wasm-freshness.py --record "$digest"
''')
        self.stamp()
        (self.root / 'src/lib.rs').write_text('changed API')
        result = self.freshness('--ensure')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue((self.root / 'build-called').exists())
        self.assertEqual(self.freshness('--check').returncode, 0)

    def test_ensure_propagates_build_failure(self):
        (self.root / 'scripts/build-wasm.sh').write_text('#!/bin/sh\nexit 42\n')
        self.assertNotEqual(self.freshness('--ensure').returncode, 0)
        self.assertNotEqual(self.freshness('--check').returncode, 0)

    def test_invalid_stamp_is_stale(self):
        (self.root / 'web/pkg/.build-fingerprint.json').write_text('{}')
        self.assertNotEqual(self.freshness('--check').returncode, 0)

    def wasm(self, output, status=0):
        cargo = self.root / 'bin/cargo'
        cargo.write_text(f'#!/bin/sh\necho "$*" >> "{self.root / "cargo-calls"}"\nprintf "%s\\n" "{output}"\nexit {status}\n')
        cargo.chmod(0o755)
        return subprocess.run(['bash', str(self.root / 'scripts/test-wasm.sh')], cwd=self.root,
                              env=dict(os.environ, PATH=f'{self.root / "bin"}:/usr/bin:/bin'),
                              text=True, capture_output=True)

    def test_zero_wasm_execution_is_not_success(self):
        self.assertNotEqual(self.wasm('test result: ok. 0 passed; 0 failed; 0 ignored;').returncode, 0)

    def test_positive_wasm_execution(self):
        self.assertEqual(self.wasm('test result: ok. 11 passed; 0 failed; 0 ignored;').returncode, 0)

    def test_both_wasm_targets_are_executed(self):
        self.assertEqual(self.wasm('test result: ok. 11 passed; 0 failed; 0 ignored;').returncode, 0)
        calls = (self.root / 'cargo-calls').read_text()
        self.assertIn('--lib', calls)
        self.assertIn('--test wasm_tests', calls)
        self.assertTrue(all('--locked' in line for line in calls.splitlines()))

    def test_wasm_failure_is_propagated(self):
        self.assertNotEqual(self.wasm('test result: ok. 11 passed; 0 failed;', 1).returncode, 0)


if __name__ == '__main__':
    unittest.main()
