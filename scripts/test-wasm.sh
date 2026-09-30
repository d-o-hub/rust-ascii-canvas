#!/usr/bin/env bash
# These suites are CPU/JS interop only, not DOM. Playwright owns browser evidence.
# Check each target separately: a native zero-test run cannot masquerade as it.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
log="$(mktemp)"
trap 'rm -f "$log"' EXIT
run_suite() {
  WASM_BINDGEN_TEST_ONLY_NODE=1 cargo test --locked "$@" \
    --target wasm32-unknown-unknown 2>&1 | tee "$log"
  if ! grep -Eq 'test result: ok\. [1-9][0-9]* passed; 0 failed;' "$log"; then
    echo "[FAIL] WASM suite $* did not execute any passing tests." >&2
    echo 'FIX: restore test registration/annotations and the Node wasm-bindgen-test-runner.' >&2
    exit 1
  fi
}
run_suite --lib
run_suite --test wasm_tests
