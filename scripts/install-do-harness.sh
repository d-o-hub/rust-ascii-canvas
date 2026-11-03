#!/usr/bin/env bash
# Install the pinned do-harness release outside the repository target directory.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="$(tr -d '[:space:]' < "$ROOT/.do-harness-version")"
INSTALL_DIR="${DO_HARNESS_INSTALL_DIR:-$HOME/.local/bin}"

if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "invalid do-harness version: $VERSION" >&2
  exit 2
fi

mkdir -p "$INSTALL_DIR"
if [[ -x "$INSTALL_DIR/do-harness" ]] &&
   DO_HARNESS_BIN="$INSTALL_DIR/do-harness" \
   bash "$ROOT/scripts/do-harness.sh" version --format json >/dev/null 2>&1; then
  echo "[PASS] do-harness v$VERSION already installed at $INSTALL_DIR/do-harness"
  exit 0
fi

curl -fsSL https://raw.githubusercontent.com/d-o-hub/do-harness/main/scripts/install.sh \
  | sh -s -- --version "v$VERSION" --bin-dir "$INSTALL_DIR"

DO_HARNESS_BIN="$INSTALL_DIR/do-harness" DO_HARNESS_VERSION="$VERSION" \
  bash "$ROOT/scripts/check-do-harness.sh"
