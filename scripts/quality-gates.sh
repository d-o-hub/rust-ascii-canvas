#!/usr/bin/env bash
# scripts/quality-gates.sh
# Computational feedback sensors for the ASCII Canvas harness.
# See agents-docs/harness.md and ADR-037.
#
# Usage:
#   ./scripts/quality-gates.sh           # full tier (pre-PR / CI)
#   ./scripts/quality-gates.sh --fast    # fast tier (agent loop / pre-commit)
#   ./scripts/quality-gates.sh --fix    # auto-fix fmt/clippy where possible
#   ./scripts/quality-gates.sh --fast --fix
#
# Exit 0 = success, Exit 1 = errors.
# Failure lines include FIX: hints for agent self-correction.
set +e
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT" || exit 1

readonly MAX_LINES_PER_SOURCE_FILE=500
readonly LOC_ALLOWLIST_FILE="${REPO_ROOT}/.loc-allowlist"

FIX=false
FAST=false
for arg in "$@"; do
  case $arg in
    --fix) FIX=true ;;
    --fast) FAST=true ;;
    -h|--help)
      sed -n '2,16p' "$0"
      exit 0
      ;;
    *) echo "Unknown argument: $arg (try --fast, --fix, --help)"; exit 1 ;;
  esac
done

if [[ -t 1 ]] && [[ "${FORCE_COLOR:-}" != "0" ]]; then
  RED='\033[0;31m'
  GREEN='\033[0;32m'
  YELLOW='\033[1;33m'
  BLUE='\033[0;34m'
  NC='\033[0m'
else
  RED=''; GREEN=''; YELLOW=''; BLUE=''; NC=''
fi

