#!/usr/bin/env bash
# Release preflight checker (read-only).
#
# Releases are cut by the Release workflow (.github/workflows/release.yml,
# workflow_dispatch). This script no longer creates tags, pushes, or publishes
# anything — it validates that the four version pins agree and that the target
# version is not already released, then prints the next steps.
#
# Runbook: plans/RELEASING.md
# Usage:   ./scripts/release.sh
# Exit:    0 = ready to follow the runbook, 1 = fixes required.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

RED=$'\033[0;31m'; GREEN=$'\033[0;32m'; YELLOW=$'\033[1;33m'; NC=$'\033[0m'

if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
  echo "Usage: ./scripts/release.sh"
  echo "Read-only release preflight. See plans/RELEASING.md for the runbook."
  exit 0
fi
if [ "$#" -gt 0 ]; then
  echo "This script is a read-only preflight checker; it no longer creates releases."
  echo "Run it without arguments, then follow plans/RELEASING.md."
  exit 1
fi

failures=0
fail() { printf '%s✗ %s%s\n' "$RED" "$1" "$NC"; failures=$((failures + 1)); }
ok()   { printf '%s✓ %s%s\n' "$GREEN" "$1" "$NC"; }
warn() { printf '%s! %s%s\n' "$YELLOW" "$1" "$NC"; }

echo "=============================================="
echo "ASCII Canvas — release preflight (read-only)"
echo "=============================================="

# --- 1. Version pins must agree -------------------------------------------
VERSION_FILE=$(tr -d '[:space:]' < VERSION 2>/dev/null || true)
CARGO_VERSION=$(sed -nE 's/^version = "([^"]+)"/\1/p' Cargo.toml | head -1)
ROOT_PKG_VERSION=$(sed -nE 's/^[[:space:]]*"version": "([^"]+)".*/\1/p' package.json | head -1)
WEB_PKG_VERSION=$(sed -nE 's/^[[:space:]]*"version": "([^"]+)".*/\1/p' web/package.json | head -1)

if ! printf '%s' "$VERSION_FILE" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
  fail "VERSION file is missing or not valid semver (x.y.z): '${VERSION_FILE:-<empty>}'"
fi

PINS_OK=true
for entry in "VERSION:$VERSION_FILE" "Cargo.toml:$CARGO_VERSION" "package.json:$ROOT_PKG_VERSION" "web/package.json:$WEB_PKG_VERSION"; do
  file="${entry%%:*}"
  value="${entry#*:}"
  if [ "$value" != "$VERSION_FILE" ]; then
    fail "$file version '$value' does not match VERSION '$VERSION_FILE'"
    PINS_OK=false
  fi
done
if [ "$PINS_OK" = "true" ]; then
  ok "Version pins agree: $VERSION_FILE"
fi

# --- 2. Target version must not already be released ----------------------
if ! command -v gh >/dev/null 2>&1 || ! gh auth status >/dev/null 2>&1; then
  warn "gh unavailable/unauthenticated — GitHub Release state NOT verified"
else
  LATEST_TAG=$(gh release list --limit 1 --json tagName --jq '.[0].tagName' 2>/dev/null || true)
  if [ -z "$LATEST_TAG" ]; then
    warn "No GitHub Releases found — cannot compare against a baseline (L-004)"
  else
    LATEST_VERSION="${LATEST_TAG#v}"
    echo "Latest GitHub Release: $LATEST_TAG"
    if ! printf '%s' "$LATEST_VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
      warn "Latest release tag '$LATEST_TAG' is not semver — skipping version comparison"
    elif [ "$VERSION_FILE" = "$LATEST_VERSION" ]; then
      fail "Version $VERSION_FILE already has a GitHub Release ($LATEST_TAG) — dispatching Release would fail Guard Rails (L-007)"
      echo "  Fix: open a release-prep PR bumping VERSION, Cargo.toml, package.json and web/package.json together, merge it, then re-dispatch."
    elif ! printf '%s\n%s\n' "$LATEST_VERSION" "$VERSION_FILE" | sort -V -C; then
      fail "Version $VERSION_FILE is older than the latest release $LATEST_TAG"
    else
      IFS='.' read -r V_MAJOR V_MINOR V_PATCH <<< "$VERSION_FILE"
      IFS='.' read -r L_MAJOR L_MINOR L_PATCH <<< "$LATEST_VERSION"
      VALID=false
      if [ "$V_MAJOR" -gt "$L_MAJOR" ] && [ "$V_MINOR" -eq 0 ] && [ "$V_PATCH" -eq 0 ]; then
        VALID=true; KIND="major"
      elif [ "$V_MAJOR" -eq "$L_MAJOR" ] && [ "$V_MINOR" -gt "$L_MINOR" ] && [ "$V_PATCH" -eq 0 ]; then
        VALID=true; KIND="minor"
      elif [ "$V_MAJOR" -eq "$L_MAJOR" ] && [ "$V_MINOR" -eq "$L_MINOR" ] && [ "$V_PATCH" -gt "$L_PATCH" ]; then
        VALID=true; KIND="patch"
      fi
      if [ "$VALID" = "true" ]; then
        ok "v$VERSION_FILE is a valid $KIND increment over $LATEST_TAG and is not yet released"
      else
        fail "v$VERSION_FILE is not a valid major/minor/patch increment over $LATEST_TAG"
      fi
    fi
  fi
fi

# --- 3. Summary ------------------------------------------------------------
echo
if [ "$failures" -gt 0 ]; then
  echo "Preflight FAILED ($failures issue(s)). Runbook: plans/RELEASING.md"
  exit 1
fi

echo "Preflight OK."
echo "Next steps:"
echo "  1. Open/merge the release-prep PR (must be green: npm run gate:fast)."
echo "  2. gh workflow run release.yml -f dry_run=true && gh run watch"
echo "  3. gh workflow run release.yml && gh run watch"
echo "  4. gh release view v$VERSION_FILE"
echo "Runbook: plans/RELEASING.md"
