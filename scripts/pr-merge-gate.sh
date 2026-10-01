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

# Resolve this script's own path BEFORE cd, so --help still works when invoked
# from another directory (BASH_SOURCE is absolute; $0 is not).
SCRIPT_PATH="${BASH_SOURCE[0]}"
ROOT="$(cd "$(dirname "$SCRIPT_PATH")/.." && pwd)"

RED=$'\033[0;31m'; GREEN=$'\033[0;32m'; YELLOW=$'\033[1;33m'; NC=$'\033[0m'
[[ -t 1 ]] || { RED=''; GREEN=''; YELLOW=''; NC=''; }

PR_ARG=""
JSON_OUT=false
SELF_TEST=false
for arg in "$@"; do
  case $arg in
    --json) JSON_OUT=true ;;
    --self-test) SELF_TEST=true ;;
    -h|--help) sed -n '2,34p' "$SCRIPT_PATH" | sed 's/^# \{0,1\}//'; exit 0 ;;
    [0-9]*) PR_ARG="$arg" ;;
    *) echo "Unknown argument: $arg (try a PR number, --json, --self-test, --help)" >&2; exit 1 ;;
  esac
done

cd "$ROOT"

# In --json mode stdout is reserved for the JSON document, so every human-facing
# line goes to stderr. Otherwise `script --json | jq .` would fail to parse.
if $JSON_OUT; then
  fail() { printf '%s✗ %s%s\n' "$RED" "$1" "$NC" >&2; BLOCKED=$((BLOCKED + 1)); }
  ok()   { printf '%s✓ %s%s\n' "$GREEN" "$1" "$NC" >&2; }
  warn() { printf '%s! %s%s\n' "$YELLOW" "$1" "$NC" >&2; }
else
  fail() { printf '%s✗ %s%s\n' "$RED" "$1" "$NC"; BLOCKED=$((BLOCKED + 1)); }
  ok()   { printf '%s✓ %s%s\n' "$GREEN" "$1" "$NC"; }
  warn() { printf '%s! %s%s\n' "$YELLOW" "$1" "$NC"; }
fi

