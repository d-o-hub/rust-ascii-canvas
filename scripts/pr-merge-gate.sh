#!/usr/bin/env bash
# PR merge guard-rail (computational feedback sensor).
#
# Answers one question: "may this pull request be merged right now, by a human
# or by auto-merge?" It is the *pre-merge* counterpart of
# scripts/quality-gates.sh (which answers "is the code itself healthy?") and
# the preflight sibling of scripts/release.sh (read-only, never merges).
#
# The contract, in one place:
#   1. the PR is not a draft
#   2. the PR is mergeable (no conflicts with the base branch)
#   3. every status check has concluded SUCCESS
#   4. every review thread is resolved  (harness L-008)
#   5. no outstanding CHANGES_REQUESTED review
#
# Harness: see agents-docs/harness.md and ADR-044.
# Runbook: agents-docs/delivery.md
#
# Usage:
#   ./scripts/pr-merge-gate.sh            # PR for the current branch
#   ./scripts/pr-merge-gate.sh 212        # a specific PR
#   ./scripts/pr-merge-gate.sh --json     # machine-readable (for skills/CI)
#   ./scripts/pr-merge-gate.sh --self-test  # offline fixtures, no network
#   ./scripts/pr-merge-gate.sh --help
#
# Exit 0 = mergeable now. Exit 1 = blocked (FIX: hints explain how).
# This script NEVER merges. Arming auto-merge is a separate, explicit act:
#   gh pr merge <PR> --auto --squash
#
# Why --self-test: a guard-rail nobody has ever seen fire is not a guard-rail.
# The repo has no historical PR with review threads, so the blocking branches
# are proven against fixtures instead of a live mutation. See agents-docs/harness.md (L-008).

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

RED=$'\033[0;31m'; GREEN=$'\033[0;32m'; YELLOW=$'\033[1;33m'; NC=$'\033[0m'
[[ -t 1 ]] || { RED=''; GREEN=''; YELLOW=''; NC=''; }

PR_ARG=""
JSON_OUT=false
SELF_TEST=false
for arg in "$@"; do
  case $arg in
    --json) JSON_OUT=true ;;
    --self-test) SELF_TEST=true ;;
    -h|--help) sed -n '2,34p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    [0-9]*) PR_ARG="$arg" ;;
    *) echo "Unknown argument: $arg (try a PR number, --json, --self-test, --help)" >&2; exit 1 ;;
  esac
done

fail() { printf '%s✗ %s%s\n' "$RED" "$1" "$NC"; BLOCKED=$((BLOCKED + 1)); }
ok()   { printf '%s✓ %s%s\n' "$GREEN" "$1" "$NC"; }
warn() { printf '%s! %s%s\n' "$YELLOW" "$1" "$NC"; }

BLOCKED=0

# --- Pure predicates --------------------------------------------------------
# Single source of truth for the merge contract. Each takes fixture-shaped
# JSON on stdin and prints a number or a list, so the same logic backs both
# the live verdict and --self-test.

