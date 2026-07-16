#!/usr/bin/env bash
# Optional local pre-commit hook body (computational sensors, fast tier).
# Install: ln -sf ../../scripts/pre-commit-fast.sh .git/hooks/pre-commit
# Or run manually: ./scripts/pre-commit-fast.sh
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec bash "$REPO_ROOT/scripts/quality-gates.sh" --fast
