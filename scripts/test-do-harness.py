#!/usr/bin/env python3
"""Offline fixtures for pinned do-harness discovery and bounded setup output."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parent.parent
RESOLVER = ROOT / "scripts/do-harness.sh"
CHECK = ROOT / "scripts/check-do-harness.sh"
INSTALL = ROOT / "scripts/install-do-harness.sh"


class DoHarnessResolution(unittest.TestCase):
    def make_binary(self, version="0.4.1", sets=True):
        directory = Path(self.tmp.name)
        binary = directory / "fake-do-harness"
        signal_sets = '["feedback","verification","release"]' if sets else '["verification"]'
        binary.write_text(
            "#!/bin/sh\n"
            'printf "%s\\n" "$*" > "$DO_HARNESS_TEST_LOG"\n'
            'if [ "$1" = "version" ]; then printf \'{"version":"%s"}\\n\' "'
            + version
            + '"; exit 0; fi\n'
            "if [ \"$1\" = \"list\" ]; then printf '%s\\n' '"
            + signal_sets
            + "'; exit 0; fi\n"
            'exit 2\n',
            encoding="utf-8",
        )
        binary.chmod(0o755)
        return binary

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.log = Path(self.tmp.name) / "invoked"

    def run_script(self, script, binary=None, extra=None):
        env = os.environ.copy()
        env.update({"PATH": "/usr/bin:/bin", "DO_HARNESS_TEST_LOG": str(self.log)})
        if binary is not None:
            env["DO_HARNESS_BIN"] = str(binary)
        else:
            env.pop("DO_HARNESS_BIN", None)
        if extra:
            env.update(extra)
        return subprocess.run(
            ["bash", str(script), "list", "--sets", "--format", "json"],
            cwd=ROOT,
            env=env,
            capture_output=True,
            text=True,
        )

    def test_pinned_binary_is_used_and_arguments_are_forwarded(self):
        binary = self.make_binary()
        result = self.run_script(RESOLVER, binary)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.log.read_text(encoding="utf-8").strip(), "list --sets --format json")

    def test_version_mismatch_is_unverified(self):
        result = self.run_script(RESOLVER, self.make_binary(version="0.4.0"))
        self.assertEqual(result.returncode, 127)
        self.assertIn("version mismatch", result.stderr)

    def test_missing_binary_is_unverified(self):
        result = self.run_script(RESOLVER, Path(self.tmp.name) / "missing")
        self.assertEqual(result.returncode, 127)
        self.assertIn("not executable", result.stderr)

    def test_contract_check_accepts_expected_signal_sets(self):
        binary = self.make_binary()
        env = {"DO_HARNESS_BIN": str(binary), "DO_HARNESS_TEST_LOG": str(self.log)}
        result = subprocess.run(["bash", str(CHECK)], cwd=ROOT, env={**os.environ, **env},
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("signal sets", result.stdout)

    def test_contract_check_rejects_missing_signal_set(self):
        binary = self.make_binary(sets=False)
        result = subprocess.run(["bash", str(CHECK)], cwd=ROOT,
                                env={**os.environ, "DO_HARNESS_BIN": str(binary),
                                     "DO_HARNESS_TEST_LOG": str(self.log)},
                                capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("signal-set mismatch", result.stderr)

    def test_installer_uses_the_pinned_release_tag(self):
        text = INSTALL.read_text(encoding="utf-8")
        self.assertIn('do-harness/v$VERSION/scripts/install.sh', text)
        self.assertNotIn('do-harness/main/scripts/install.sh', text)


if __name__ == "__main__":
    unittest.main(verbosity=2)
