#!/usr/bin/env bash
# Fast pre-commit hook delegated to do-harness or quality-gates fast tier.
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT" || exit 1

set +e
bash "$REPO_ROOT/scripts/do-harness.sh" verify --set feedback --changed --fail-fast
HARNESS_STATUS=$?
set -e
if [[ "$HARNESS_STATUS" -ne 127 ]]; then
  exit "$HARNESS_STATUS"
fi
if [[ "${DO_HARNESS_NO_FALLBACK:-0}" == "1" ]]; then
  echo "[FAIL] do-harness is unavailable and fallback is disabled." >&2
  exit 127
fi
echo "[WARN] do-harness unavailable; using the legacy fast fallback." >&2
exec bash "$REPO_ROOT/scripts/quality-gates.sh" --fast
