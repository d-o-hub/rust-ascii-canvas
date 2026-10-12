#!/usr/bin/env bash
# Fast pre-commit hook delegated to do-harness or quality-gates fast tier.
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT" || exit 1

if command -v do-harness &>/dev/null; then
  exec do-harness verify --set feedback --changed
fi

exec bash "$REPO_ROOT/scripts/quality-gates.sh" --fast