# The invariant above is stated once but honoured only by ok/fail/warn — every
# bare `echo "  FIX: …"`, indented commit line and NOTE below is human-facing
# too, and on a failing run they reached stdout, so `--json | jq .` (the pipe
# the merge-gate skill documents as safe) died on "Invalid numeric literal"
# and CI fell back to the raw log. Annotating ~40 call sites guarantees the
# next one is forgotten; shadow `echo` instead so the contract holds by
# construction. Explicit `echo … >&2` error paths are unaffected, and the mode
# is read at call time so --self-test can exercise both directions.
echo() {
  if $JSON_OUT; then builtin echo "$@" >&2; else builtin echo "$@"; fi
}

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
# Checks still running — not yet a pass, so they block. 'EXPECTED' is a legacy
# "allowed to fail, awaiting" state; treat it as pending, never as green.
checks_pending() {
  jq '[.statusCheckRollup[] | ((.conclusion // .state)) as $s
    | select($s == "PENDING" or $s == "IN_PROGRESS" or $s == "QUEUED"
             or $s == "EXPECTED" or $s == "WAITING" or $s == "REQUESTED" or $s == "")
    ] | length'
}
checks_total() { jq '.statusCheckRollup | length'; }
checks_failed_names() {
  jq -r '[.statusCheckRollup[]
    | select(((.conclusion // .state)) as $s
        | ($s != "SUCCESS" and $s != "NEUTRAL" and $s != "SKIPPED"
           and $s != "PENDING" and $s != "IN_PROGRESS" and $s != "QUEUED"
           and $s != "EXPECTED" and $s != "WAITING" and $s != "REQUESTED" and $s != ""))
    | (.name // .context // "?")] | unique | .[]'
}
# GraphQL thread accessors. `has("data")` guards against a null pullRequest,
# which would otherwise make `length` of null read as "0 threads" — i.e. a
# failure to query would look exactly like a clean PR.
threads_data_ok() { jq -e '.data.repository.pullRequest != null' >/dev/null; }
threads_total()  { jq '.data.repository.pullRequest.reviewThreads.nodes | length'; }
threads_open()   { jq '[.data.repository.pullRequest.reviewThreads.nodes[] | select(.isResolved == false)] | length'; }
threads_open_list() {
  jq -r '[.data.repository.pullRequest.reviewThreads.nodes[] | select(.isResolved == false)
    | "\(.comments.nodes[0].author.login // "?"):\(.comments.nodes[0].path // "?")"] | .[]'
}
# reviewDecision is GitHub's authoritative roll-up: it already accounts for
# dismissal and staleness. Counting every CHANGES_REQUESTED ever submitted
# would block forever on a review the author has since addressed.
review_decision() { jq -r '.reviewDecision // ""'; }
# Informational only — kept for the report, not used to block.
changes_requested() { jq '[.reviews[] | select(.state == "CHANGES_REQUESTED")] | length'; }
# The PR's own commits, as GitHub reports them. Never the local checkout:
# harness L-021 — `git log origin/$BASE..HEAD` printed whatever branch this
# clone happened to be on, so gating PR N from an unrelated branch claimed PR N
# would ship that branch's work. A confident answer about the wrong object is
# worse than no answer: it reads as verified. Fail closed when the field is
# absent or null rather than report "0 commits".
commits_data_ok() { jq -e '.commits | type == "array"' >/dev/null; }
commits_total()   { jq '.commits | length'; }
commits_list()    { jq -r '.commits[] | "\(.oid[0:7]) \(.messageHeadline)"'; }

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
  # A check cancelled mid-run must never read as green.
  X='{"isDraft":false,"mergeable":"MERGEABLE","statusCheckRollup":[{"name":"E2E Tests","conclusion":"CANCELLED"}],"reviews":[]}'
  # Legacy 'expected to fail' state: pending, not a hard failure.
  E='{"isDraft":false,"mergeable":"MERGEABLE","statusCheckRollup":[{"name":"Legacy","state":"EXPECTED"}],"reviews":[]}'
  # conclusion:null with a state field — the common real-world shape.
  N='{"isDraft":false,"mergeable":"MERGEABLE","statusCheckRollup":[{"name":"CI Success","conclusion":null,"state":"PENDING"}],"reviews":[]}'
  # Review roll-up states.
  C='{"isDraft":false,"mergeable":"MERGEABLE","statusCheckRollup":[{"name":"CI Success","conclusion":"SUCCESS"}],"reviews":[{"state":"CHANGES_REQUESTED"}]}'
  CA='{"isDraft":false,"mergeable":"MERGEABLE","statusCheckRollup":[{"name":"CI Success","conclusion":"SUCCESS"}],"reviews":[{"state":"CHANGES_REQUESTED"}],"reviewDecision":"APPROVED"}'
  CR='{"isDraft":false,"mergeable":"MERGEABLE","statusCheckRollup":[{"name":"CI Success","conclusion":"SUCCESS"}],"reviews":[],"reviewDecision":"CHANGES_REQUESTED"}'

  # GraphQL thread fixtures.
  GT='{"data":{"repository":{"pullRequest":{"reviewThreads":{"nodes":[]}}}}}'
  GO='{"data":{"repository":{"pullRequest":{"reviewThreads":{"nodes":[{"isResolved":false,"comments":{"nodes":[{"author":{"login":"reviewer"},"path":"src/core/history.rs"}]}}]}}}}}'
  GR='{"data":{"repository":{"pullRequest":{"reviewThreads":{"nodes":[{"isResolved":true,"comments":{"nodes":[{"author":{"login":"reviewer"},"path":"web/ui.ts"}]}}]}}}}}'
  # Query failed / PR not visible — must NOT read as "0 threads, all clear".
  GN='{"data":{"repository":{"pullRequest":null}}}'

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
  expect "cancelled blocks (not green)"  1 "$(checks_not_green <<<"$X")"
  expect "cancelled is not 'pending'"    0 "$(checks_pending <<<"$X")"
  expect "EXPECTED treated as pending"   1 "$(checks_pending <<<"$E")"
  expect "conclusion:null+state pending" 1 "$(checks_pending <<<"$N")"
  expect "conclusion:null not a failure" 0 "$(checks_not_green <<<"$N")"
  echo
  echo "— review threads —"
  expect "no threads: total"             0 "$(threads_total <<<"$GT")"
  expect "no threads: open"              0 "$(threads_open <<<"$GT")"
  expect "open thread: total"            1 "$(threads_total <<<"$GO")"
  expect "open thread: open"             1 "$(threads_open <<<"$GO")"
  expect "open thread: names it" "reviewer:src/core/history.rs" "$(threads_open_list <<<"$GO")"
  expect "resolved thread: total"        1 "$(threads_total <<<"$GR")"
  expect "resolved thread: open"         0 "$(threads_open <<<"$GR")"
  if threads_data_ok <<<"$GT"; then r=true; else r=false; fi
  expect "valid payload accepted"      "$r" true
  if threads_data_ok <<<"$GN"; then r=true; else r=false; fi
  expect "null pullRequest REJECTED"   "$r" false
  echo
  echo "— reviews —"
  expect "no reviews"                    0 "$(changes_requested <<<"$G")"
  expect "historical CHANGES_REQUESTED"  1 "$(changes_requested <<<"$C")"
  expect "roll-up: none set"             "" "$(review_decision <<<"$C")"
  expect "roll-up: APPROVED wins"   "APPROVED" "$(review_decision <<<"$CA")"
  expect "roll-up: CHANGES_REQUESTED" "CHANGES_REQUESTED" "$(review_decision <<<"$CR")"
  echo
  echo "— commit scope (L-021) —"
  CS1='{"commits":[{"oid":"9dfc490fdf23663d721943e31511520bbed1195c","messageHeadline":"feat(drafts): add bounded local draft recovery"}]}'
  CS5='{"commits":[
    {"oid":"ab9c6160b7d2cd3421895801ad819f71036e1dfc","messageHeadline":"feat(drafts): add bounded local draft recovery"},
    {"oid":"480bb84e16e73a0b6156a19b8755f3b36863b2cd","messageHeadline":"fix(harness): clear pre-existing quality gate warnings"},
    {"oid":"ec2e99b9400aac6464a9fdb7eb5ab1c20408fc34","messageHeadline":"fix(harness): make merge-gate shellcheck-clean"},
    {"oid":"823cf9c0fbff3334aea7136a314b69a7884c4243","messageHeadline":"fix(tests): bind storage method in draft fixtures"},
    {"oid":"9dfc490fdf23663d721943e31511520bbed1195c","messageHeadline":"fix(tests): bind migration storage fixture method"}]}'
  CSEMPTY='{"commits":[]}'
  CSNULL='{"commits":null}'
  CSMISSING='{}'
  expect "one commit: total"         1 "$(commits_total <<<"$CS1")"
  expect "one commit: short+subject" "9dfc490 feat(drafts): add bounded local draft recovery" "$(commits_list <<<"$CS1")"
  expect "five commits: total"       5 "$(commits_total <<<"$CS5")"
  expect "five commits: lines"       5 "$(grep -c . <<<"$(commits_list <<<"$CS5")")"
  expect "empty array is countable"  0 "$(commits_total <<<"$CSEMPTY")"
  if commits_data_ok <<<"$CS1"; then r=true; else r=false; fi
  expect "commit array accepted"     "$r" true
  if commits_data_ok <<<"$CSEMPTY"; then r=true; else r=false; fi
  expect "empty array accepted"      "$r" true
  if commits_data_ok <<<"$CSNULL"; then r=true; else r=false; fi
  expect "commits:null REJECTED"     "$r" false
  if commits_data_ok <<<"$CSMISSING"; then r=true; else r=false; fi
  expect "commits absent REJECTED"   "$r" false
  echo
  echo "— --json stdout contract —"
  # The merge-gate skill documents `gate:pr -- --json | jq .` as safe, so the
  # verdict document must be the ONLY thing on stdout, whatever the report
  # prints. Probe the shadowed echo in both directions (positive + negative).
  PREV_JSON_OUT="$JSON_OUT"
  echo_probe() { echo probe; }   # exercises the shadowed echo, not a substitution
  JSON_OUT=true
  json_stdout="$(echo_probe 2>/dev/null)"
  json_stderr="$(echo_probe 2>&1 >/dev/null)"
  JSON_OUT="$PREV_JSON_OUT"
  expect "json mode: stdout stays clean" "" "$json_stdout"
  expect "json mode: report reaches stderr" "probe" "$json_stderr"
  JSON_OUT=false
  json_stdout_plain="$(echo_probe 2>/dev/null)"
  JSON_OUT="$PREV_JSON_OUT"
  expect "plain mode: stdout still carries text" "probe" "$json_stdout_plain"
  unset -f echo_probe
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