# Checks that concluded and are NOT green (PENDING is handled separately).
checks_not_green() {
  jq '[.statusCheckRollup[] | ((.conclusion // .state)) as $s
    | select($s != "SUCCESS" and $s != "NEUTRAL" and $s != "SKIPPED"
             and $s != "PENDING" and $s != "IN_PROGRESS" and $s != "QUEUED" and $s != "")
    ] | length'
}
# Checks still running — not yet a pass, so they block.
checks_pending() {
  jq '[.statusCheckRollup[] | ((.conclusion // .state)) as $s
    | select($s == "PENDING" or $s == "IN_PROGRESS" or $s == "QUEUED" or $s == "")
    ] | length'
}
checks_total() { jq '.statusCheckRollup | length'; }
checks_failed_names() {
  jq -r '[.statusCheckRollup[]
    | select(((.conclusion // .state)) as $s
        | ($s != "SUCCESS" and $s != "NEUTRAL" and $s != "SKIPPED"
           and $s != "PENDING" and $s != "IN_PROGRESS" and $s != "QUEUED" and $s != ""))
    | (.name // .context // "?")] | unique | .[]'
}
threads_total()  { jq '.data.repository.pullRequest.reviewThreads.nodes | length'; }
threads_open()   { jq '[.data.repository.pullRequest.reviewThreads.nodes[] | select(.isResolved == false)] | length'; }
threads_open_list() {
  jq -r '[.data.repository.pullRequest.reviewThreads.nodes[] | select(.isResolved == false)
    | "\(.comments.nodes[0].author.login // "?"):\(.comments.nodes[0].path // "?")"] | .[]'
}
changes_requested() { jq '[.reviews[] | select(.state == "CHANGES_REQUESTED")] | length'; }

# --- Offline self-test ------------------------------------------------------
# Proves the merge contract on fixtures. No network, no gh, no repo mutation.
# Run this after editing the predicates above, and in CI (see the
# "PR Readiness (merge gate)" job).
if $SELF_TEST; then
  if ! command -v jq >/dev/null 2>&1; then
    echo "ERROR: jq is required for --self-test." >&2; exit 1
  fi
  echo "pr-merge-gate self-test (fixtures)..."
  echo

  FAILED_FIXTURES=0
  expect() { # expect <label> <expected> <actual>
    if [[ "$2" == "$3" ]]; then
      printf '%s✓%s %-46s = %s\n' "$GREEN" "$NC" "$1" "$3"
    else
      printf '%s✗%s %-46s expected %s, got %s\n' "$RED" "$NC" "$1" "$2" "$3"
      FAILED_FIXTURES=$((FAILED_FIXTURES + 1))
    fi
  }

  # All green, no threads.
  G='{"isDraft":false,"mergeable":"MERGEABLE","statusCheckRollup":[{"name":"CI Success","conclusion":"SUCCESS"}],"reviews":[]}'
  # One red check.
  R='{"isDraft":false,"mergeable":"MERGEABLE","statusCheckRollup":[{"name":"Clippy","conclusion":"FAILURE"},{"name":"CI Success","conclusion":"SUCCESS"}],"reviews":[]}'
  # One check still running.
  P='{"isDraft":false,"mergeable":"MERGEABLE","statusCheckRollup":[{"name":"E2E Tests","state":"IN_PROGRESS"}],"reviews":[]}'
  # Path filter legitimately skipped a job.
  S='{"isDraft":false,"mergeable":"MERGEABLE","statusCheckRollup":[{"name":"Clippy","conclusion":"SUCCESS"},{"name":"Web","conclusion":"SKIPPED"}],"reviews":[]}'
  # No CI ran at all — the L-002/L-003 trap.
  Z='{"isDraft":false,"mergeable":"MERGEABLE","statusCheckRollup":[],"reviews":[]}'
  # Changes requested.
  C='{"isDraft":false,"mergeable":"MERGEABLE","statusCheckRollup":[{"name":"CI Success","conclusion":"SUCCESS"}],"reviews":[{"state":"CHANGES_REQUESTED"}]}'

  # GraphQL thread fixtures.
  GT='{"data":{"repository":{"pullRequest":{"reviewThreads":{"nodes":[]}}}}}'
  GO='{"data":{"repository":{"pullRequest":{"reviewThreads":{"nodes":[{"isResolved":false,"comments":{"nodes":[{"author":{"login":"reviewer"},"path":"src/core/history.rs"}]}}]}}}}}'
  GR='{"data":{"repository":{"pullRequest":{"reviewThreads":{"nodes":[{"isResolved":true,"comments":{"nodes":[{"author":{"login":"reviewer"},"path":"web/ui.ts"}]}}]}}}}}'

  echo "— status checks —"
  expect "green: not-green count"        0 "$(checks_not_green <<<"$G")"
  expect "green: pending count"          0 "$(checks_pending <<<"$G")"
  expect "green: total"                  1 "$(checks_total <<<"$G")"
  expect "red: not-green count"          1 "$(checks_not_green <<<"$R")"
  expect "red: names the offender" "Clippy" "$(checks_failed_names <<<"$R")"
  expect "pending: pending count"        1 "$(checks_pending <<<"$P")"
  expect "pending: not NOT a failure"    0 "$(checks_not_green <<<"$P")"
  expect "skipped job still satisfies"   0 "$(checks_not_green <<<"$S")"
  expect "zero-check PR: total"          0 "$(checks_total <<<"$Z")"
  echo
  echo "— review threads —"
  expect "no threads: total"             0 "$(threads_total <<<"$GT")"
  expect "no threads: open"              0 "$(threads_open <<<"$GT")"
  expect "open thread: total"            1 "$(threads_total <<<"$GO")"
  expect "open thread: open"             1 "$(threads_open <<<"$GO")"
  expect "open thread: names it" "reviewer:src/core/history.rs" "$(threads_open_list <<<"$GO")"
  expect "resolved thread: total"        1 "$(threads_total <<<"$GR")"
  expect "resolved thread: open"         0 "$(threads_open <<<"$GR")"
  echo
  echo "— reviews —"
  expect "no reviews"                    0 "$(changes_requested <<<"$G")"
  expect "changes requested blocks"      1 "$(changes_requested <<<"$C")"
  echo

  if [[ "$FAILED_FIXTURES" -gt 0 ]]; then
    printf '%sself-test FAILED (%d assertion(s))%s\n' "$RED" "$FAILED_FIXTURES" "$NC"
    exit 1
  fi
  printf '%sself-test PASSED — merge contract holds on all fixtures%s\n' "$GREEN" "$NC"
  exit 0
fi

# --- 0. Preconditions -------------------------------------------------------
if ! command -v gh >/dev/null 2>&1 || ! gh auth status >/dev/null 2>&1; then
  echo "ERROR: gh CLI unavailable or unauthenticated — merge state CANNOT be verified." >&2
  echo "  FIX: install gh and run 'gh auth login'. Never assume a PR is mergeable without gh." >&2
  exit 1
fi
if ! command -v jq >/dev/null 2>&1; then
  echo "ERROR: jq is required by this sensor." >&2
  echo "  FIX: install jq, or read state manually: gh pr view <PR> --json ..." >&2
  exit 1
fi

REPO="$(gh repo view --json nameWithOwner --jq .nameWithOwner)"
OWNER="${REPO%%/*}"
REPO_NAME="${REPO##*/}"

# Resolve the PR: explicit number, else the branch's PR.
PR="$PR_ARG"
if [[ -z "$PR" ]]; then
  BRANCH="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo '')"
  if [[ -z "$BRANCH" || "$BRANCH" == "HEAD" ]]; then
    echo "ERROR: detached HEAD and no PR number given." >&2
    echo "  FIX: pass the PR number, e.g. ./scripts/pr-merge-gate.sh 212" >&2
    exit 1
  fi
  PR="$(gh pr view --json number --jq .number 2>/dev/null || true)"
  if [[ -z "$PR" ]]; then
    echo "ERROR: no open PR found for branch '$BRANCH'." >&2
    echo "  FIX: push the branch and open a PR first, or pass a PR number." >&2
    exit 1
  fi
fi



# --- Fetch state ------------------------------------------------------------
PR_JSON="$(gh pr view "$PR" --json number,title,url,isDraft,mergeable,reviewDecision,statusCheckRollup,reviews 2>/dev/null)" || {
  echo "ERROR: could not read PR #$PR (does it exist?)." >&2
  exit 1
}

GRAPHQL="$(gh api graphql -f query='
query($owner:String!,$repo:String!,$number:Int!){
  repository(owner:$owner,name:$repo){
    pullRequest(number:$number){
      reviewThreads(first:100){
        nodes{ isResolved isOutdated comments(first:1){ nodes{ author{login} path } } }
      }
    }
  }
}' -F "owner=$OWNER" -F "repo=$REPO_NAME" -F "number=$PR" 2>/dev/null)" || {
  echo "ERROR: could not read review threads for PR #$PR (GraphQL)." >&2
  echo "  FIX: verify token scopes (needs 'read: pull requests')." >&2
  exit 1
}

IS_DRAFT="$(jq -r '.isDraft' <<<"$PR_JSON")"
MERGEABLE="$(jq -r '.mergeable' <<<"$PR_JSON")"
TITLE="$(jq -r '.title' <<<"$PR_JSON")"
URL="$(jq -r '.url' <<<"$PR_JSON")"

# --- 1. Not a draft ---------------------------------------------------------
if [[ "$IS_DRAFT" == "true" ]]; then
  fail "PR #$PR is still a draft"
  echo "  FIX: mark it ready for review: gh pr ready $PR"
else
  ok "Not a draft"
fi

# --- 2. Mergeable (no conflicts) --------------------------------------------
if [[ "$MERGEABLE" == "MERGEABLE" ]]; then
  ok "Mergeable (no conflicts with base)"
elif [[ "$MERGEABLE" == "CONFLICTING" ]]; then
  fail "PR #$PR has merge conflicts"
  echo "  FIX: rebase onto the base branch: git rebase origin/main && git push --force-with-lease"
else
  # UNKNOWN means GitHub is still computing — not a verdict, so do not block.
  warn "Mergeability is UNKNOWN (GitHub still computing) — re-run in a few seconds"
fi



# --- 3. Every status check SUCCESS ------------------------------------------
# A queued/in-progress check is NOT a pass. SKIPPED counts as satisfied:
# path filters legitimately skip jobs for docs-only changes (see ci.yml).
CHECK_TOTAL="$(checks_total <<<"$PR_JSON")"
CHECK_BAD="$(checks_not_green <<<"$PR_JSON")"
CHECK_PENDING="$(checks_pending <<<"$PR_JSON")"
CHECK_FAILED_NAMES="$(checks_failed_names <<<"$PR_JSON" 2>/dev/null || true)"

if [[ "$CHECK_TOTAL" -eq 0 ]]; then
  fail "PR #$PR reports ZERO status checks"
  echo "  FIX: no CI ran. Check the 'Detect Changes' path filters in .github/workflows/ci.yml."
  echo "       Harness L-002/L-003: a green PR that ran no sensors is NOT verified."
elif [[ "$CHECK_BAD" -gt 0 ]]; then
  fail "$CHECK_BAD of $CHECK_TOTAL status checks are not successful:"
  while IFS= read -r c; do [[ -n "$c" ]] && echo "    - $c"; done <<<"$CHECK_FAILED_NAMES"
  echo "  FIX: read the failure with 'gh run list --branch <branch>', fix the ROOT CAUSE,"
  echo "       then re-run locally: npm run gate:fast  (gate:full before push)."
  echo "       Do NOT weaken a test, add #[allow], or skip a job to get this green."
elif [[ "$CHECK_PENDING" -gt 0 ]]; then
  fail "$CHECK_PENDING of $CHECK_TOTAL status checks are still running"
  echo "  FIX: wait for CI, then re-run: ./scripts/pr-merge-gate.sh $PR"
else
  ok "All $CHECK_TOTAL status checks successful"
fi

# --- 4. Every review thread resolved (harness L-008) ------------------------
THREADS_TOTAL="$(threads_total <<<"$GRAPHQL")"
THREADS_OPEN="$(threads_open <<<"$GRAPHQL")"
THREADS_OPEN_LIST="$(threads_open_list <<<"$GRAPHQL" 2>/dev/null || true)"

if [[ "$THREADS_TOTAL" -eq 0 ]]; then
  ok "No review threads to resolve"
elif [[ "$THREADS_OPEN" -gt 0 ]]; then
  fail "$THREADS_OPEN of $THREADS_TOTAL review threads unresolved:"
  while IFS= read -r t; do [[ -n "$t" ]] && echo "    - $t"; done <<<"$THREADS_OPEN_LIST"
  echo "  FIX: address each thread (reply, fix the code, then 'Resolve thread')."
  echo "       Unresolved threads mean reviewer feedback was dropped on the floor."
else
  ok "All $THREADS_TOTAL review threads resolved"
fi

# --- 5. No outstanding CHANGES_REQUESTED -------------------------------------
CHANGES_REQ="$(changes_requested <<<"$PR_JSON")"
if [[ "$CHANGES_REQ" -gt 0 ]]; then
  fail "$CHANGES_REQ CHANGES_REQUESTED review(s) on record"
  echo "  FIX: resolve the requested changes and request re-review. A new APPROVED review"
  echo "       does not erase an old CHANGES_REQUESTED — push a fix commit instead."
else
  ok "No outstanding CHANGES_REQUESTED"
fi

# --- Verdict ----------------------------------------------------------------
if [[ "$JSON_OUT" == "true" ]]; then
  jq -n --argjson pr "$PR" --arg url "$URL" --arg title "$TITLE" \
        --argjson blocked "$BLOCKED" \
        --argjson draft "$IS_DRAFT" --arg mergeable "$MERGEABLE" \
        --argjson checks "$CHECK_TOTAL" --argjson checksBad "$CHECK_BAD" \
        --argjson threadsTotal "$THREADS_TOTAL" --argjson threadsOpen "$THREADS_OPEN" \
        --argjson changesRequested "$CHANGES_REQ" \
        '{pr:$pr,url:$url,title:$title,mergeable:($blocked==0),
          details:{draft:$draft,mergeability:$mergeable,checks:$checks,checksNotGreen:$checksBad,
          reviewThreads:$threadsTotal,unresolvedThreads:$threadsOpen,changesRequested:$changesRequested}}'
  [[ "$BLOCKED" -eq 0 ]] || exit 1
  exit 0
fi

echo
echo "PR #$PR — $TITLE"
echo "$URL"
echo
if [[ "$BLOCKED" -gt 0 ]]; then
  printf '%s----------------------------------------%s\n' "$RED" "$NC"
  printf '%s| MERGE BLOCKED (%d unmet condition(s))%s\n' "$RED" "$BLOCKED" "$NC"
  printf '%s----------------------------------------%s\n' "$RED" "$NC"
  echo "Do not merge. Fix the conditions above, then re-run this script."
  echo "Harness: agents-docs/harness.md · Runbook: agents-docs/delivery.md"
  exit 1
fi

printf '%s----------------------------------------%s\n' "$GREEN" "$NC"
printf '%s| MERGEABLE — all guard-rails satisfied%s\n' "$GREEN" "$NC"
printf '%s----------------------------------------%s\n' "$GREEN" "$NC"
echo "Next (arm auto-merge so CI gates the merge, not you):"
echo "  gh pr merge $PR --auto --squash"
echo
echo "Then verify:  gh pr view $PR --json autoMergeRequest"

