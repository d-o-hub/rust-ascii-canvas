#!/usr/bin/env bash
# Shared local full-tier / CI security-npm entrypoint (ADR-047).
# Required evidence: BOTH committed lockfiles, structured pnpm audit JSON.
# Exit 0 clean, 1 findings, 2 unverified (including missing tools/offline/partial).
set -euo pipefail
exec python3 "$(dirname "${BASH_SOURCE[0]}")/npm-audit.py" "$@"
