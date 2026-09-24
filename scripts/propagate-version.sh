#!/usr/bin/env bash
# scripts/propagate-version.sh
# Single-source version propagation.
#
# VERSION is the single source of truth for the project version. This script
# either propagates it to every pinned file, or verifies that they all agree:
#
#   ./scripts/propagate-version.sh          # write VERSION into all pins
#   ./scripts/propagate-version.sh --check  # verify pins agree (no writes)
#
# Pins:
#   - Cargo.toml         ([package] version)
#   - package.json       (top-level "version")
#   - web/package.json   (top-level "version")
#
# `--check` runs in the fast quality tier (scripts/quality-gates.sh section 2c).
# Runbook: plans/RELEASING.md
#
# Exit 0 = success, 1 = mismatch or parse error.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

RED=$'\033[0;31m'; GREEN=$'\033[0;32m'; NC=$'\033[0m'

CHECK_ONLY=false
for arg in "$@"; do
  case $arg in
    --check) CHECK_ONLY=true ;;
    -h|--help) sed -n '2,18p' "$0"; exit 0 ;;
    *) echo "Unknown argument: $arg (try --check, --help)"; exit 1 ;;
  esac
done

ok()   { printf '%s✓%s %s\n' "$GREEN" "$NC" "$1"; }
fail() { printf '%s✗%s %s\n' "$RED" "$NC" "$1"; }

# --- Single source -----------------------------------------------------------
if [[ ! -f VERSION ]]; then
  fail "VERSION file not found"
  exit 1
fi
VERSION_VALUE="$(tr -d '[:space:]' < VERSION)"
if ! printf '%s' "$VERSION_VALUE" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
  fail "VERSION is not valid semver (x.y.z): '${VERSION_VALUE:-<empty>}'"
  exit 1
fi

# --- Pin readers/writers (awk: portable, section-aware) ----------------------
read_cargo_pin() {
  awk '
    /^\[package\]/ { in_pkg = 1; next }
    /^\[/          { in_pkg = 0 }
    in_pkg && /^version[[:space:]]*=/ {
      if (match($0, /"[^"]+"/)) {
        print substr($0, RSTART + 1, RLENGTH - 2)
        exit
      }
    }
  ' Cargo.toml
}

read_json_pin() {
  sed -n 's/^[[:space:]]*"version"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$1" | head -n 1
}

write_cargo_pin() {
  local tmp="Cargo.toml.tmp"
  awk -v v="$VERSION_VALUE" '
    /^\[package\]/ { in_pkg = 1; print; next }
    /^\[/          { in_pkg = 0; print; next }
    {
      if (in_pkg && !done && /^version[[:space:]]*=/) {
        print "version = \"" v "\""
        done = 1
      } else {
        print
      }
    }
  ' Cargo.toml > "$tmp"
  mv "$tmp" Cargo.toml
}

write_json_pin() {
  local file="$1" tmp="$1.tmp"
  awk -v v="$VERSION_VALUE" '
    !done && /^[[:space:]]*"version"[[:space:]]*:/ {
      sub(/:[[:space:]]*"[^"]*"/, ": \"" v "\"")
      done = 1
    }
    { print }
  ' "$file" > "$tmp"
  mv "$tmp" "$file"
}

read_pin() {
  case "$1" in
    Cargo.toml) read_cargo_pin ;;
    *)          read_json_pin "$1" ;;
  esac
}

write_pin() {
  case "$1" in
    Cargo.toml) write_cargo_pin ;;
    *)          write_json_pin "$1" ;;
  esac
}

PINS=(Cargo.toml package.json web/package.json)

echo "=============================================="
echo "Version propagation (single source: VERSION)"
echo "=============================================="
echo "VERSION: $VERSION_VALUE"
echo ""

# --- Check mode ---------------------------------------------------------------
if $CHECK_ONLY; then
  failures=0
  for pin_file in "${PINS[@]}"; do
    if [[ ! -f "$pin_file" ]]; then
      fail "$pin_file: missing"
      failures=$((failures + 1))
      continue
    fi
    pin_value="$(read_pin "$pin_file")"
    if [[ -z "$pin_value" ]]; then
      fail "$pin_file: could not read version"
      failures=$((failures + 1))
    elif [[ "$pin_value" != "$VERSION_VALUE" ]]; then
      fail "$pin_file: '$pin_value' != VERSION '$VERSION_VALUE'"
      failures=$((failures + 1))
    else
      ok "$pin_file: $pin_value"
    fi
  done
  echo ""
  if [[ "$failures" -gt 0 ]]; then
    echo "FIX: run ./scripts/propagate-version.sh to propagate VERSION to all pins."
    echo "Runbook: plans/RELEASING.md"
    exit 1
  fi
  ok "all pins agree with VERSION ($VERSION_VALUE)"
  exit 0
fi

# --- Write mode ---------------------------------------------------------------
changed=0
failures=0
for pin_file in "${PINS[@]}"; do
  if [[ ! -f "$pin_file" ]]; then
    fail "$pin_file: missing"
    failures=$((failures + 1))
    continue
  fi
  pin_value="$(read_pin "$pin_file")"
  if [[ "$pin_value" == "$VERSION_VALUE" ]]; then
    ok "$pin_file: already at $VERSION_VALUE"
    continue
  fi
  write_pin "$pin_file"
  new_value="$(read_pin "$pin_file")"
  if [[ "$new_value" != "$VERSION_VALUE" ]]; then
    fail "$pin_file: update failed (still '$new_value')"
    failures=$((failures + 1))
  else
    ok "$pin_file: $pin_value -> $VERSION_VALUE"
    changed=$((changed + 1))
  fi
done
echo ""
if [[ "$failures" -gt 0 ]]; then
  echo "Propagation FAILED ($failures file(s))."
  exit 1
fi
if [[ "$changed" -gt 0 ]]; then
  ok "propagated $VERSION_VALUE to $changed file(s)"
else
  ok "nothing to do — all pins already at $VERSION_VALUE"
fi
