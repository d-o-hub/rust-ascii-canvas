#!/usr/bin/env bash
# Resolve and validate the repository's pinned do-harness CLI.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION_FILE="$ROOT/.do-harness-version"

if [[ ! -f "$VERSION_FILE" ]]; then
  echo "[UNVERIFIED] missing $VERSION_FILE; restore the pinned do-harness version." >&2
  exit 2
fi

EXPECTED_VERSION="$(tr -d '[:space:]' < "$VERSION_FILE")"
if [[ ! "$EXPECTED_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "[UNVERIFIED] invalid do-harness version: $EXPECTED_VERSION" >&2
  exit 2
fi

resolve_candidate() {
  local candidate="${DO_HARNESS_BIN:-}"
  if [[ -z "$candidate" ]]; then
    candidate="$(command -v do-harness 2>/dev/null || true)"
  elif [[ "$candidate" != */* ]]; then
    candidate="$(command -v "$candidate" 2>/dev/null || true)"
  fi
  if [[ -z "$candidate" && -x "$HOME/.local/bin/do-harness" ]]; then
    candidate="$HOME/.local/bin/do-harness"
  fi
  printf '%s' "$candidate"
}

BIN="$(resolve_candidate)"
if [[ -z "$BIN" ]]; then
  echo "[UNVERIFIED] do-harness v$EXPECTED_VERSION is not installed." >&2
  echo "  FIX: run bash scripts/install-do-harness.sh or set DO_HARNESS_BIN." >&2
  exit 127
fi
if [[ ! -x "$BIN" ]]; then
  echo "[UNVERIFIED] do-harness is not executable: $BIN" >&2
  echo "  FIX: install v$EXPECTED_VERSION or set DO_HARNESS_BIN to an executable." >&2
  exit 127
fi

VERSION_JSON="$($BIN version --format json 2>/dev/null)" || {
  echo "[UNVERIFIED] unable to query do-harness version from $BIN." >&2
  echo "  FIX: install v$EXPECTED_VERSION and run bash scripts/check-do-harness.sh." >&2
  exit 127
}
ACTUAL_VERSION="$(python3 -c 'import json, sys; print(json.load(sys.stdin).get("version", ""))' <<<"$VERSION_JSON")"
if [[ "$ACTUAL_VERSION" != "$EXPECTED_VERSION" ]]; then
  echo "[UNVERIFIED] do-harness version mismatch: expected $EXPECTED_VERSION, found ${ACTUAL_VERSION:-unknown}." >&2
  echo "  FIX: run bash scripts/install-do-harness.sh or update .do-harness-version deliberately." >&2
  exit 127
fi

exec "$BIN" "$@"
