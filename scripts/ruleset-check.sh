#!/usr/bin/env bash
# Repository ruleset drift sensor (computational feedback).
#
# WHY THIS EXISTS
#   The `main` ruleset is *repository state*, not a file. `git revert` does not
#   undo it, a code review does not see it, and nothing in CI noticed when it
#   drifted. That invisibility is how PRs merged with red E2E for years: the
#   ruleset required one check and nobody could tell by reading the diff.
#   (harness L-008)
#
#   This makes the ruleset a reviewed artifact: `.github/ruleset-main.json` is
#   the committed, canonical projection of the merge contract, so a change to
#   the real ruleset appears in a PR diff — and can be restored.
#
# Usage:
#   ./scripts/ruleset-check.sh            # check for drift (exit 1 on drift)
#   ./scripts/ruleset-check.sh --diff     # show the drift as a unified diff
#   ./scripts/ruleset-check.sh --update   # accept live state into the snapshot
#   ./scripts/ruleset-check.sh --restore  # push the snapshot back to GitHub
#
# Exit codes:
#   0 = live ruleset matches the snapshot
#   1 = DRIFT, or no ruleset exists at all
#   2 = could not verify (no gh/auth/network) — NOT the same as "matches"
#
# Changing the merge contract is deliberate: it needs an ADR
# (goap-adr-planner) plus an updated snapshot in the same change. Relaxing a
# guard-rail here is exactly the L-008 failure mode.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SNAPSHOT="$ROOT/.github/ruleset-main.json"
RED=$'\033[0;31m'; NC=$'\033[0m'
[[ -t 1 ]] || { RED=''; NC=''; }

MODE="check"
case "${1:-}" in
  --diff) MODE="diff" ;;
  --update) MODE="update" ;;
  --restore) MODE="restore" ;;
  "") MODE="check" ;;
  -h|--help) sed -n '2,33p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
  *) echo "Unknown argument: $1 (try --diff, --update, --restore, --help)" >&2; exit 1 ;;
esac

if ! command -v gh >/dev/null 2>&1 || ! command -v jq >/dev/null 2>&1; then
  echo "ERROR: gh and jq are required; the ruleset cannot be verified without them." >&2
  echo "  FIX: install them. Absent tooling means UNKNOWN, not 'passing'." >&2
  exit 2
fi
if ! gh auth status >/dev/null 2>&1; then
  echo "ERROR: gh is not authenticated — ruleset state UNVERIFIED." >&2
  exit 2
fi

REPO="$(gh repo view --json nameWithOwner --jq .nameWithOwner 2>/dev/null)" || {
  echo "ERROR: cannot determine the repository." >&2; exit 2; }

# Resolve the ruleset covering the default branch. Do not hardcode the id: ids
# change when a ruleset is recreated, and a stale id must not make this check
# vacuous. Note the LIST endpoint does not include `conditions`, so each
# candidate has to be fetched in full to test its ref filter.
RULESET_ID=""
for id in $(gh api "repos/$REPO/rulesets" --jq \
    '.[] | select(.target=="branch" and .enforcement=="active") | .id' 2>/dev/null); do
  if gh api "repos/$REPO/rulesets/$id" --jq \
      'any(.conditions.ref_name.include[]?; .=="~DEFAULT_BRANCH")' 2>/dev/null | grep -q true; then
    RULESET_ID="$id"
    break
  fi
done
if [[ -z "$RULESET_ID" ]]; then
  echo "RULESET MISSING — nothing blocks a merge on the default branch." >&2
  echo "  A deleted or disabled ruleset is a Blocker, not a nit: CI runs but" >&2
  echo "  gates nothing. Recreate it per ADR-044, then --update the snapshot." >&2
  exit 1
fi



