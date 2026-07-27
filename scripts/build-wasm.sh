#!/usr/bin/env bash
# scripts/build-wasm.sh
# Builds the WASM target and runs wasm-opt to optimize the binary.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo "Building WASM target..."
cargo build --target wasm32-unknown-unknown --release

echo "Generating WASM bindings..."
wasm-bindgen target/wasm32-unknown-unknown/release/ascii_canvas.wasm --out-dir web/pkg --target web

WASM_FILE="web/pkg/ascii_canvas_bg.wasm"

if command -v wasm-opt >/dev/null; then
  echo "Optimizing WASM binary with wasm-opt..."
  wasm-opt --enable-simd --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext -O3 "$WASM_FILE" -o "$WASM_FILE"
  echo "WASM optimization complete."
else
  # If GITHUB_ACTIONS is "true" or CI is "true"
  if [ "${GITHUB_ACTIONS:-}" = "true" ] || [ "${CI:-}" = "true" ]; then
    echo "==========================================================" >&2
    echo " ERROR: wasm-opt is REQUIRED in CI but was not found!" >&2
    echo " Ensure binaryen (wasm-opt) is installed in your runner." >&2
    echo "==========================================================" >&2
    exit 1
  elif [ "${SKIP_WASM_OPT:-}" = "1" ]; then
    echo "=========================================================="
    echo " WARNING: wasm-opt not found!"
    echo " Skipping optimization because SKIP_WASM_OPT=1 is set."
    echo " THIS BINARY IS NOT OPTIMIZED FOR PRODUCTION."
    echo "=========================================================="
  else
    echo "==========================================================" >&2
    echo " ERROR: wasm-opt (binaryen) is not installed." >&2
    echo " To ensure size and performance parity with CI, install wasm-opt." >&2
    echo "   - Via mise (recommended): run 'mise install'" >&2
    echo "   - Via npm: run 'npm install -g binaryen'" >&2
    echo "   - Via package manager: e.g. 'brew install binaryen' or 'apt install binaryen'" >&2
    echo "" >&2
    echo " If you must build without wasm-opt, run with SKIP_WASM_OPT=1:" >&2
    echo "   SKIP_WASM_OPT=1 pnpm run build:wasm" >&2
    echo "==========================================================" >&2
    exit 1
  fi
fi
