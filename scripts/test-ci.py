#!/usr/bin/env python3
"""Retained applicability and jobs -> needs -> result-map mutation fixtures."""
import importlib.util
from pathlib import Path
import sys
import unittest

sys.dont_write_bytecode = True

ROOT = Path(__file__).resolve().parent.parent
SPEC = importlib.util.spec_from_file_location('check_ci', ROOT / 'scripts/check-ci.py')
ci = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ci)


class Coherence(unittest.TestCase):
    def setUp(self):
        self.workflow = (ROOT / '.github/workflows/ci.yml').read_text()

    def assert_rejected(self, text):
        with self.assertRaises(ValueError):
            ci.check(text)

    def test_real_workflow_and_path_fixtures(self):
        ci.check(self.workflow)

    def test_new_unaggregated_job(self):
        self.assert_rejected(self.workflow + '\n  surprise:\n    runs-on: ubuntu-latest\n')

    def test_missing_need(self):
        self.assert_rejected(self.workflow.replace('security-npm, deny, loc', 'deny, loc'))

    def test_missing_env_result(self):
        self.assert_rejected(self.workflow.replace('          LOC: ${{ needs.loc.result }}\n', ''))

    def test_missing_shell_result(self):
        self.assert_rejected(self.workflow.replace('[loc]="$LOC"', ''))

    def test_swapped_result(self):
        self.assert_rejected(self.workflow.replace('[loc]="$LOC"', '[loc]="$WEB"'))

    def test_stale_job_name(self):
        self.assert_rejected(self.workflow.replace('  loc:\n', '  renamed-loc:\n'))

    def test_missing_config_path(self):
        self.assert_rejected(self.workflow.replace("              - '.cargo/**'\n", ''))

    def test_rust_api_must_typecheck_web(self):
        before, web = self.workflow.split('  web:\n', 1)
        web, after = web.split('  e2e:\n', 1)
        web = web.replace("needs.changes.outputs.rust == 'true' ||", '')
        self.assert_rejected(before + '  web:\n' + web + '  e2e:\n' + after)

    def test_false_condition_cannot_masquerade_as_coverage(self):
        before, web = self.workflow.split('  web:\n', 1)
        web = web.replace('always() &&', 'always() && false &&', 1)
        self.assert_rejected(before + '  web:\n' + web)

    def test_unknown_condition_fails_closed(self):
        self.assert_rejected(self.workflow.replace('always() &&', 'mystery() &&', 1))

    def test_removed_direct_sensor(self):
        self.assert_rejected(self.workflow.replace('python3 scripts/test-sensors.py', 'echo skipped'))

    def test_missing_workflow(self):
        self.assert_rejected('')

    def test_unpinned_package_manager_is_rejected(self):
        with self.assertRaises(ValueError):
            ci.package_manager_version({}, {})

    def test_workspace_package_managers_must_match(self):
        with self.assertRaises(ValueError):
            ci.package_manager_version({'packageManager': 'pnpm@10.34.5'},
                                       {'packageManager': 'pnpm@11.7.0'})

    def test_package_manager_pin_is_exact(self):
        for value in ('pnpm@10', 'pnpm@latest', 'npm@10.34.5'):
            with self.subTest(value=value), self.assertRaises(ValueError):
                ci.package_manager_version({'packageManager': value}, {'packageManager': value})
        self.assertEqual(ci.package_manager_version({'packageManager': 'pnpm@10.34.5'},
                                                    {'packageManager': 'pnpm@10.34.5'}), '10.34.5')

    def test_ci_package_manager_cannot_drift(self):
        self.assert_rejected(self.workflow.replace('version: 10.34.5', 'version: 11.7.0', 1))

    def test_locked_ci_requires_committed_cargo_lock(self):
        with self.assertRaises(ValueError):
            ci.lockfile_committed('cargo build --locked', is_tracked=lambda: False)
        with self.assertRaises(ValueError):
            ci.lockfile_committed('cargo audit --file Cargo.lock', is_tracked=lambda: False)
        ci.lockfile_committed('cargo build --locked', is_tracked=lambda: True)

    def test_unlocked_ci_needs_no_committed_lock(self):
        ci.lockfile_committed('cargo build', is_tracked=lambda: False)

    def test_real_workflow_lock_requirement_matches_repo_state(self):
        # ci.check() runs the real `git ls-files` probe: an ignored/untracked
        # Cargo.lock with --locked steps must fail here, not only on CI.
        ci.check(self.workflow)


if __name__ == '__main__':
    unittest.main()
