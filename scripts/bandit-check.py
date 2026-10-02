#!/usr/bin/env python3
"""Bandit parity for the required Codacy check: 0 clean, 1 findings, 2 unverified."""
import json
from pathlib import Path
import shutil
import subprocess
import sys

SEVERITIES = ('LOW', 'MEDIUM', 'HIGH', 'CRITICAL')
THRESHOLD = 'HIGH'
PIN = '1.9.4'
ROOT = Path(__file__).resolve().parent.parent
TARGET = 'scripts'


def classify(output, status):
    """Never classify prose; require a complete structured report."""
    report = json.loads(output)
    if not isinstance(report, dict) or 'error' in report:
        raise ValueError('bandit errored or emitted prose')
    if report.get('errors'):
        raise ValueError(f'bandit reported {len(report["errors"])} scanner error(s)')
    results = report['results']
    if not isinstance(results, list):
        raise ValueError('missing results array')
    for item in results:
        if not isinstance(item, dict):
            raise ValueError('malformed finding entry')
        severity = item.get('issue_severity')
        if severity not in SEVERITIES:
            raise ValueError(f'unknown bandit severity {severity!r}')
    findings = sum(1 for item in results
                   if SEVERITIES.index(item['issue_severity']) >= SEVERITIES.index(THRESHOLD))
    advisories = len(results) - findings
    # Bandit exits 1 iff it found anything at all; a report contradicting the
    # exit code (or an unknown code) is not proof the scanner ran cleanly.
    if status not in (0, 1) or (not results and status != 0) or (results and status != 1):
        raise ValueError(f'inconsistent bandit exit status {status}')
    return findings, advisories


def command():
    """Prefer an installed bandit; fall back to uvx with the CI pin; else unverified."""
    if shutil.which('bandit'):
        return ['bandit', '-r', '-f', 'json', '-q', TARGET]
    if shutil.which('uvx'):
        return ['uvx', '--from', f'bandit=={PIN}', 'bandit', '-r', '-f', 'json', '-q', TARGET]
    raise ValueError(f'bandit=={PIN} (or uvx) not found on PATH')


def main():
    try:
        result = subprocess.run(command(), cwd=ROOT, capture_output=True, text=True, timeout=300)
        findings, advisories = classify(result.stdout, result.returncode)
    except (OSError, subprocess.TimeoutExpired, ValueError, KeyError, TypeError) as error:
        print(f'[UNVERIFIED] bandit: {error}; install bandit=={PIN} (or uvx) and retry',
              file=sys.stderr)
        return 2
    print(f'bandit: {findings} finding(s) at or above {THRESHOLD}, '
          f'{advisories} below threshold (advisory only)')
    if findings:
        print(result.stdout)
        return 1
    print(f'[PASS] {TARGET}/ clean at the {THRESHOLD} threshold')
    return 0


if __name__ == '__main__':
    sys.exit(main())
