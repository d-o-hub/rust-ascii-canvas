#!/usr/bin/env bash
# LOC limits for every source file the harness owns (harness L-020).
#
# WHY A SEPARATE SCRIPT
#   This check used to live inline in `scripts/quality-gates.sh`, which means it
#   ran in the developer's shell and in CI *nowhere* — L-016, the exact trap the
#   repo has now hit three times (L-013, L-014, L-016). The e2e sensor had the
#   same shape and was fixed by giving both runners one implementation
#   (L-019). Same fix here: one file, invoked by `quality-gates.sh` *and* by the
#   `web` CI job, so the two cannot drift.
#
# SCOPE
#   `src/**/*.rs` (all of it) and `web/*.ts` **excluding** `*.test.ts`. The web
#   side was previously unchecked entirely, which is how `web/events.ts` reached
#   850 lines and `web/ui.ts` 540 while `gate:fast` reported "LOC: no new
#   oversized files" — the sensor existed and was green and did not look at
#   either file.
#
# THE ALLOWLIST IS A RATCHET, NOT A PERMISSION SLIP
#   An entry is `path` (acknowledge the debt) or `path=lines` (acknowledge it
#   *and* forbid growth). With a number, the file may shrink but not grow; the
#   number is the budget, so paying debt down and forgetting to update the
#   allowlist is a failure rather than a silently renewed excuse. Entries for
#   files that no longer exist, or that are now under the limit, are reported —
#   stale allowlist entries are how an allowlist becomes a permanent exemption.
#
#   The legacy bare-path form still works, so an existing allowlist is not
#   broken by this change. Convert entries to `path=lines` to get the ratchet.
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT" || exit 1

MAX_LINES="${MAX_LINES_PER_SOURCE_FILE:-500}"
ALLOWLIST="${REPO_ROOT}/.loc-allowlist"

red=''
green=''
yellow=''
reset=''
if [[ -t 1 ]]; then
  red=$'\033[1;31m'; green=$'\033[1;32m'; yellow=$'\033[1;33m'; reset=$'\033[0m'
fi

VIOLATIONS=0
STALE=0
CHECKED=0

# Look up `path` or `path=lines` for a repo-relative path.
# Sets ALLOW_BUDGET to the pinned line count, or "" when the entry is a bare
# acknowledgement, or leaves it "absent" when the path is not listed at all.
allowlist_entry() {
  local rel="$1" spec=""
  if [[ -f "$ALLOWLIST" ]]; then
    spec="$(grep -E "^(./)?${rel//./\\.}(=|$)" "$ALLOWLIST" 2>/dev/null | head -1)"
  fi
  if [[ -z "$spec" ]]; then
    ALLOW_BUDGET="absent"
    return 0
  fi
  if [[ "$spec" == *=* ]]; then
    ALLOW_BUDGET="${spec##*=}"
  else
    ALLOW_BUDGET=""
  fi
}

check_file() {
  local file="$1" rel lines
  rel="${file#./}"
  lines="$(wc -l < "$file" 2>/dev/null | tr -d ' ')"
  [[ -z "$lines" ]] && return 0
  CHECKED=$((CHECKED + 1))

  allowlist_entry "$rel"
  local budget="$ALLOW_BUDGET"

  if [[ "$budget" == "absent" ]]; then
    # Not allowlisted: the limit is absolute.
    if [[ "$lines" -gt "$MAX_LINES" ]]; then
      printf '  %sFAIL%s   %s: %s lines (max %s)\n' "$red" "$reset" "$rel" "$lines" "$MAX_LINES"
      VIOLATIONS=$((VIOLATIONS + 1))
    fi
    return 0
  fi

  if [[ -n "$budget" ]]; then
    if [[ "$lines" -gt "$budget" ]]; then
      printf '  %sFAIL%s   %s: %s lines, budget %s (grew by %s)\n' \
        "$red" "$reset" "$rel" "$lines" "$budget" "$((lines - budget))"
      VIOLATIONS=$((VIOLATIONS + 1))
    elif [[ "$lines" -gt "$MAX_LINES" ]]; then
      printf '  %sWARN%s   %s: %s lines (acknowledge debt, shrinking toward %s)\n' \
        "$yellow" "$reset" "$rel" "$lines" "$MAX_LINES"
    else
      printf '  %sSTALE%s  %s: %s lines, now under the %s limit — drop the entry\n' \
        "$yellow" "$reset" "$rel" "$lines" "$MAX_LINES"
      STALE=$((STALE + 1))
    fi
    return 0
  fi

  # Bare acknowledgement: over the limit today, but with no pinned budget.
  printf '  %sWARN%s   %s: %s lines (allowlisted debt — do not grow; extract modules)\n' \
    "$yellow" "$reset" "$rel" "$lines"
}

echo "LOC limits (max ${MAX_LINES} lines; allowlist: .loc-allowlist)"

while IFS= read -r file; do
  [[ -z "$file" ]] && continue
  check_file "$file"
done < <(find ./src -name '*.rs' -type f 2>/dev/null | sort)

while IFS= read -r file; do
  [[ -z "$file" ]] && continue
  check_file "$file"
done < <(find ./web -maxdepth 1 -name '*.ts' -not -name '*.test.ts' -type f 2>/dev/null | sort)

# Allowlist entries that point at nothing. An allowlist nobody prunes stops
# being a record of debt and becomes a list of exemptions.
if [[ -f "$ALLOWLIST" ]]; then
  while IFS= read -r entry; do
    [[ -z "$entry" || "$entry" == \#* ]] && continue
    rel="${entry%%=*}"
    rel="${rel#./}"
    if [[ ! -f "$rel" ]]; then
      printf '  %sSTALE%s  allowlist entry "%s" — no such file. Remove it.\n' \
        "$yellow" "$reset" "$rel"
      STALE=$((STALE + 1))
    fi
  done < "$ALLOWLIST"
fi

if [[ $VIOLATIONS -eq 0 && $STALE -eq 0 ]]; then
  printf '  %sPASS%s   LOC: %s file(s) checked, none over %s\n' "$green" "$reset" "$CHECKED" "$MAX_LINES"
  exit 0
fi
if [[ $VIOLATIONS -eq 0 ]]; then
  printf '  %sWARN%s   LOC: %s stale allowlist entr(ies) — no limit exceeded, but prune them\n' \
    "$yellow" "$reset" "$STALE"
  exit 0
fi
printf '  FIX: Split oversized files into modules. An allowlist entry is not a waiver:\n'
printf '       add "path=lines" to acknowledge debt, and it may only shrink from there.\n'
printf '       Adding an entry needs an ADR (agents-docs/architecture.md).\n'
exit 1
