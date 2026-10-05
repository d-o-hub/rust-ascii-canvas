#!/usr/bin/env bash
# One fail-closed scanner, shared by fast/full and the CI architecture job.
# Exit 0 clean, 1 layer violation, 2 scan unavailable/incomplete.
set -euo pipefail
exec python3 "$(dirname "${BASH_SOURCE[0]}")/check-architecture.py" "$@"
