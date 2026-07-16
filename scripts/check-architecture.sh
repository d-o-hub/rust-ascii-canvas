#!/usr/bin/env bash
# Architecture fitness sensor (computational feedback).
# Enforces layer import rules — see agents-docs/architecture.md.
#
# Exit 0 = OK, Exit 1 = violation.
# Messages are written for agent self-correction.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT" || exit 1

FAILED=0

if [[ -t 1 ]] && [[ "${FORCE_COLOR:-}" != "0" ]]; then
  RED='\033[0;31m'; GREEN='\033[0;32m'; NC='\033[0m'
else
  RED=''; GREEN=''; NC=''
fi

fail() {
  echo -e "${RED}[FAIL]${NC} $1"
  FAILED=1
}

pass() {
  echo -e "${GREEN}[PASS]${NC} $1"
}

# Returns matching lines of illegal imports in a directory.
# Args: dir pattern description fix_hint
check_forbidden_imports() {
  local dir="$1"
  local pattern="$2"
  local label="$3"
  local fix="$4"

  if [[ ! -d "$dir" ]]; then
    return 0
  fi

  local matches
  matches="$(rg -n --type rust "$pattern" "$dir" 2>/dev/null || true)"
  if [[ -n "$matches" ]]; then
    fail "$label"
    echo "$matches"
    echo "  FIX: $fix"
    echo "  See agents-docs/architecture.md"
  fi
}

echo "Architecture fitness check..."
echo

# core → must not depend on wasm, render, ui
check_forbidden_imports \
  "src/core" \
  'use crate::(wasm|render|ui)(::|;)' \
  "src/core must not import wasm/render/ui (keep core pure)" \
  "Move shared types into core or utils; keep WASM/render out of core."

# utils → leaf: avoid pulling higher layers (and prefer not depending on core heavily)
check_forbidden_imports \
  "src/utils" \
  'use crate::(wasm|render|ui)(::|;)' \
  "src/utils must not import wasm/render/ui" \
  "Utils must remain a leaf module."

# render → no wasm, no ui
check_forbidden_imports \
  "src/render" \
  'use crate::(wasm|ui)(::|;)' \
  "src/render must not import wasm/ui" \
  "Render may use core/utils only; put JS interop in wasm."

# ui → no wasm, no render
check_forbidden_imports \
  "src/ui" \
  'use crate::(wasm|render)(::|;)' \
  "src/ui must not import wasm/render" \
  "UI config stays free of renderer and WASM bindings."

# web-sys / wasm-bindgen must not appear in core
if [[ -d src/core ]]; then
  matches="$(rg -n 'wasm_bindgen|web_sys|js_sys' src/core --type rust 2>/dev/null || true)"
  if [[ -n "$matches" ]]; then
    fail "src/core must not use wasm-bindgen/web-sys/js-sys"
    echo "$matches"
    echo "  FIX: Confine browser APIs to src/wasm/. Core must stay pure Rust."
  fi
fi

if [[ "$FAILED" -ne 0 ]]; then
  echo
  echo "Architecture fitness FAILED. Fix layer violations before continuing."
  exit 1
fi

pass "Layer import rules OK"
exit 0
