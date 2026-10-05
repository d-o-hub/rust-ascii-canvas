#!/usr/bin/env bash
# Local computational sensors. CI invokes shared entrypoints directly, not this
# convenience runner (L-016). See ADR-047 and scripts/check-ci.py for parity.
# Usage: ./scripts/quality-gates.sh [--fast] [--fix]
set -uo pipefail
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT" || exit 1
FAST=false
FIX=false
FAILED=0
for arg in "$@"; do
  case "$arg" in
    --fast) FAST=true ;;
    --fix) FIX=true ;;
    -h|--help) echo 'Usage: scripts/quality-gates.sh [--fast] [--fix]'; exit 0 ;;
    *) echo "Unknown argument: $arg" >&2; exit 2 ;;
  esac
done
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

TIER=full
$FAST && TIER=fast
printf 'Quality gates (tier=%s)...\n\n' "$TIER"
run 'LOC limits (src + web)' 'Extract oversized modules; never grow allowlist budgets.' bash scripts/check-loc.sh
run 'Architecture paths' 'Move shared types to core/utils; scanner errors are unverified.' bash scripts/check-architecture.sh
run 'CI applicability/coherence' 'Align path filters, needs and result maps; see fixtures.' python3 scripts/check-ci.py
run 'CI mutation fixtures' 'Fix the sensor, never weaken its negative fixtures.' python3 scripts/test-ci.py
run 'Audit/artifact/architecture/LOC fixtures' 'Fix the failing sensor regression.' python3 scripts/test-sensors.py
run 'Build evidence fixtures' 'Freshness and nonzero execution must fail closed.' python3 scripts/test-build.py
run 'Skill resource coherence' 'Restore missing resources/owned procedures; respect provenance.' python3 scripts/check-skills.py
run 'Skill coherence fixtures' 'Fix the checker without bypassing owned guidance.' python3 scripts/test-skills.py
run 'Version pin parity' 'Run scripts/propagate-version.sh; no release pin changes in this task.' bash scripts/propagate-version.sh --check

# CLI/schema versions must match. Pins themselves are not changed here.
WBG_CARGO="$(sed -n 's/^wasm-bindgen = "=\([0-9.]*\)".*/\1/p' Cargo.toml | head -n 1)"
WBG_MISE="$(sed -n 's/.*cargo:wasm-bindgen-cli" = "\([0-9.]*\)".*/\1/p' mise.toml | head -n 1)"
WBG_NETLIFY="$(sed -n 's/.*wasm-bindgen-cli --version \([0-9.]*\).*/\1/p' package.json | head -n 1)"
if [[ -n "$WBG_CARGO" && "$WBG_CARGO" == "$WBG_MISE" && "$WBG_CARGO" == "$WBG_NETLIFY" ]]; then
  pass "wasm-bindgen pins aligned ($WBG_CARGO)"
else
  fail "wasm-bindgen pins unreadable/skewed: $WBG_CARGO / $WBG_MISE / $WBG_NETLIFY"
  echo '  FIX: coordinate Cargo.toml/Cargo.lock, mise.toml and netlify:build schema pins.'
fi
run 'Merge-gate predicates' 'Restore the read-only merge contract predicates.' bash scripts/pr-merge-gate.sh --self-test
# Offline snapshot half only; the live ruleset check remains in CI PR Readiness.
run 'Ruleset snapshot' 'Restore the committed contract; live mutations require human-approved ADR.' \
  jq -e '(.requiredStatusChecks | index("CI Success") != null and index("PR Readiness (merge gate)") != null) and .requiredReviewThreadResolution == true' .github/ruleset-main.json

if $FIX; then
  run 'Format (fix)' 'Fix rustfmt errors.' cargo fmt --all
  run 'Clippy (fix)' 'Re-run without --fix to enforce -D warnings.' cargo clippy --locked --fix --allow-dirty --allow-staged --all-targets --all-features
else
  run 'Format' 'cargo fmt --all' cargo fmt --all -- --check
  run 'Clippy' 'Fix lints instead of suppressing them.' cargo clippy --locked --all-targets --all-features -- -D warnings
fi
run 'Rust build' 'Fix compile errors before continuing.' cargo build --locked --all-targets
run 'Rust tests' 'Fix root cause; never skip or weaken assertions.' cargo test --locked --all

# Presence is not freshness. A content fingerprint travels with the CI artifact;
# locally --ensure rebuilds only on changed source/config/lockfile/output bytes.
BINDINGS_READY=false
if python3 scripts/wasm-freshness.py --ensure; then
  BINDINGS_READY=true
else
  fail 'Fresh WASM bindings unavailable; typechecks cannot be verified'
