#!/usr/bin/env bash
# scripts/quality-gates.sh
# Quality gate for the ASCII Canvas project.
# Usage: ./scripts/quality-gates.sh [--fix]
# Exit 0 = success, Exit 1 = errors.
set +e
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT" || exit 1

readonly GIT_EXCLUDE="./.git/*"
readonly MAX_LINES_PER_SOURCE_FILE=500

FIX=false
for arg in "$@"; do
  case $arg in
    --fix) FIX=true ;;
    *) echo "Unknown argument: $arg"; exit 1 ;;
  esac
done

if [[ -t 1 ]] && [[ "${FORCE_COLOR:-}" != "0" ]]; then
  RED='\033[0;31m'
  GREEN='\033[0;32m'
  YELLOW='\033[1;33m'
  BLUE='\033[0;34m'
  NC='\033[0m'
else
  RED=''
  GREEN=''
  YELLOW=''
  BLUE=''
  NC=''
fi

pass() { echo -e "${GREEN}[PASS]${NC} $1"; }
fail() { echo -e "${RED}[FAIL]${NC} $1"; FAILED=1; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
info() { echo -e "${BLUE}[INFO]${NC} $1"; }

FAILED=0

printf "Running quality gate...\n\n"

# ============================================================
# 1. LOC LIMITS
# ============================================================
info "Enforcing LOC limits (max ${MAX_LINES_PER_SOURCE_FILE} lines per file)..."
LOC_VIOLATIONS=0
while IFS= read -r file; do
  lines=$(wc -l < "$file" 2>/dev/null || echo 0)
  if [[ "$lines" -gt "$MAX_LINES_PER_SOURCE_FILE" ]]; then
    warn "  $file: $lines lines (max $MAX_LINES_PER_SOURCE_FILE)"
    LOC_VIOLATIONS=$((LOC_VIOLATIONS + 1))
  fi
done < <(find . -name "*.rs" -not -path "./target/*" -not -path "./.git/*" -type f 2>/dev/null)

if [[ $LOC_VIOLATIONS -gt 0 ]]; then
  fail "LOC: $LOC_VIOLATIONS files exceed ${MAX_LINES_PER_SOURCE_FILE} lines"
else
  pass "LOC: All source files within limit"
fi
printf "\n"

# ============================================================
# 2. RUST CHECKS
# ============================================================
info "Running Rust checks..."

# Format
if $FIX; then
  cargo fmt --all
  pass "Format: auto-fixed"
else
  if ! OUTPUT=$(cargo fmt --all -- --check 2>&1); then
    fail "Format: run 'cargo fmt --all' to fix"
    printf "%s\n" "$OUTPUT" >&2
  else
    pass "Format: OK"
  fi
fi

# Clippy
if $FIX; then
  cargo clippy --fix --allow-dirty --allow-staged --all-targets --all-features
  pass "Clippy: auto-fixed"
else
  if ! OUTPUT=$(cargo clippy --all-targets --all-features -- -D warnings 2>&1); then
    fail "Clippy: fix lint errors above"
    printf "%s\n" "$OUTPUT" >&2
  else
    pass "Clippy: OK"
  fi
fi

# Build
if ! OUTPUT=$(cargo build --all-targets 2>&1); then
  fail "Build: failed"
  printf "%s\n" "$OUTPUT" >&2
else
  pass "Build: OK"
fi

# Tests
if ! OUTPUT=$(cargo test --all 2>&1); then
  fail "Tests: failed"
  printf "%s\n" "$OUTPUT" >&2
else
  pass "Tests: OK"
fi

# WASM Tests (skip if wasm-bindgen-test-runner not available)
if command -v wasm-bindgen-test-runner &>/dev/null; then
  if ! OUTPUT=$(cargo test --all --target wasm32-unknown-unknown 2>&1); then
    fail "WASM Tests: failed"
    printf "%s\n" "$OUTPUT" >&2
  else
    pass "WASM Tests: OK"
  fi
else
  warn "WASM Tests: skipped (wasm-bindgen-test-runner not found)"
fi
printf "\n"

# ============================================================
# 3. SECURITY AUDIT
# ============================================================
if command -v cargo-audit &>/dev/null; then
  info "Running security audit..."
  AUDIT_OUTPUT=$(cargo audit 2>&1) && AUDIT_EXIT=$? || AUDIT_EXIT=$?
  if [ $AUDIT_EXIT -ne 0 ]; then
    if echo "$AUDIT_OUTPUT" | grep -q "unsupported CVSS version"; then
      warn "cargo-audit: Skipping due to RustSec advisory format issue"
    else
      fail "Security audit: vulnerabilities found"
    fi
  else
    pass "Audit: OK"
  fi
  printf "\n"
fi

# ============================================================
# 4. PRIVACY CHECK (No emails)
# ============================================================
info "Checking for email addresses (privacy-first)..."
EMAIL_PATTERN='[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}'
EXCLUDE_PATTERN='example\.com|example\.org|test\.com|\.git|target|\.opencode|\.mimocode|\.md|references'

if grep -rE "$EMAIL_PATTERN" \
  --exclude-dir=.git --exclude-dir=target --exclude-dir=.opencode --exclude-dir=.mimocode \
  --exclude-dir=references \
  . 2>/dev/null | grep -vE "$EXCLUDE_PATTERN"; then
  fail "Email address detected in codebase"
else
  pass "Privacy: OK"
fi
printf "\n"

# ============================================================
# 5. SECRET SCAN
# ============================================================
info "Scanning for potential secrets..."
SECRET_PATTERN="(api_key|token|secret|password|auth|key)[[:space:]]*[:=][[:space:]]*['\"][a-zA-Z0-9_\-]{16,}['\"]"
EXCLUDE_DIR='--exclude-dir=.git --exclude-dir=target --exclude-dir=.agents --exclude-dir=.opencode'
EXCLUDE_SECRET='example\.com|example\.org|test\.com|GITHUB_TOKEN|CARGO_REGISTRY_TOKEN|worktree|shared-key|release-workflow'

if grep -rE "$SECRET_PATTERN" $EXCLUDE_DIR . 2>/dev/null | grep -vE "$EXCLUDE_SECRET"; then
  fail "Potential secret detected in codebase"
else
  pass "Secret Scan: OK"
fi
printf "\n"

# ============================================================
# SUMMARY
# ============================================================
if [[ $FAILED -ne 0 ]]; then
  printf "${RED}─────────────────────────────────────────────────────────────────${NC}\n"
  printf "${RED}│ Quality Gate FAILED                                           │${NC}\n"
  printf "${RED}─────────────────────────────────────────────────────────────────${NC}\n"
  exit 1
fi

printf "${GREEN}─────────────────────────────────────────────────────────────────${NC}\n"
printf "${GREEN}│ All Quality Gates PASSED                                      │${NC}\n"
printf "${GREEN}─────────────────────────────────────────────────────────────────${NC}\n"