if ! REPO="$(gh repo view --json nameWithOwner --jq .nameWithOwner 2>/dev/null)"; then
  echo "ERROR: could not determine the repository (gh repo view failed)." >&2
  echo "  FIX: check network/auth, or run from inside a git checkout of the repo." >&2
  exit 1
fi
OWNER="${REPO%%/*}"
REPO_NAME="${REPO##*/}"

# Resolve the PR: explicit number, else the branch's PR.
PR="$PR_ARG"
if [[ -z "$PR" ]]; then
  BRANCH="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || true)"
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
PR_JSON="$(gh pr view "$PR" --json number,title,url,isDraft,mergeable,reviewDecision,statusCheckRollup,reviews,commits,headRefName,headRefOid,baseRefName 2>/dev/null)" || {
  echo "ERROR: could not read PR #$PR (does it exist?)." >&2
  exit 1
}

GRAPHQL_QUERY="$(cat <<'GRAPHQL'
query($owner:String!,$repo:String!,$number:Int!){
  repository(owner:$owner,name:$repo){
    pullRequest(number:$number){
      reviewThreads(first:100){
        nodes{ isResolved isOutdated comments(first:1){ nodes{ author{login} path } } }
      }
    }
  }
}
GRAPHQL
)"
GRAPHQL="$(gh api graphql -f query="$GRAPHQL_QUERY" -F "owner=$OWNER" -F "repo=$REPO_NAME" -F "number=$PR" 2>/dev/null)" || {
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
# Fail closed. `mergeable` is UNKNOWN while GitHub is still computing (routine
# right after a push), and it is also UNKNOWN if the query was truncated. An
# unverified conflict state must never read as "mergeable" — see AGENTS.md:
# do not treat "could not verify" as "verified".
if [[ "$MERGEABLE" == "MERGEABLE" ]]; then
  ok "Mergeable (no conflicts with base)"
elif [[ "$MERGEABLE" == "CONFLICTING" ]]; then
  fail "PR #$PR has merge conflicts"
  echo "  FIX: rebase onto the base branch: git rebase origin/main && git push --force-with-lease"
else
  fail "Mergeability is $MERGEABLE (GitHub has not computed it yet)"
  echo "  FIX: wait ~10s and re-run: ./scripts/pr-merge-gate.sh $PR"
  echo "       UNKNOWN is not a pass. The auto-merge path will re-check anyway,"
  echo "       but the local gate refuses to guess about conflicts."
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
THREADS_PAGE_SIZE=100
if ! threads_data_ok <<<"$GRAPHQL"; then
  fail "Could not read review threads for PR #$PR (GraphQL returned no pullRequest)"
  echo "  FIX: the PR may not exist, or the token lacks 'read: pull requests'."
  echo "       Treating an unanswerable query as 'no threads' would let a red PR through."
else
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

  # A full page means the result set may be truncated; unreturned threads would
  # be invisible to this sensor (the ruleset still enforces them server-side).
  if [[ "$THREADS_TOTAL" -ge "$THREADS_PAGE_SIZE" ]]; then
    warn "PR #$PR has >= $THREADS_PAGE_SIZE review threads — this query is capped at $THREADS_PAGE_SIZE."
    echo "  Note: additional threads may exist beyond the first page. The ruleset"
    echo "        still enforces thread resolution, so this cannot bypass the merge gate."
  fi
fi

# --- 5. Review decision ------------------------------------------------------
# Use GitHub's roll-up, not a raw count: a dismissed or stale CHANGES_REQUESTED
# no longer counts against the PR, and blocking on one would deadlock the merge.
REVIEW_DECISION="$(review_decision <<<"$PR_JSON")"
CHANGES_REQ="$(changes_requested <<<"$PR_JSON")"
if [[ "$REVIEW_DECISION" == "CHANGES_REQUESTED" || "$REVIEW_DECISION" == "REVIEW_REQUIRED" ]]; then
  fail "Review decision is '$REVIEW_DECISION' (changes are still requested)"
  echo "  FIX: push a commit that addresses the review, then request re-review."
  echo "       Only the latest state counts — a dismissed review no longer blocks."
elif [[ "$CHANGES_REQ" -gt 0 ]]; then
  ok "Review decision: ${REVIEW_DECISION:-none required} ($CHANGES_REQ historical CHANGES_REQUESTED, none current)"
else
  ok "Review decision: ${REVIEW_DECISION:-none required}, no CHANGES_REQUESTED"
fi

# --- 6. Scope: which commits would this PR actually merge? ------------------
# Harness L-011: a PR was opened from a branch that already contained an
# unmerged feature commit, so the feature merged under an unrelated title.
# Per-commit "no product code staged" checks cannot see that — the foreign
# commit was never staged by this session. Surfacing the commit list here makes
# it visible at the point of decision, where a human or agent can catch it.
#
# Harness L-021: read the commits from GitHub, never from this checkout. The
# old `git log origin/$BASE_BRANCH..HEAD` described whatever branch the local
# clone happened to be on, so running the gate for PR N while sitting on an
# unrelated branch printed that branch's commits as PR N's scope — and its
# "NOTE: every commit above ships with this PR" was simply false. The PR's own
# commit list is the only list that answers the question.
BASE_BRANCH="$(jq -r '.baseRefName // "main"' <<<"$PR_JSON")"
if ! commits_data_ok <<<"$PR_JSON"; then
  fail "Could not read the commit list for PR #$PR"
  echo "  FIX: re-run ./scripts/pr-merge-gate.sh $PR. If it persists, check gh"
  echo "       scopes ('read: pull requests'). An unverified scope is not a pass."
else
  COMMIT_COUNT="$(commits_total <<<"$PR_JSON")"
  if [[ "$COMMIT_COUNT" -eq 0 ]]; then
    fail "PR #$PR reports 0 commits — nothing to merge, or the query was truncated"
    echo "  FIX: open the PR in a browser and confirm it actually has commits."
  else
    ok "Commit scope: $COMMIT_COUNT commit(s) would merge into $BASE_BRANCH"
    commits_list <<<"$PR_JSON" | while IFS= read -r c; do echo "    $c"; done
    echo "  NOTE: every commit above ships with this PR. If any is not part of"
    echo "        the change you intend to make, your branch is wrong — rebase it"
    echo "        onto origin/$BASE_BRANCH. (harness L-011)"
  fi
fi

# Context, not evidence: where this checkout happens to be. It never feeds the
# commit list above (L-021); it only tells you whether your working tree is the
# thing CI just tested.
PR_HEAD_NAME="$(jq -r '.headRefName // ""' <<<"$PR_JSON")"
PR_HEAD_OID="$(jq -r '.headRefOid // ""' <<<"$PR_JSON")"
LOCAL_BRANCH="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || true)"
LOCAL_OID="$(git rev-parse HEAD 2>/dev/null || true)"
if [[ -z "$PR_HEAD_NAME" || -z "$PR_HEAD_OID" ]]; then
  warn "PR head is unknown — cannot compare it with this checkout"
elif [[ -z "$LOCAL_BRANCH" || -z "$LOCAL_OID" ]]; then
  warn "git unavailable here — cannot say where this checkout is; scope above is the PR's"
elif [[ "$LOCAL_BRANCH" == "$PR_HEAD_NAME" && "$LOCAL_OID" == "$PR_HEAD_OID" ]]; then
  ok "Local checkout matches the PR head ($PR_HEAD_NAME @ ${PR_HEAD_OID:0:7})"
elif [[ "$LOCAL_BRANCH" == "$PR_HEAD_NAME" ]]; then
  warn "Local '$PR_HEAD_NAME' is at ${LOCAL_OID:0:7}, PR head is ${PR_HEAD_OID:0:7} — your tree is stale"
  echo "  FIX: git fetch origin && git rebase origin/$BASE_BRANCH"
  echo "       CI tested ${PR_HEAD_OID:0:7}; do not claim local verification of it."
else
  ok "Local checkout '$LOCAL_BRANCH' is not the PR head — scope above is the PR's (L-021)"
fi

# --- Verdict ----------------------------------------------------------------
if [[ "$JSON_OUT" == "true" ]]; then
  jq -n --argjson pr "$PR" --arg url "$URL" --arg title "$TITLE" \
        --argjson blocked "$BLOCKED" \
        --argjson draft "$IS_DRAFT" --arg mergeable "$MERGEABLE" \
        --argjson checks "$CHECK_TOTAL" --argjson checksBad "$CHECK_BAD" \
        --argjson checksPending "$CHECK_PENDING" \
        --argjson threadsTotal "${THREADS_TOTAL:-null}" \
        --argjson threadsOpen "${THREADS_OPEN:-null}" \
        --arg reviewDecision "$REVIEW_DECISION" \
        --argjson changesRequested "$CHANGES_REQ" \
        '{pr:$pr,url:$url,title:$title,mergeable:($blocked==0),
          details:{draft:$draft,mergeability:$mergeable,
          checks:$checks,checksNotGreen:$checksBad,checksPending:$checksPending,
          reviewThreads:$threadsTotal,unresolvedThreads:$threadsOpen,
          reviewDecision:$reviewDecision,changesRequested:$changesRequested}}'
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

