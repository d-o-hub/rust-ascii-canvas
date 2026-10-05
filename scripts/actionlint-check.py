#!/usr/bin/env python3
"""Fail-closed actionlint parity for GitHub Actions workflows: 0 clean, 1 findings, 2 unverified."""
import json
from pathlib import Path
import shutil
import subprocess
import sys

PIN = '1.6.26'
ROOT = Path(__file__).resolve().parent.parent
FIELDS = ('message', 'filepath', 'line', 'column', 'kind')


def classify(output, status):
    """Never classify prose; require a complete structured report."""
    findings = json.loads(output)
    if not isinstance(findings, list):
        raise ValueError('actionlint emitted prose or a non-array payload')
    for item in findings:
        if not isinstance(item, dict):
            raise ValueError('malformed finding entry')
        missing = [field for field in FIELDS if field not in item]
        if missing:
            raise ValueError(f'finding missing {missing}')
    # actionlint exits 1 iff it found anything; a report contradicting the exit
    # code (or an unknown code) is not proof the linter ran cleanly.
    if status not in (0, 1) or (not findings and status != 0) or (findings and status != 1):
        raise ValueError(f'inconsistent actionlint exit status {status}')
    return findings


def scan():
    """Trust an installed actionlint only when its version matches the CI pin;
    a stale sensor reporting PASS is a different sensor than CI ran (L-005)."""
    probe = subprocess.run(['actionlint', '--version'], cwd=ROOT,
                           capture_output=True, text=True, timeout=30)
    if probe.returncode != 0:
        raise ValueError('actionlint --version failed; the tool is broken')
    first = probe.stdout.splitlines()[0] if probe.stdout else ''
    if first != PIN:
        raise ValueError(f'installed {first!r} does not match the pinned actionlint {PIN}')
    return subprocess.run(['actionlint', '-format', '{{ json . }}'],
                          cwd=ROOT, capture_output=True, text=True, timeout=180)


def main():
    try:
        if not shutil.which('actionlint'):
            raise ValueError(f'actionlint=={PIN} not found on PATH')
        result = scan()
        findings = classify(result.stdout, result.returncode)
    except (OSError, subprocess.TimeoutExpired, ValueError) as error:
        print(f'[UNVERIFIED] actionlint: {error}; install actionlint=={PIN} and retry',
              file=sys.stderr)
        return 2
    if findings:
        print(f'actionlint: {len(findings)} finding(s) across .github/workflows')
        for item in findings:
            kind = item['kind'] or 'expression'
            print(f'  {item["filepath"]}:{item["line"]}:{item["column"]}: '
                  f'[{kind}] {item["message"]}')
        return 1
    print(f'[PASS] .github/workflows clean under actionlint {PIN}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