pass() { echo -e "${GREEN}[PASS]${NC} $1"; }
fail() { echo -e "${RED}[FAIL]${NC} $1"; FAILED=1; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
info() { echo -e "${BLUE}[INFO]${NC} $1"; }

FAILED=0
TIER="full"
if $FAST; then TIER="fast"; fi

printf "Quality gates (tier=%s)...\n\n" "$TIER"

is_allowlisted() {
  local file="$1"
  local rel="${file#./}"
  [[ -f "$LOC_ALLOWLIST_FILE" ]] || return 1
  grep -qxF "$rel" "$LOC_ALLOWLIST_FILE" 2>/dev/null || \
    grep -qxF "./$rel" "$LOC_ALLOWLIST_FILE" 2>/dev/null
}

# ============================================================
# 1. LOC LIMITS
# ============================================================
info "LOC limits (max ${MAX_LINES_PER_SOURCE_FILE}; allowlist: .loc-allowlist)..."
LOC_VIOLATIONS=0
while IFS= read -r file; do
  [[ -z "$file" ]] && continue
  lines=$(wc -l < "$file" 2>/dev/null | tr -d ' ')
  if [[ "${lines:-0}" -gt "$MAX_LINES_PER_SOURCE_FILE" ]]; then
    if is_allowlisted "$file"; then
      warn "  $file: $lines lines (allowlisted debt — do not grow; extract modules)"
    else
      fail "  $file: $lines lines (max $MAX_LINES_PER_SOURCE_FILE)"
      echo "  FIX: Split into smaller modules. Do not add to .loc-allowlist without an ADR."
      LOC_VIOLATIONS=$((LOC_VIOLATIONS + 1))
    fi
  fi
done < <(find ./src -name "*.rs" -type f 2>/dev/null | sort)

if [[ $LOC_VIOLATIONS -eq 0 ]]; then
  pass "LOC: no new oversized files"
fi
printf "\n"

# ============================================================
# 2. ARCHITECTURE FITNESS
# ============================================================
info "Architecture fitness..."
if [[ -x "$REPO_ROOT/scripts/check-architecture.sh" ]] || [[ -f "$REPO_ROOT/scripts/check-architecture.sh" ]]; then
  if bash "$REPO_ROOT/scripts/check-architecture.sh"; then
    :
  else
    fail "Architecture: layer violations (see above)"
    echo "  FIX: Read agents-docs/architecture.md and remove illegal imports."
  fi
else
  warn "Architecture script missing"
fi
printf "\n"

# ============================================================
# 3. RUST CHECKS
# ============================================================
info "Rust checks..."

if $FIX; then
  cargo fmt --all
  pass "Format: auto-fixed"
else
  if ! OUTPUT=$(cargo fmt --all -- --check 2>&1); then
    fail "Format"
    echo "  FIX: run 'cargo fmt --all' (or ./scripts/quality-gates.sh --fix)"
    printf "%s\n" "$OUTPUT" >&2
  else
    pass "Format: OK"
  fi
fi

if $FIX; then
  cargo clippy --fix --allow-dirty --allow-staged --all-targets --all-features
  pass "Clippy: auto-fixed (re-run without --fix to verify -D warnings)"
else
  if ! OUTPUT=$(cargo clippy --all-targets --all-features -- -D warnings 2>&1); then
    fail "Clippy"
    echo "  FIX: Address clippy lints. Prefer idiomatic fixes over #[allow]."
    printf "%s\n" "$OUTPUT" >&2
  else
    pass "Clippy: OK"
  fi
fi

if ! OUTPUT=$(cargo build --all-targets 2>&1); then
  fail "Build"
  echo "  FIX: Fix compile errors above before continuing."
  printf "%s\n" "$OUTPUT" >&2
else
  pass "Build: OK"
fi

if ! OUTPUT=$(cargo test --all 2>&1); then
  fail "Tests"
  echo "  FIX: Fix failing tests; do not skip or weaken assertions to pass."
  printf "%s\n" "$OUTPUT" >&2
else
  pass "Tests: OK"
fi
printf "\n"

# ============================================================
# 4. WEB CHECKS (when web/ exists)
# ============================================================
# LEARNING (PR #128): web/pkg is gitignored. tsc imports ./pkg/ascii_canvas.js.
# Local trees often have a stale/cached pkg so gate:fast passes while CI fails.
# Always require pkg bindings before typecheck (build if missing).
if [[ -d "$REPO_ROOT/web" ]]; then
  info "Web checks (lint, typecheck, unit tests)..."

  if [[ ! -f "$REPO_ROOT/web/pkg/ascii_canvas.d.ts" ]] || [[ ! -f "$REPO_ROOT/web/pkg/ascii_canvas.js" ]]; then
    info "web/pkg missing (gitignored) — building WASM for typecheck parity with CI..."
    if ! OUTPUT=$(pnpm run build:wasm 2>&1); then
      fail "web/pkg / WASM build"
      echo "  FIX: npm run build:wasm (mise: wasm-bindgen-cli 0.2.126, target wasm32-unknown-unknown)."
      echo "  WHY: main.ts imports ./pkg/ascii_canvas.js; CI downloads wasm-pkg artifact before tsc."
      printf "%s\n" "$OUTPUT" >&2
    else
      pass "WASM pkg generated for typecheck"
    fi
  else
    pass "web/pkg bindings present"
  fi

  pushd "$REPO_ROOT/web" >/dev/null || exit 1

  if [[ ! -d node_modules ]]; then
    warn "web/node_modules missing — running pnpm install"
    pnpm install --frozen-lockfile 2>/dev/null || pnpm install
  fi

  if ! OUTPUT=$(pnpm run lint 2>&1); then
    fail "ESLint"
    echo "  FIX: cd web && pnpm run lint — fix reported issues."
    printf "%s\n" "$OUTPUT" >&2
  else
    pass "ESLint: OK"
  fi

  if command -v pnpm >/dev/null && pnpm exec tsc --version >/dev/null 2>&1; then
    if ! OUTPUT=$(pnpm exec tsc --noEmit 2>&1); then
      fail "TypeScript"
      echo "  FIX: cd web && pnpm exec tsc --noEmit — fix type errors (strict mode)."
      echo "  IF 'Cannot find module ./pkg/ascii_canvas.js': run npm run build:wasm (pkg is gitignored)."
      printf "%s\n" "$OUTPUT" >&2
    else
      pass "TypeScript: OK"
    fi
  else
    warn "tsc not available; skipped typecheck"
  fi

  if ! OUTPUT=$(pnpm test 2>&1); then
    fail "Vitest"
    echo "  FIX: cd web && pnpm test — fix unit tests."
    printf "%s\n" "$OUTPUT" >&2
  else
    pass "Vitest: OK"
  fi

  popd >/dev/null || true
  printf "\n"
fi

# ============================================================
# 5. PRIVACY + SECRETS (fast + full)
# ============================================================
info "Privacy (no real emails)..."
EMAIL_PATTERN='[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}'
EXCLUDE_PATTERN='example\.com|example\.org|test\.com|\.git|target|\.opencode|\.mimocode|node_modules|playwright-report|test-results|\.md|references|agents-docs|\.agents'

if grep -rE "$EMAIL_PATTERN" \
  --exclude-dir=.git --exclude-dir=target --exclude-dir=.opencode --exclude-dir=.mimocode \
  --exclude-dir=node_modules --exclude-dir=references --exclude-dir=playwright-report \
  --exclude-dir=test-results --exclude-dir=.agents \
  . 2>/dev/null | grep -vE "$EXCLUDE_PATTERN"; then
  fail "Email address detected"
  echo "  FIX: Remove personal emails; use example.com placeholders."
else
  pass "Privacy: OK"
fi
printf "\n"

info "Secret scan..."
SECRET_PATTERN="(api_key|token|secret|password|auth|key)[[:space:]]*[:=][[:space:]]*['\"][a-zA-Z0-9_\-]{16,}['\"]"
EXCLUDE_SECRET='example\.com|example\.org|test\.com|GITHUB_TOKEN|CARGO_REGISTRY_TOKEN|worktree|shared-key|release-workflow'

if grep -rE "$SECRET_PATTERN" \
  --exclude-dir=.git --exclude-dir=target --exclude-dir=.agents --exclude-dir=.opencode \
  --exclude-dir=node_modules --exclude-dir=playwright-report --exclude-dir=test-results \
  . 2>/dev/null | grep -vE "$EXCLUDE_SECRET"; then
  fail "Potential secret detected"
  echo "  FIX: Remove secrets; use env vars / GitHub Actions secrets."
else
  pass "Secret scan: OK"
fi
printf "\n"

# ============================================================
# 6. FULL-ONLY: security, WASM, size, E2E
# ============================================================
if ! $FAST; then
  if command -v cargo-audit &>/dev/null; then
    info "Security audit..."
    AUDIT_OUTPUT=$(cargo audit 2>&1) && AUDIT_EXIT=$? || AUDIT_EXIT=$?
    if [ "$AUDIT_EXIT" -ne 0 ]; then
      if echo "$AUDIT_OUTPUT" | grep -q "unsupported CVSS version"; then
        warn "cargo-audit: skipped (advisory format issue)"
      else
        fail "Security audit"
        echo "  FIX: Review cargo audit output; update or yank vulnerable crates."
        printf "%s\n" "$AUDIT_OUTPUT" >&2
      fi
    else
      pass "Audit: OK"
    fi
    printf "\n"
  else
    warn "cargo-audit not installed (CI runs it)"
  fi

  if command -v cargo-deny &>/dev/null; then
    info "cargo-deny..."
    if ! OUTPUT=$(cargo deny check 2>&1); then
      fail "cargo-deny"
      echo "  FIX: Align dependencies with deny.toml licenses/sources."
      printf "%s\n" "$OUTPUT" >&2
    else
      pass "cargo-deny: OK"
    fi
    printf "\n"
  fi

  info "WASM build + size..."
  if ! OUTPUT=$(pnpm run build:wasm 2>&1); then
    fail "WASM build"
    echo "  FIX: Ensure rustup target wasm32-unknown-unknown and wasm-bindgen-cli 0.2.126 (mise.toml)."
    printf "%s\n" "$OUTPUT" >&2
  else
    pass "WASM build: OK"
    if ! OUTPUT=$(pnpm run check-size 2>&1); then
      fail "WASM size"
      echo "  FIX: Reduce binary size (opt-level z already on); avoid heavy deps in wasm path."
      printf "%s\n" "$OUTPUT" >&2
    else
      pass "WASM size: OK"
      printf "%s\n" "$OUTPUT"
    fi
  fi
  printf "\n"

  info "E2E (Playwright chromium)..."
  if [[ ! -d "$REPO_ROOT/node_modules" ]]; then
    pnpm install --frozen-lockfile 2>/dev/null || pnpm install
  fi
  if ! OUTPUT=$(npx playwright test --project=chromium 2>&1); then
    fail "E2E"
    echo "  FIX: Inspect Playwright report; fix product or test. Prefer POM helpers over waits."
    printf "%s\n" "$OUTPUT" >&2
  else
    pass "E2E: OK"
  fi
  printf "\n"
else
  info "Skipping full-tier sensors (WASM, size, E2E, audit). Run without --fast before PR."
  printf "\n"
fi

# ============================================================
# SUMMARY
# ============================================================
if [[ $FAILED -ne 0 ]]; then
  printf "${RED}─────────────────────────────────────────────────────────────────${NC}\n"
  printf "${RED}│ Quality Gate FAILED (tier=%s)%*s│${NC}\n" "$TIER" $((40 - ${#TIER})) ""
  printf "${RED}│ Self-correct using FIX: hints above, then re-run.             │${NC}\n"
  printf "${RED}─────────────────────────────────────────────────────────────────${NC}\n"
  exit 1
fi

printf "${GREEN}─────────────────────────────────────────────────────────────────${NC}\n"
printf "${GREEN}│ All quality gates PASSED (tier=%s)%*s│${NC}\n" "$TIER" $((37 - ${#TIER})) ""
printf "${GREEN}─────────────────────────────────────────────────────────────────${NC}\n"
if $FAST; then
  printf "Next: npm run gate:full before opening a PR.\n"
fi
exit 0