fi
if [[ ! -d node_modules || ! -d web/node_modules ]]; then
  fail 'Required root/web dependencies missing'
  echo '  FIX: pnpm install --frozen-lockfile && (cd web && pnpm install --frozen-lockfile)'
  echo '       No mutable fallback or silent global installation is permitted.'
fi
if $BINDINGS_READY; then
  run 'Web lint + types' 'cd web && pnpm run lint' bash -c 'cd web && pnpm run lint'
  run 'TypeScript (web)' 'cd web && pnpm exec tsc --noEmit' bash -c 'cd web && pnpm exec tsc --noEmit'
  run 'TypeScript (e2e)' 'Use the same web compiler, not an accidental root tsc.' web/node_modules/.bin/tsc --noEmit -p e2e/tsconfig.json
fi
run 'Playwright production/shadow config fixtures' 'Preserve strict production preview and server-free BASE_URL support.' node --test scripts/test-playwright-config.mjs
run 'Vitest' 'Fix the web unit regressions.' bash -c 'cd web && pnpm test'
run 'ESLint (root/e2e)' 'Root config deliberately excludes web, which has its own check.' ./node_modules/.bin/eslint .

# Keep existing local privacy/secret scans; a scanner error is not a clean tree.
scan() {
  local label="$1" pattern="$2" exclusions="$3" matches filtered rc
  matches=$(grep -rE "$pattern" \
    --exclude-dir=.git --exclude-dir=target --exclude-dir=.agents --exclude-dir=.opencode \
    --exclude-dir=.mimocode --exclude-dir=node_modules --exclude-dir=playwright-report \
    --exclude-dir=test-results --exclude-dir=__pycache__ . 2>&1)
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
  'example\.com|example\.org|test\.com|\.git|target|\.opencode|\.mimocode|node_modules|playwright-report|test-results|\.md|references|agents-docs|\.agents'
scan 'Secrets' "(api_key|token|secret|password|auth|key)[[:space:]]*[:=][[:space:]]*['\"][a-zA-Z0-9_\-]{16,}['\"]" \
  'example\.com|example\.org|test\.com|GITHUB_TOKEN|CARGO_REGISTRY_TOKEN|worktree|shared-key|release-workflow'

if ! $FAST; then
  # These required checks fail nonzero if tools/registry/advisories are unavailable.
  run 'Security audit (committed Cargo.lock)' 'Install cargo-audit explicitly or fix reported advisories/tool errors.' cargo audit --file Cargo.lock
  run 'Security audit (both npm lockfiles)' 'Restore pnpm/registry access or fix the findings; partial is unverified.' bash scripts/npm-audit.sh
  run 'Security audit (Python sensors)' 'Install bandit==1.9.4 or uvx; a scanner error is not a clean tree.' python3 scripts/bandit-check.py
  run 'GitHub Actions workflow lint' 'Install actionlint==1.6.26; a broken or stale linter is not a clean tree.' python3 scripts/actionlint-check.py
  run 'Dependency policy' 'Install cargo-deny explicitly; align locked dependencies with deny.toml.' cargo deny --locked check
  # Account credentials are not CI sensors: this intake is intentionally local.
  run 'Codacy repo intake (local only, not a CI gate)' 'Read/fix findings; never remove required Codacy checks.' bash scripts/codacy-check.sh
  run 'WASM tests (nonzero Node execution)' 'Restore test discovery/runner and fix behavior regressions.' bash scripts/test-wasm.sh
  run 'WASM artifact validity + size' 'Build valid WASM and reduce size if over budget.' node scripts/check-artifact.mjs
  # Refuse development-only evidence: build dist, then let Playwright own the
  # strict production preview lifecycle. BASE_URL still targets an external host.
  if OUTPUT=$(pnpm run build:web 2>&1); then
    pass 'Production web build'
    printf '%s\n' "$OUTPUT"
    run 'Production E2E (Chromium)' 'Inspect Playwright failures; do not substitute a stale dev server.' \
      env PRODUCTION_E2E=1 pnpm exec playwright test --project=chromium --timeout=120000
  else
    fail 'Production web build'
    printf '%s\n' "$OUTPUT"
    echo '  FIX: repair production bundling before running deployment-shaped E2E.'
  fi
else
  info 'Full-only: locked audits, WASM behavior/size, production dist and E2E. Run gate:full before review.'
fi

if [[ $FAILED -ne 0 ]]; then
  echo "[FAIL] Quality gate (tier=$TIER). Self-correct using FIX hints and rerun."
  exit 1
fi
echo "[PASS] All quality gates (tier=$TIER)"
