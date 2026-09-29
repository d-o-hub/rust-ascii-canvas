#!/usr/bin/env bash
# npm-side advisory audit (harness L-013).
#
# WHY THIS EXISTS
#   `quality-gates.sh` has printed `[PASS] Audit: OK` since ADR-044 — and it runs
#   `cargo audit` only. Its name is ecosystem-neutral, so a PR that refreshes
#   ~250 npm dependency resolutions reads as "audited" when no package was
#   checked. The R-05 roast found it: `grep -rn 'pnpm audit\|npm audit' .github/ scripts/`
#   returned nothing, i.e. the repository had **no** npm advisory sensor at all.
#   npm findings were caught only reactively, by Dependabot opening an alert —
#   a *reporting* channel, not a gate, and one that never fails a build.
#
#   This matters more than usual for a WASM app: the 2026 Vite dev-server
#   advisories (GHSA-v2wj-q39q-566r, GHSA-p9ff-h696-f583, CVE-2026-53571) all
#   list "exposes the dev server to the network" as a precondition, and R-06
#   removed that precondition. An advisory sensor is what would have *told* us
#   the exposure mattered before anyone reasoned about it.
#
# BOTH LOCKFILES
#   The root and `web/` have separate `pnpm-lock.yaml` files and are installed
#   separately (`pnpm install --frozen-lockfile` at each level, in that order, in
#   the CI `web` job). Auditing only one would leave half the dependency tree
#   unchecked — so both are audited here, and the first failure is reported.
#
# WHY THIS IS *NOT* IN ci.yml
#   `pnpm audit` needs the npm registry, and the gate script is a developer
#   convenience (L-016). The CI side lives in a dedicated `security-npm` job in
#   `.github/workflows/ci.yml`; keep the two in step, and grep the workflow to
#   confirm rather than trusting this file.
#
# THRESHOLD
#   `--audit-level=high` by default. Moderate advisories in a dev-only
#   transitive tree are noise (R-05's lockfile refresh surfaced several), and a
#   sensor that cries wolf gets ignored. `AUDIT_LEVEL=critical` to tighten,
#   `AUDIT_LEVEL=moderate` to widen.
#
# OFFLINE
#   A registry failure is reported as a skip, never as a pass. `--offline` is
#   deliberately not used: it would turn "could not check" into "nothing found".
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT" || exit 1

LEVEL="${AUDIT_LEVEL:-high}"
case "$LEVEL" in
  low|moderate|high|critical) ;;
  *) echo "AUDIT_LEVEL must be low|moderate|high|critical (got '$LEVEL')" >&2; exit 2 ;;
esac

if ! command -v pnpm >/dev/null 2>&1; then
  echo "[WARN] pnpm not found — npm advisories NOT checked (this is not a pass)."
  exit 0
fi

FAILED=0
CHECKED=0

audit_one() {
  local dir="$1" label="$2" output rc
  if [[ "$dir" != "." && ! -f "$dir/pnpm-lock.yaml" ]]; then
    echo "  $label: no lockfile — skipped"
    return 0
  fi

  output="$(cd "$dir" && pnpm audit --audit-level="$LEVEL" 2>&1)"
  rc=$?

  if [[ $rc -eq 0 ]]; then
    echo "  $label: no advisories at or above $LEVEL"
    CHECKED=$((CHECKED + 1))
    return 0
  fi

  # Distinguish "found something" from "could not ask the registry". pnpm exits
  # non-zero for both, and a network error must not read as a vulnerability.
  if grep -qiE "ENOTFOUND|EAI_AGAIN|ECONNREFUSED|ETIMEDOUT|network|registry\.npmjs\.org|offline" <<<"$output"; then
    echo "  $label: registry unreachable — NOT CHECKED (this is not a pass)"
    return 0
  fi

  echo "  $label: advisories at or above $LEVEL"
  printf '%s\n' "$output" | sed 's/^/    /'
  FAILED=1
  CHECKED=$((CHECKED + 1))
  return 1
}

echo "npm advisories (pnpm audit --audit-level=$LEVEL, root + web):"
audit_one "." "root" || true
audit_one "web" "web " || true

if [[ $CHECKED -eq 0 ]]; then
  echo "  nothing was checked — do not read this as a pass"
  exit 0
fi
if [[ $FAILED -ne 0 ]]; then
  exit 1
fi
echo "  $CHECKED lockfile(s) checked, clean at or above $LEVEL"
exit 0
