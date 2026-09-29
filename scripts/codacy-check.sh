#!/usr/bin/env bash
# Codacy repo-level intake check (harness L-017).
#
# WHY THIS EXISTS
#   Codacy's *pull-request* analysis is diff-scoped. PR #220 reported
#   `newIssues: 0` / `isUpToStandards: true` on a PR whose repository still
#   carried 11 High findings, and PR #222 merged with the Codacy check green
#   while those 11 sat on the repo-level backlog. A PR-scoped read can never
#   see a pre-existing issue, so "the check is green" was compatible with an
#   arbitrarily large backlog. This script reads the repository-level list,
#   which is the only place a backlog is visible.
#
# WHAT THIS IS *NOT*
#   It is an AGENT PROCEDURE, not a gate. It runs on a developer machine (or in
#   an agent session) with whatever credential the local `codacy` CLI holds; it
#   is not wired into `ci.yml` and it is not part of `scripts/quality-gates.sh`.
#   Those are different things and the difference matters (L-016): a check that
#   only runs on one machine fails on one machine and merges anyway.
#
#   It is also not a substitute for the required check. `Codacy Static Code
#   Analysis` in the `main` ruleset is what blocks a merge; this reads the same
#   data so a backlog is visible *before* a PR is opened, not only after.
#
# CREDENTIALS (the part that is easy to get wrong — see L-017)
#   `codacy login` stores an **account** API token on this machine, encrypted,
#   at ~/.codacy/credentials. That is why the CLI works here. It is invisible to
#   a GitHub Actions runner, which can read only repository secrets — so a CI
#   job needs `CODACY_API_TOKEN` (read) or `CODACY_PROJECT_TOKEN`
#   (read + coverage upload) as a repository secret. Do not put an account token
#   in CI: the CLI itself refuses to fall back to one for scoped work. As of
#   2026-09-29 this repository has no secrets set, so there is no CI wiring.
#
# USAGE
#   scripts/codacy-check.sh                 # fail on Critical/High
#   scripts/codacy-check.sh --level Medium  # widen the threshold
#   scripts/codacy-check.sh --level-info    # list everything, never fails
#   scripts/codacy-check.sh --pr 222        # diff-scoped view of one PR
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT" || exit 1

LEVEL="High"
PR=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --level) LEVEL="${2:-}"; shift 2 || true ;;
    --level-info) LEVEL="Info"; shift ;;
    --pr) PR="${2:-}"; shift 2 || true ;;
    -h|--help) sed -n '1,40p' "${BASH_SOURCE[0]}"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

if [[ -n "$PR" ]]; then
  command -v codacy >/dev/null 2>&1 || { echo "[WARN] codacy CLI not found."; exit 0; }
  echo "Codacy diff-scoped view — PR #$PR (new issues only, cannot see a backlog):"
  read -r -d '' PY_PR <<'PYCODE' || true
import json, sys
d = json.load(sys.stdin)
pr = d.get("pullRequest", {})
print("  isUpToStandards:", pr.get("isUpToStandards"), "  newIssues:", pr.get("newIssues"))
for item in d.get("newIssues", []):
    ci = item.get("commitIssue", {})
    pat = ci.get("patternInfo", {})
    loc = "%s:%s" % (ci.get("filePath"), ci.get("lineNumber"))
    print("  -", pat.get("id"), loc, "|", ci.get("message"))
PYCODE
  codacy -o json pull-request "$PR" 2>/dev/null | python3 -c "$PY_PR"
  exit 0
fi

# Not a failure: the Codacy CLI is not a project dependency and must not become
# one. Absence means "not checked", which is different from "checked and clean" —
# so it warns, and never prints a pass.
if ! command -v codacy >/dev/null 2>&1; then
  echo "[WARN] codacy CLI not found — repo-level intake NOT checked (this is not a pass)."
  echo "       npm install -g @codacy/codacy-cloud-cli && codacy login"
  exit 0
fi

case "$LEVEL" in
  Info|Minor|Medium|High|Critical) ;;
  *) echo "unknown severity: $LEVEL (expected Info|Minor|Medium|High|Critical)" >&2; exit 2 ;;
esac

raw="$(codacy -o json issues 2>/dev/null)"
if [[ -z "$raw" ]]; then
  echo "[WARN] codacy CLI returned nothing — repo-level intake NOT checked (this is not a pass)."
  echo "       Authenticate with 'codacy login' or set CODACY_API_TOKEN."
  exit 0
fi

read -r -d '' PY_REPORT <<'PYCODE' || true
import json, os, sys

rank = {"Info": 0, "Minor": 1, "Medium": 2, "High": 3, "Critical": 4}
threshold = rank[os.environ["CODACY_LEVEL"]]
issues = json.load(sys.stdin).get("issues", [])

if not issues:
    print("  0 open issues — repo-level backlog clear.")
    sys.exit(0)

def sev_of(item):
    return item.get("patternInfo", {}).get("severityLevel", "Info")

issues.sort(key=lambda i: -rank.get(sev_of(i), -1))
failing = [i for i in issues if rank.get(sev_of(i), -1) >= threshold]

for item in issues:
    pat = item.get("patternInfo", {})
    sev = sev_of(item)
    mark = "FAIL" if rank.get(sev, -1) >= threshold else "info"
    print("  [%s] %-8s %s %s:%s" % (mark, sev, pat.get("id"), item.get("filePath"), item.get("lineNumber")))

print("  %d open, %d at or above %s." % (len(issues), len(failing), os.environ["CODACY_LEVEL"]))
sys.exit(1 if failing else 0)
PYCODE

echo "Codacy repo-level intake (fail on $LEVEL and above):"
CODACY_LEVEL="$LEVEL" python3 -c "$PY_REPORT" <<<"$raw"
exit $?
