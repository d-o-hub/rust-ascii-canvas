#!/usr/bin/env bash
# Offline fixtures for pinned do-harness discovery and fallback diagnostics.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

fail() {
  echo "[FAIL] $1" >&2
  exit 1
}

contains() {
  local haystack="$1" needle="$2"
  [[ "$haystack" == *"$needle"* ]] || fail "expected output to contain: $needle"
}

FAKE="$TMP/fake-do-harness"
LOG="$TMP/invoked"
cat > "$FAKE" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$*" > "$DO_HARNESS_TEST_LOG"
case "$1" in
  version) printf '{"version":"%s"}\n' "${FAKE_VERSION:-0.4.1}" ;;
  list) printf '%s\n' "${FAKE_SETS:-[\"feedback\",\"verification\",\"release\"]}" ;;
  *) exit 2 ;;
esac
EOF
chmod +x "$FAKE"

output="$(DO_HARNESS_BIN="$FAKE" DO_HARNESS_TEST_LOG="$LOG" \
  bash "$ROOT/scripts/do-harness.sh" list --sets --format json)"
contains "$output" 'feedback'
[[ "$(cat "$LOG")" == "list --sets --format json" ]] || fail "resolver did not forward arguments"

set +e
output="$(DO_HARNESS_BIN="$FAKE" DO_HARNESS_TEST_LOG="$LOG" FAKE_VERSION=0.4.0 \
  bash "$ROOT/scripts/do-harness.sh" list --sets --format json 2>&1)"
status=$?
set -e
[[ "$status" -eq 127 ]] || fail "version mismatch returned $status"
contains "$output" "version mismatch"

set +e
output="$(DO_HARNESS_BIN="$TMP/missing" bash "$ROOT/scripts/do-harness.sh" \
  list --sets --format json 2>&1)"
status=$?
set -e
[[ "$status" -eq 127 ]] || fail "missing binary returned $status"
contains "$output" "not executable"

output="$(DO_HARNESS_BIN="$FAKE" DO_HARNESS_TEST_LOG="$LOG" \
  bash "$ROOT/scripts/check-do-harness.sh")"
contains "$output" "signal sets"

set +e
output="$(DO_HARNESS_BIN="$FAKE" DO_HARNESS_TEST_LOG="$LOG" \
  FAKE_SETS='["verification"]' bash "$ROOT/scripts/check-do-harness.sh" 2>&1)"
status=$?
set -e
[[ "$status" -ne 0 ]] || fail "missing signal set passed"
contains "$output" "signal-set mismatch"

grep -Fq 'do-harness/v$VERSION/scripts/install.sh' "$ROOT/scripts/install-do-harness.sh" || \
  fail "installer is not pinned to the release tag"
if grep -Fq 'do-harness/main/scripts/install.sh' "$ROOT/scripts/install-do-harness.sh"; then
  fail "installer uses moving main branch"
fi

set +e
output="$(DO_HARNESS_BIN="$TMP/missing" DO_HARNESS_NO_FALLBACK=1 \
  bash "$ROOT/scripts/quality-gates.sh" --fast 2>&1)"
status=$?
set -e
[[ "$status" -eq 127 ]] || fail "fail-closed fallback returned $status"
contains "$output" "fallback is disabled"

echo "[PASS] do-harness resolver, contract, installer, and fallback fixtures"
