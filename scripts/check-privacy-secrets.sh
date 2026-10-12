#!/usr/bin/env bash
# Privacy and secrets scanner for quality gates and do-harness sensor.
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT" || exit 1

pass() { echo "[PASS] $1"; }
fail() { echo "[FAIL] $1"; FAILED=1; }

FAILED=0

scan() {
  local label="$1" pattern="$2" exclusions="$3" matches filtered rc
  matches=$(grep -rE "$pattern" \
    --exclude-dir=.git --exclude-dir=target --exclude-dir=.agents --exclude-dir=.opencode \
    --exclude-dir=.mimocode --exclude-dir=node_modules --exclude-dir=playwright-report \
    --exclude-dir=test-results --exclude-dir=__pycache__ --exclude-dir=.do-harness . 2>&1)
  rc=$?
  filtered=$(printf '%s\n' "$matches" | grep -vE "$exclusions")
  if [[ $rc -gt 1 ]]; then
    fail "$label scanner unavailable"
    printf '%s\n' "$matches"
  elif [[ -n "$filtered" ]]; then
    fail "$label"
    printf '%s\n' "$filtered"
    echo '  FIX: remove personal data/secrets; use example.com or environment inputs.'
  else
    pass "$label"
  fi
}

scan 'Privacy' '[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}' \
  'example\.com|example\.org|test\.com|\.git|target|\.opencode|\.mimocode|node_modules|playwright-report|test-results|\.md|references|agents-docs|\.agents|\.do-harness'

scan 'Secrets' "(api_key|token|secret|password|auth|key)[[:space:]]*[:=][[:space:]]*['\"][a-zA-Z0-9_\-]{16,}['\"]" \
  'example\.com|example\.org|test\.com|GITHUB_TOKEN|CARGO_REGISTRY_TOKEN|worktree|shared-key|release-workflow'

if [[ $FAILED -ne 0 ]]; then
  exit 1
fi
