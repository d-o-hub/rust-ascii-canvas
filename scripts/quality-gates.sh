#!/usr/bin/env bash
# Local computational sensors runner delegated to do-harness.
# Usage: ./scripts/quality-gates.sh [--fast] [--fix]
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT" || exit 1

FAST=false
FIX=false
for arg in "$@"; do
  case "$arg" in
    --fast) FAST=true ;;
    --fix) FIX=true ;;
    -h|--help) echo 'Usage: scripts/quality-gates.sh [--fast] [--fix]'; exit 0 ;;
    *) echo "Unknown argument: $arg" >&2; exit 2 ;;
  esac
done

if $FIX; then
  env RUSTUP_TOOLCHAIN=1.94.0 cargo fmt --all
  env RUSTUP_TOOLCHAIN=1.94.0 cargo clippy --locked --fix --allow-dirty --allow-staged --all-targets --all-features
fi

if command -v do-harness &>/dev/null; then
  if $FAST; then
    exec do-harness verify --set feedback
  else
    exec do-harness verify --set verification
  fi
fi

echo "Warning: do-harness CLI not found on PATH. Falling back to direct sensor execution." >&2

# Fallback direct execution loop for rollback / environment compatibility
TIER=full
$FAST && TIER=fast
FAILED=0

pass() { echo "[PASS] $1"; }
fail() { echo "[FAIL] $1"; FAILED=1; }
info() { echo "[INFO] $1"; }
run() {
  local label="$1" hint="$2" output
  shift 2
  info "$label"
  if output=$("$@" 2>&1); then
    pass "$label"
    [[ -z "$output" ]] || printf '%s\n' "$output"
  else
    fail "$label"
    printf '%s\n  FIX: %s\n' "$output" "$hint"
  fi
}

run 'LOC limits (src + web)' 'Extract oversized modules; never grow allowlist budgets.' bash scripts/check-loc.sh
run 'Architecture paths' 'Move shared types to core/utils; scanner errors are unverified.' bash scripts/check-architecture.sh
run 'CI applicability/coherence' 'Align path filters, needs and result maps; see fixtures.' python3 scripts/check-ci.py
run 'CI mutation fixtures' 'Fix the sensor, never weaken its negative fixtures.' python3 scripts/test-ci.py
run 'Audit/artifact/architecture/LOC fixtures' 'Fix the failing sensor regression.' python3 scripts/test-sensors.py
run 'Build evidence fixtures' 'Freshness and nonzero execution must fail closed.' python3 scripts/test-build.py
run 'Skill resource coherence' 'Restore missing resources/owned procedures; respect provenance.' python3 scripts/check-skills.py
run 'Skill coherence fixtures' 'Fix the checker without bypassing owned guidance.' python3 scripts/test-skills.py
run 'Version pin parity' 'Run scripts/propagate-version.sh; no release pin changes in this task.' bash scripts/propagate-version.sh --check

run 'Merge-gate predicates' 'Restore the read-only merge contract predicates.' bash scripts/pr-merge-gate.sh --self-test
run 'Ruleset snapshot' 'Restore the committed contract; live mutations require human-approved ADR.' \
  jq -e '(.requiredStatusChecks | index("CI Success") != null and index("PR Readiness (merge gate)") != null) and .requiredReviewThreadResolution == true' .github/ruleset-main.json

run 'Format' 'cargo fmt --all' env RUSTUP_TOOLCHAIN=1.94.0 cargo fmt --all -- --check
run 'Clippy' 'Fix lints instead of suppressing them.' env RUSTUP_TOOLCHAIN=1.94.0 cargo clippy --locked --all-targets --all-features -- -D warnings
run 'Rust build' 'Fix compile errors before continuing.' env RUSTUP_TOOLCHAIN=1.94.0 cargo build --locked --all-targets
run 'Rust tests' 'Fix root cause; never skip or weaken assertions.' env RUSTUP_TOOLCHAIN=1.94.0 cargo test --locked --all

BINDINGS_READY=false
if env RUSTUP_TOOLCHAIN=1.94.0 python3 scripts/wasm-freshness.py --ensure; then
  BINDINGS_READY=true
else
  fail 'Fresh WASM bindings unavailable; typechecks cannot be verified'
fi

if [[ ! -d node_modules || ! -d web/node_modules ]]; then
  fail 'Required root/web dependencies missing'
fi

if $BINDINGS_READY; then
  run 'Web lint + types' 'cd web && pnpm run lint' bash -c 'cd web && pnpm run lint'
  run 'TypeScript (web)' 'cd web && pnpm exec tsc --noEmit' bash -c 'cd web && pnpm exec tsc --noEmit'
  run 'TypeScript (e2e)' 'Use the same web compiler, not an accidental root tsc.' web/node_modules/.bin/tsc --noEmit -p e2e/tsconfig.json
fi

run 'Playwright production/shadow config fixtures' 'Preserve strict production preview and server-free BASE_URL support.' node --test scripts/test-playwright-config.mjs
run 'Vitest' 'Fix the web unit regressions.' bash -c 'cd web && pnpm test'
run 'ESLint (root/e2e)' 'Root config deliberately excludes web, which has its own check.' ./node_modules/.bin/eslint .
run 'Privacy & Secrets' 'Remove personal data/secrets' bash scripts/check-privacy-secrets.sh

if ! $FAST; then
  run 'Security audit (committed Cargo.lock)' 'Install cargo-audit explicitly or fix reported advisories/tool errors.' env RUSTUP_TOOLCHAIN=1.94.0 cargo audit --file Cargo.lock
  run 'Security audit (both npm lockfiles)' 'Restore pnpm/registry access or fix the findings; partial is unverified.' bash scripts/npm-audit.sh
  run 'Security audit (Python sensors)' 'Install bandit==1.9.4 or uvx; a scanner error is not a clean tree.' python3 scripts/bandit-check.py
  run 'GitHub Actions workflow lint' 'Install actionlint==1.6.26; a broken or stale linter is not a clean tree.' python3 scripts/actionlint-check.py
  run 'Dependency policy' 'Install cargo-deny explicitly; align locked dependencies with deny.toml.' env RUSTUP_TOOLCHAIN=1.94.0 cargo deny --locked check
  run 'Codacy repo intake (local only, not a CI gate)' 'Read/fix findings; never remove required Codacy checks.' bash scripts/codacy-check.sh
  run 'WASM tests (nonzero Node execution)' 'Restore test discovery/runner and fix behavior regressions.' env RUSTUP_TOOLCHAIN=1.94.0 bash scripts/test-wasm.sh
  run 'WASM artifact validity + size' 'Build valid WASM and reduce size if over budget.' node scripts/check-artifact.mjs
  if OUTPUT=$(env RUSTUP_TOOLCHAIN=1.94.0 pnpm run build:web 2>&1); then
    pass 'Production web build'
    run 'Production E2E (Chromium)' 'Inspect Playwright failures; do not substitute a stale dev server.' \
      env PRODUCTION_E2E=1 pnpm exec playwright test --project=chromium --timeout=120000
  else
    fail 'Production web build'
  fi
fi

if [[ $FAILED -ne 0 ]]; then
  echo "[FAIL] Quality gate (tier=$TIER). Self-correct using FIX hints and rerun."
  exit 1
fi
echo "[PASS] All quality gates (tier=$TIER)"
