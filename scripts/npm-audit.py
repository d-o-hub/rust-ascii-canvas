#!/usr/bin/env python3
"""Audit BOTH committed pnpm lockfiles: 0 clean, 1 findings, 2 unverified."""
import json
from pathlib import Path
import subprocess
import sys

LEVELS = ('info', 'low', 'moderate', 'high', 'critical')
THRESHOLD = 'high'
ROOT = Path(__file__).resolve().parent.parent


def classify(output, status):
    """Never classify prose; require complete structured vulnerability counts."""
    report = json.loads(output)
    if not isinstance(report, dict) or 'error' in report:
        raise ValueError('registry/tool error or invalid report')
    counts = report['metadata']['vulnerabilities']
    if not isinstance(counts, dict) or any(type(counts.get(key)) is not int or counts[key] < 0
                                          for key in LEVELS):
        raise ValueError('missing/invalid vulnerability counts')
    findings = sum(counts[key] for key in LEVELS[LEVELS.index(THRESHOLD):])
    # pnpm uses 1 for findings and 0 for clean. Neither an unknown exit code
    # nor a clean-looking report accompanying a failed command proves an audit.
    if status not in (0, 1) or (not findings and status != 0) or (findings and status != 1):
        raise ValueError(f'inconsistent audit exit status {status}')
    return findings


def main():
    checked = 0
    findings = False
    for directory in (ROOT, ROOT / 'web'):
        label = 'root' if directory == ROOT else 'web'
        try:
            if not (directory / 'pnpm-lock.yaml').is_file():
                raise ValueError('required committed pnpm-lock.yaml missing')
            result = subprocess.run(['pnpm', 'audit', '--json', '--audit-level=high'],
                                    cwd=directory, capture_output=True, text=True, timeout=120)
            count = classify(result.stdout, result.returncode)
            checked += 1
            findings |= count > 0
            print(f'{label}: {count} advisories at or above {THRESHOLD}')
            if count:
                print(result.stdout)
        except (OSError, subprocess.TimeoutExpired, ValueError, KeyError, TypeError) as error:
            print(f'[UNVERIFIED] {label}: {error}; restore tools/lockfile/registry and retry',
                  file=sys.stderr)
    if checked != 2:
        print(f'[UNVERIFIED] Only {checked}/2 lockfiles checked; NOT a pass', file=sys.stderr)
        return 2
    if findings:
        return 1
    print('[PASS] Both lockfiles checked, clean at the requested threshold')
    return 0


if __name__ == '__main__':
    sys.exit(main())