# Project the raw ruleset down to the fields that define the merge contract.
# Volatile fields (id, node_id, timestamps, bypass actors) are excluded on
# purpose, so the snapshot only changes when *policy* changes.
canonicalize() {
  jq '{
    enforcement,
    refs: (.conditions.ref_name.include // [] | sort),
    requiredStatusChecks:
      ([ .rules[] | select(.type=="required_status_checks")
          | .parameters.required_status_checks[]?.context ] | sort),
    strictRequiredStatusChecks:
      ([ .rules[] | select(.type=="required_status_checks")
          | .parameters.strict_required_status_checks_policy ] | first // false),
    requiredReviewThreadResolution:
      ([ .rules[] | select(.type=="pull_request")
          | .parameters.required_review_thread_resolution ] | first // false),
    requiredApprovingReviewCount:
      ([ .rules[] | select(.type=="pull_request")
          | .parameters.required_approving_review_count ] | first // 0),
    allowedMergeMethods:
      ([ .rules[] | select(.type=="pull_request")
          | .parameters.allowed_merge_methods[]? ] | sort),
    requiresLinearHistory: any(.rules[]; .type=="required_linear_history"),
    allowsForcePush: (any(.rules[]; .type=="non_fast_forward") | not),
    allowsDeletion: (any(.rules[]; .type=="deletion") | not)
  }'
}

LIVE_RAW="$(gh api "repos/$REPO/rulesets/$RULESET_ID" 2>/dev/null)" || {
  echo "ERROR: could not read ruleset $RULESET_ID." >&2; exit 2; }
# -S sorts object keys so the comparison is stable regardless of jq's key order.
LIVE="$(canonicalize <<<"$LIVE_RAW" | jq -S .)"

if [[ "$MODE" == "update" ]]; then
  printf '%s\n' "$LIVE" > "$SNAPSHOT"
  echo "Snapshot updated from live ruleset $RULESET_ID -> $SNAPSHOT"
  echo "Review that diff: a merge-contract change needs an ADR and a human."
  exit 0
fi

if [[ ! -f "$SNAPSHOT" ]]; then
  echo "ERROR: no snapshot at $SNAPSHOT" >&2
  echo "  FIX: ./scripts/ruleset-check.sh --update, then commit the result." >&2
  exit 1
fi
# The snapshot is ALREADY canonical (it is written by --update), so it only
# needs key normalisation — it cannot be run back through canonicalize(), which
# expects a full ruleset document.
SNAP="$(jq -S . <"$SNAPSHOT")"

restore() {
  local tmp; tmp="$(mktemp)"
  # Keep the full live document, overwrite only the fields the snapshot governs.
  jq --argjson c "$SNAP" '
      .rules = (
        ( [ .rules[] | select(.type=="required_status_checks") ]
          | map(.parameters.required_status_checks = ($c.requiredStatusChecks | map({context: .}))
               | .parameters.strict_required_status_checks_policy = $c.strictRequiredStatusChecks)
        ) +
        ( [ .rules[] | select(.type=="pull_request") ]
          | map(.parameters.required_review_thread_resolution = $c.requiredReviewThreadResolution
               | .parameters.required_approving_review_count    = $c.requiredApprovingReviewCount
               | .parameters.allowed_merge_methods              = $c.allowedMergeMethods)
        ) +
        [ .rules[] | select(.type!="required_status_checks" and .type!="pull_request") ]
      )
  ' <<<"$LIVE_RAW" > "$tmp"
  gh api -X PUT "repos/$REPO/rulesets/$RULESET_ID" --input "$tmp" --jq '.id' >/dev/null
  rm -f "$tmp"
  echo "Live ruleset $RULESET_ID restored from the committed snapshot."
}

if [[ "$MODE" == "restore" ]]; then
  if [[ "$LIVE" == "$SNAP" ]]; then
    echo "Live ruleset already matches the snapshot; nothing to restore."
    exit 0
  fi
  echo "This will overwrite the LIVE merge contract with the committed snapshot."
  diff -u <(printf '%s\n' "$SNAP") <(printf '%s\n' "$LIVE") || true
  printf 'Type "restore" to continue: '
  read -r ans
  [[ "$ans" == "restore" ]] || { echo "Aborted."; exit 1; }
  restore
  exit 0
fi

if [[ "$LIVE" == "$SNAP" ]]; then
  echo "Ruleset $RULESET_ID matches .github/ruleset-main.json — merge contract intact."
  exit 0
fi

if [[ "$MODE" == "diff" ]]; then
  diff -u <(printf '%s\n' "$SNAP") <(printf '%s\n' "$LIVE") || true
  echo
  echo "(left = committed snapshot, right = live GitHub state)"
  exit 1
fi

printf '%s----------------------------------------%s\n' "$RED" "$NC"
printf '%s| RULESET DRIFT — the merge contract changed%s\n' "$RED" "$NC"
printf '%s----------------------------------------%s\n' "$RED" "$NC"
echo "Live GitHub state no longer matches .github/ruleset-main.json:"
echo
diff -u <(printf '%s\n' "$SNAP") <(printf '%s\n' "$LIVE") || true
echo
echo "  Intentional?  Then it needs an ADR, followed by:"
echo "      ./scripts/ruleset-check.sh --update && git add .github/ruleset-main.json"
echo "  Unintentional? The merge gate was weakened outside of review:"
echo "      ./scripts/ruleset-check.sh --restore"
echo "  See agents-docs/harness.md (L-008) and plans/ADRs/044-merge-automation-and-delivery-loop.md"
exit 1
