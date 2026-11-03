#!/usr/bin/env bash
# Check the pinned CLI and the repository's declared signal sets without running sensors.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION_JSON="$(bash "$ROOT/scripts/do-harness.sh" version --format json)"
SETS_JSON="$(bash "$ROOT/scripts/do-harness.sh" list --sets --format json)"

python3 - "$VERSION_JSON" "$SETS_JSON" <<'PY'
import json
import sys

version = json.loads(sys.argv[1])
sets = json.loads(sys.argv[2])
expected = {"feedback", "verification", "release"}
actual = set(sets)
if actual != expected:
    missing = ", ".join(sorted(expected - actual)) or "none"
    extra = ", ".join(sorted(actual - expected)) or "none"
    raise SystemExit(f"signal-set mismatch (missing: {missing}; unexpected: {extra})")
print(f"[PASS] do-harness v{version['version']} signal sets: {', '.join(sorted(actual))}")
PY
