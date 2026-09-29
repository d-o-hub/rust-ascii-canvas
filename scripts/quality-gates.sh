#!/usr/bin/env bash
# scripts/quality-gates.sh
# Computational feedback sensors for the ASCII Canvas harness.
# See agents-docs/harness.md and ADR-037.
#
# Usage:
#   ./scripts/quality-gates.sh           # full tier (pre-PR / CI)
#   ./scripts/quality-gates.sh --fast    # fast tier (agent loop / pre-commit)
#   ./scripts/quality-gates.sh --fix    # auto-fix fmt/clippy where possible
#   ./scripts/quality-gates.sh --fast --fix
#
# Exit 0 = success, Exit 1 = errors.
# Failure lines include FIX: hints for agent self-correction.
set +e
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT" || exit 1

readonly MAX_LINES_PER_SOURCE_FILE=500
readonly LOC_ALLOWLIST_FILE="${REPO_ROOT}/.loc-allowlist"

FIX=false
FAST=false
for arg in "$@"; do
  case $arg in
    --fix) FIX=true ;;
    --fast) FAST=true ;;
    -h|--help)
      sed -n '2,16p' "$0"
      exit 0
      ;;
    *) echo "Unknown argument: $arg (try --fast, --fix, --help)"; exit 1 ;;
  esac
done

if [[ -t 1 ]] && [[ "${FORCE_COLOR:-}" != "0" ]]; then
  RED='\033[0;31m'
  GREEN='\033[0;32m'
  YELLOW='\033[1;33m'
  BLUE='\033[0;34m'
  NC='\033[0m'
else
  RED=''; GREEN=''; YELLOW=''; BLUE=''; NC=''
fi

pass() { echo -e "${GREEN}[PASS]${NC} $1"; }
fail() { echo -e "${RED}[FAIL]${NC} $1"; FAILED=1; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
info() { echo -e "${BLUE}[INFO]${NC} $1"; }

FAILED=0
TIER="full"
if $FAST; then TIER="fast"; fi

printf "Quality gates (tier=%s)...\n\n" "$TIER"

# The LOC check itself lives in scripts/check-loc.sh so the gate script and the
# `web` CI job run ONE implementation. It used to be inline here, which meant it
# ran in the developer's shell and in CI nowhere (L-016) — the same trap this repo
# hit for the audit sensor (L-013), the e2e lint scope (L-014) and the merge-gate
# coherence list (L-018). See scripts/check-loc.sh and L-020.
#
# `is_allowlisted` and the inline loop are gone; check-loc.sh owns the whole
# policy, including the ratchet (an allowlist entry pins a budget a file may
# only shrink from) and stale-entry reporting.

# ============================================================
# 1. LOC LIMITS
# ============================================================
info "LOC limits (max ${MAX_LINES_PER_SOURCE_FILE}; src/**/*.rs + web/*.ts)..."
if LOC_OUTPUT=$(bash "$REPO_ROOT/scripts/check-loc.sh" 2>&1); then
  printf "%s\n" "$LOC_OUTPUT" | sed 's/^/  /'
else
  printf "%s\n" "$LOC_OUTPUT" | sed 's/^/  /'
  fail "LOC limits"
  echo "  FIX: See the entries above. Split the file, or pin a budget in .loc-allowlist"
  echo "       (an ADR is required to add an entry)."
fi
printf "\n"

# ============================================================
# 2. ARCHITECTURE FITNESS
# ============================================================
info "Architecture fitness..."
if [[ -x "$REPO_ROOT/scripts/check-architecture.sh" ]] || [[ -f "$REPO_ROOT/scripts/check-architecture.sh" ]]; then
  if bash "$REPO_ROOT/scripts/check-architecture.sh"; then
    :
  else
    fail "Architecture: layer violations (see above)"
    echo "  FIX: Read agents-docs/architecture.md and remove illegal imports."
  fi
else
  warn "Architecture script missing"
fi
printf "\n"

# ============================================================
# 2b. WASM-BINDGEN PIN PARITY (harness L-005)
# ============================================================
# Dependabot bumps Cargo.toml alone; the CLI pin lives in mise.toml,
# package.json netlify:build, and CI (mise-action). Dep and CLI schemas
# must match exactly or `wasm-bindgen` fails with a schema-version error.
info "wasm-bindgen pin parity (L-005)..."
WBG_CARGO="$(sed -n 's/^wasm-bindgen = \"=\([0-9.]*\)\".*/\1/p' "$REPO_ROOT/Cargo.toml" | head -n 1)"
WBG_MISE="$(sed -n 's/.*cargo:wasm-bindgen-cli\" = \"\([0-9.]*\)\".*/\1/p' "$REPO_ROOT/mise.toml" | head -n 1)"
WBG_NETLIFY="$(sed -n 's/.*wasm-bindgen-cli --version \([0-9.]*\).*/\1/p' "$REPO_ROOT/package.json" | head -n 1)"
if [[ -z "$WBG_CARGO" ]] || [[ -z "$WBG_MISE" ]] || [[ -z "$WBG_NETLIFY" ]]; then
  fail "wasm-bindgen pins unreadable (cargo=$WBG_CARGO mise=$WBG_MISE netlify=$WBG_NETLIFY)"
  echo "  FIX: Keep =X.Y.Z pins in Cargo.toml, mise.toml, package.json netlify:build."
else
  if [[ "$WBG_CARGO" != "$WBG_MISE" ]] || [[ "$WBG_CARGO" != "$WBG_NETLIFY" ]]; then
    fail "wasm-bindgen pin skew (cargo=$WBG_CARGO mise=$WBG_MISE netlify=$WBG_NETLIFY)"
    echo "  FIX: Bump all three together (Cargo.toml + mise.toml + netlify:build + Cargo.lock), then npm run build:wasm."
    echo "  See agents-docs/harness.md L-005."
  else
    pass "wasm-bindgen pins aligned ($WBG_CARGO)"
  fi
fi
printf "\n"

# ============================================================
# 2c. VERSION PIN PARITY (single source: VERSION)
# ============================================================
# VERSION is the single source of truth; scripts/propagate-version.sh writes it
# to Cargo.toml / package.json / web/package.json. Checking here makes dev-time
# drift fail the fast tier; scripts/release.sh is the release-time guard.
info "Version pin parity (VERSION SSOT)..."
if ! OUTPUT=$(bash "$REPO_ROOT/scripts/propagate-version.sh" --check 2>&1); then
  fail "Version pins drifted from VERSION"
  echo "  FIX: run ./scripts/propagate-version.sh (see plans/RELEASING.md)."
  printf "%s\n" "$OUTPUT" >&2
else
  pass "Version pins aligned ($(tr -d '[:space:]' < "$REPO_ROOT/VERSION"))"
fi
printf "\n"

# ============================================================
# 2d. MERGE-GATE COHERENCE (harness L-008)
# CI Success is a REQUIRED status check, so the set of jobs it aggregates is
# load-bearing. A PR can edit .github/workflows/**, so without this sensor a
# change could quietly drop a job from `needs` and the check would still go
# green — a guard-rail weakened by an edit to the guard-rail itself.
info "Merge-gate coherence (L-008)..."
CI_YML="$REPO_ROOT/.github/workflows/ci.yml"
if [[ -f "$CI_YML" ]]; then
  # The jobs whose results CI Success aggregates. Keep in sync with the workflow —
  # the check below is bidirectional precisely so that forgetting to is a failure
  # rather than a silent hole (see L-018).
  MERGE_JOBS=(fmt clippy architecture rust security security-npm deny loc web wasm e2e)
  # Read the `needs:` line of the ci-success job. It is NOT at a fixed offset:
  # comment blocks above it (which explain *why* the list matters) push it
  # further down, so scan until the next top-level job key.
  NEEDS_LINE="$(awk '
    /^  ci-success:/ { injob=1; next }
    injob && /^  [a-zA-Z0-9_-]+:/ { exit }
    injob && /needs:/ { print; exit }
  ' "$CI_YML")"
  COHERENCE_OK=true

  # (a) every gate job must be aggregated by CI Success.
  for job in "${MERGE_JOBS[@]}"; do
    if ! grep -qw "$job" <<<"$NEEDS_LINE"; then
      COHERENCE_OK=false
      echo "  ci-success.needs is missing '$job'"
    fi
  done

  # (b) every aggregated job must be one this harness knows about. A one-way
  # subset check passes when a new sensor job is added to the workflow and wired
  # into `needs` — and then that job has no guard: dropping it from `needs`
  # later is invisible, and CI Success reports green without it. Bidirectional is
  # the difference between "the list matches" and "the list happens to match".
  # Parse only what is inside the brackets, so the `needs:` key itself is not
  # mistaken for a job name.
  NEEDS_JOBS="$(sed -e 's/.*\[//' -e 's/\].*//' <<<"$NEEDS_LINE" | tr ',' ' ')"
  for job in $NEEDS_JOBS; do
    job="$(xargs <<<"$job")"
    [[ -z "$job" || "$job" == "changes" ]] && continue
    known=false
    for candidate in "${MERGE_JOBS[@]}"; do
      [[ "$job" == "$candidate" ]] && known=true && break
    done
    if ! $known; then
      COHERENCE_OK=false
      echo "  ci-success aggregates '$job', which is not in MERGE_JOBS (quality-gates.sh)"
    fi
  done

  # (c) every gate job must actually exist in the workflow, so a renamed job
  # cannot satisfy (a) with a stale name while the real one runs unaggregated.
  for job in "${MERGE_JOBS[@]}"; do
    if ! grep -qE "^  ${job}:" "$CI_YML"; then
      COHERENCE_OK=false
      echo "  MERGE_JOBS lists '$job', which is not a job in ci.yml"
    fi
  done

  # `changes` must be present: if it fails, every dependant is skipped and
  # CI Success would see all-skipped and report green (L-002/L-003).
  if ! grep -qw "changes" <<<"$NEEDS_LINE"; then
    COHERENCE_OK=false
    echo "  ci-success.needs is missing 'changes' (all-skipped would read as green)"
  fi
  if $COHERENCE_OK; then
    pass "ci-success aggregates all ${#MERGE_JOBS[@]} sensors + changes (bidirectional, L-018)"
  else
    fail "ci-success.needs drifted from the required job set"
    local want
    want="$(IFS=,; echo "${MERGE_JOBS[*]}")"
    echo "  FIX: needs: [changes, ${want}] on the ci-success job, with MERGE_JOBS"
    echo "       in this script listing exactly those jobs."
    echo "       It is a REQUIRED status check; dropping a job silently weakens the merge gate."
    echo "       See agents-docs/harness.md L-008."
  fi

  # The merge gate must be able to prove itself.
  if [[ -x "$REPO_ROOT/scripts/pr-merge-gate.sh" || -f "$REPO_ROOT/scripts/pr-merge-gate.sh" ]]; then
    if bash "$REPO_ROOT/scripts/pr-merge-gate.sh" --self-test >/dev/null 2>&1; then
      pass "Merge-gate self-test: predicates still block correctly"
    else
      fail "Merge-gate self-test FAILED"
      echo "  FIX: scripts/pr-merge-gate.sh --self-test must pass. A required check that"
      echo "       can no longer detect a red PR must not ship."
    fi
  else
    fail "scripts/pr-merge-gate.sh is missing"
    echo "  FIX: restore it; 'npm run gate:pr' is the local mirror of the merge contract."
  fi

  # Offline half of the ruleset guard. The ruleset is repository state, so its
  # drift is only detectable against a committed snapshot. This checks the
  # snapshot exists, parses, and still names the required checks — no network
  # needed. The live comparison lives in scripts/ruleset-check.sh (CI only),
  # because it needs the network and would false-fail offline.
  RULESET_SNAP="$REPO_ROOT/.github/ruleset-main.json"
  if [[ ! -f "$RULESET_SNAP" ]]; then
    fail ".github/ruleset-main.json is missing"
    echo "  FIX: ./scripts/ruleset-check.sh --update, then commit the result."
    echo "       Without it, merge-contract changes are invisible to code review."
  elif ! command -v jq >/dev/null 2>&1; then
    warn "jq missing — skipped ruleset snapshot validation"
  else
    SNAP_OK=true
    for ctx in "CI Success" "PR Readiness (merge gate)"; do
      if ! jq -e --arg c "$ctx" '.requiredStatusChecks | index($c) != null' \
           "$RULESET_SNAP" >/dev/null 2>&1; then
        SNAP_OK=false
        echo "  ruleset snapshot does not require '$ctx'"
      fi
    done
    if ! jq -e '.requiredReviewThreadResolution == true' "$RULESET_SNAP" >/dev/null 2>&1; then
      SNAP_OK=false
      echo "  ruleset snapshot does not require review-thread resolution"
    fi
    if $SNAP_OK; then
      pass "Ruleset snapshot requires CI Success + PR Readiness + thread resolution"
    else
      fail "Ruleset snapshot no longer encodes the merge contract"
      echo "  FIX: if intentional, update the snapshot in the SAME change as the"
      echo "       ruleset (ADR required). Otherwise: ./scripts/ruleset-check.sh --restore"
    fi
  fi
else
  warn "ci.yml not found — skipped merge-gate coherence check"
fi
printf "\n"

# ============================================================
# 3. RUST CHECKS
# ============================================================
info "Rust checks..."

if $FIX; then
  cargo fmt --all
  pass "Format: auto-fixed"
else
  if ! OUTPUT=$(cargo fmt --all -- --check 2>&1); then
    fail "Format"
    echo "  FIX: run 'cargo fmt --all' (or ./scripts/quality-gates.sh --fix)"
    printf "%s\n" "$OUTPUT" >&2
  else
    pass "Format: OK"
  fi
fi

if $FIX; then
  cargo clippy --fix --allow-dirty --allow-staged --all-targets --all-features
  pass "Clippy: auto-fixed (re-run without --fix to verify -D warnings)"
else
  if ! OUTPUT=$(cargo clippy --all-targets --all-features -- -D warnings 2>&1); then
    fail "Clippy"
    echo "  FIX: Address clippy lints. Prefer idiomatic fixes over #[allow]."
    printf "%s\n" "$OUTPUT" >&2
  else
    pass "Clippy: OK"
  fi
fi

if ! OUTPUT=$(cargo build --all-targets 2>&1); then
  fail "Build"
  echo "  FIX: Fix compile errors above before continuing."
  printf "%s\n" "$OUTPUT" >&2
else
  pass "Build: OK"
fi

if ! OUTPUT=$(cargo test --all 2>&1); then
  fail "Tests"
  echo "  FIX: Fix failing tests; do not skip or weaken assertions to pass."
  printf "%s\n" "$OUTPUT" >&2
else
  pass "Tests: OK"
fi
printf "\n"

# ============================================================
# 4. WEB CHECKS (when web/ exists)
# ============================================================
# LEARNING (PR #128): web/pkg is gitignored. tsc imports ./pkg/ascii_canvas.js.
# Local trees often have a stale/cached pkg so gate:fast passes while CI fails.
# Always require pkg bindings before typecheck (build if missing).
if [[ -d "$REPO_ROOT/web" ]]; then
  info "Web checks (lint, typecheck, unit tests)..."

  if [[ ! -f "$REPO_ROOT/web/pkg/ascii_canvas.d.ts" ]] || [[ ! -f "$REPO_ROOT/web/pkg/ascii_canvas.js" ]]; then
    info "web/pkg missing (gitignored) — building WASM for typecheck parity with CI..."
    if ! OUTPUT=$(pnpm run build:wasm 2>&1); then
      fail "web/pkg / WASM build"
      echo "  FIX: npm run build:wasm (mise: wasm-bindgen-cli 0.2.128, target wasm32-unknown-unknown)."
      echo "  WHY: main.ts imports ./pkg/ascii_canvas.js; CI downloads wasm-pkg artifact before tsc."
      printf "%s\n" "$OUTPUT" >&2
    else
      pass "WASM pkg generated for typecheck"
    fi
  else
    pass "web/pkg bindings present"
  fi

  pushd "$REPO_ROOT/web" >/dev/null || exit 1

  if [[ ! -d node_modules ]]; then
    warn "web/node_modules missing — running pnpm install"
    pnpm install --frozen-lockfile 2>/dev/null || pnpm install
  fi

  if ! OUTPUT=$(pnpm run lint 2>&1); then
    fail "ESLint"
    echo "  FIX: cd web && pnpm run lint — fix reported issues."
    printf "%s\n" "$OUTPUT" >&2
  else
    pass "ESLint: OK"
  fi

  if command -v pnpm >/dev/null && pnpm exec tsc --version >/dev/null 2>&1; then
    if ! OUTPUT=$(pnpm exec tsc --noEmit 2>&1); then
      fail "TypeScript"
      echo "  FIX: cd web && pnpm exec tsc --noEmit — fix type errors (strict mode)."
      echo "  IF 'Cannot find module ./pkg/ascii_canvas.js': run npm run build:wasm (pkg is gitignored)."
      printf "%s\n" "$OUTPUT" >&2
    else
      pass "TypeScript: OK"
    fi
  else
    warn "tsc not available; skipped typecheck"
  fi

  if ! OUTPUT=$(pnpm test 2>&1); then
    fail "Vitest"
    echo "  FIX: cd web && pnpm test — fix unit tests."
    printf "%s\n" "$OUTPUT" >&2
  else
    pass "Vitest: OK"
  fi

  popd >/dev/null || true
  printf "\n"
fi

# --- Root lint + e2e typecheck (fast + full) ---------------------------
# The web lint above runs `cd web && eslint .`, which structurally cannot see
# anything outside web/. Before this existed, e2e/ was unlinted locally and its
# issues surfaced only in the required Codacy check (harness L-014). Root config
# lives in eslint.config.mjs and deliberately ignores web/ so the two never
# overlap.
if [[ -d "$REPO_ROOT/node_modules" ]]; then
  info "Root lint (e2e/, excludes web/)..."
  if ! OUTPUT=$(cd "$REPO_ROOT" && ./node_modules/.bin/eslint . 2>&1); then
    fail "ESLint (root)"
    echo "  FIX: ./node_modules/.bin/eslint . — covers e2e/ (web/ has its own config)."
    printf "%s\n" "$OUTPUT" >&2
  else
    pass "ESLint (root): OK"
  fi
else
  warn "root node_modules missing; skipped root ESLint (run pnpm install)"
fi

if [[ -x "$REPO_ROOT/web/node_modules/.bin/tsc" ]]; then
  if ! OUTPUT=$(cd "$REPO_ROOT/e2e" && ../web/node_modules/.bin/tsc --noEmit -p tsconfig.json 2>&1); then
    fail "TypeScript (e2e)"
    echo "  FIX: cd e2e && ../web/node_modules/.bin/tsc --noEmit -p tsconfig.json"
    printf "%s\n" "$OUTPUT" >&2
  else
    pass "TypeScript (e2e): OK"
  fi
else
  warn "tsc not available; skipped e2e typecheck"
fi
printf "\n"

# ============================================================
# 5. PRIVACY + SECRETS (fast + full)
# ============================================================
info "Privacy (no real emails)..."
EMAIL_PATTERN='[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}'
EXCLUDE_PATTERN='example\.com|example\.org|test\.com|\.git|target|\.opencode|\.mimocode|node_modules|playwright-report|test-results|\.md|references|agents-docs|\.agents'

if grep -rE "$EMAIL_PATTERN" \
  --exclude-dir=.git --exclude-dir=target --exclude-dir=.opencode --exclude-dir=.mimocode \
  --exclude-dir=node_modules --exclude-dir=references --exclude-dir=playwright-report \
  --exclude-dir=test-results --exclude-dir=.agents \
  . 2>/dev/null | grep -vE "$EXCLUDE_PATTERN"; then
  fail "Email address detected"
  echo "  FIX: Remove personal emails; use example.com placeholders."
else
  pass "Privacy: OK"
fi
printf "\n"

info "Secret scan..."
SECRET_PATTERN="(api_key|token|secret|password|auth|key)[[:space:]]*[:=][[:space:]]*['\"][a-zA-Z0-9_\-]{16,}['\"]"
EXCLUDE_SECRET='example\.com|example\.org|test\.com|GITHUB_TOKEN|CARGO_REGISTRY_TOKEN|worktree|shared-key|release-workflow'

if grep -rE "$SECRET_PATTERN" \
  --exclude-dir=.git --exclude-dir=target --exclude-dir=.agents --exclude-dir=.opencode \
  --exclude-dir=node_modules --exclude-dir=playwright-report --exclude-dir=test-results \
  . 2>/dev/null | grep -vE "$EXCLUDE_SECRET"; then
  fail "Potential secret detected"
  echo "  FIX: Remove secrets; use env vars / GitHub Actions secrets."
else
  pass "Secret scan: OK"
fi
printf "\n"

# ============================================================
# 6. FULL-ONLY: security, WASM, size, E2E
# ============================================================
if ! $FAST; then
  if command -v cargo-audit &>/dev/null; then
    info "Security audit (Rust / cargo-audit)..."
    AUDIT_OUTPUT=$(cargo audit 2>&1) && AUDIT_EXIT=$? || AUDIT_EXIT=$?
    if [ "$AUDIT_EXIT" -ne 0 ]; then
      if echo "$AUDIT_OUTPUT" | grep -q "unsupported CVSS version"; then
        warn "cargo-audit: skipped (advisory format issue)"
      else
        fail "Security audit (cargo-audit)"
        echo "  FIX: Review cargo audit output; update or yank vulnerable crates."
        printf "%s\n" "$AUDIT_OUTPUT" >&2
      fi
    else
      pass "Security audit (cargo-audit): OK — crates only, not npm"
    fi
    printf "\n"
  else
    warn "cargo-audit not installed (CI runs it)"
  fi

  # npm advisories. L-013: the step above is crates-only, and its old label
  # ("Audit: OK") is ecosystem-neutral, so a lockfile refresh read as "audited"
  # when no package had been checked. Both lockfiles are audited here because
  # root and web/ are installed and versioned separately.
  #
  # Runs HERE ONLY — the CI side is the `security-npm` job in ci.yml (L-016).
  info "Security audit (npm / pnpm audit, root + web)..."
  if OUTPUT=$(bash "$REPO_ROOT/scripts/npm-audit.sh" 2>&1); then
    printf "%s\n" "$OUTPUT" | sed 's/^/  /'
  else
    printf "%s\n" "$OUTPUT" | sed 's/^/  /'
    fail "Security audit (npm)"
    echo "  FIX: Update or remove the flagged package. Do NOT silence with an audit"
    echo "       ignore or a resolution override — say why in the PR if unavoidable."
  fi
  printf "\n"

  # Codacy repo-level intake. Runs HERE ONLY — no CI job runs
  # quality-gates.sh (L-016), so this is local visibility, not a gate. The thing
  # that blocks a merge is the required `Codacy Static Code Analysis` check;
  # this exists because that check is *diff-scoped* and reported a green PR
  # while 11 High findings sat on the repo-level backlog (L-017). The script
  # exits 0 with a warning when the CLI is absent or unauthenticated, so an
  # offline machine is not blocked — and it never prints a pass it did not earn.
  info "Codacy repo-level intake (local only — not a CI gate)..."
  if OUTPUT=$(bash "$REPO_ROOT/scripts/codacy-check.sh" 2>&1); then
    printf "%s\n" "$OUTPUT" | sed 's/^/  /'
  else
    printf "%s\n" "$OUTPUT" | sed 's/^/  /'
    fail "Codacy repo-level intake"
    echo "  FIX: Read the findings above and fix them in code, or escalate with them quoted."
    echo "        Do NOT remove Codacy from the ruleset to make this green (AGENTS.md)."
  fi
  printf "\n"

  if command -v cargo-deny &>/dev/null; then
    info "cargo-deny..."
    if ! OUTPUT=$(cargo deny check 2>&1); then
      fail "cargo-deny"
      echo "  FIX: Align dependencies with deny.toml licenses/sources."
      printf "%s\n" "$OUTPUT" >&2
    else
      pass "cargo-deny: OK"
    fi
    printf "\n"
  fi

  info "WASM build + size..."
  if ! OUTPUT=$(pnpm run build:wasm 2>&1); then
    fail "WASM build"
    echo "  FIX: Ensure rustup target wasm32-unknown-unknown and wasm-bindgen-cli 0.2.128 (mise.toml)."
    printf "%s\n" "$OUTPUT" >&2
  else
    pass "WASM build: OK"
    if ! OUTPUT=$(pnpm run check-size 2>&1); then
      fail "WASM size"
      echo "  FIX: Reduce binary size (opt-level z already on); avoid heavy deps in wasm path."
      printf "%s\n" "$OUTPUT" >&2
    else
      pass "WASM size: OK"
      printf "%s\n" "$OUTPUT"
    fi
  fi
  printf "\n"

  info "E2E (Playwright chromium)..."
  if [[ ! -d "$REPO_ROOT/node_modules" ]]; then
    pnpm install --frozen-lockfile 2>/dev/null || pnpm install
  fi
  # The dev server is started, awaited and torn down by Playwright's `webServer`
  # in playwright.config.ts — the single source of truth, shared with the CI e2e
  # job. This block used to be a second hand-rolled copy of that sequence
  # (nohup + curl poll + kill-by-port); it had already drifted, and the CI copy
  # of it did not fail when the server never came up.
  #
  # `web/pkg` must exist before this point: the WASM bindings are gitignored and
  # vite will not build without them (harness L-001). The WASM build step above
  # guarantees it here; the CI e2e job downloads the artifact.
  if ! OUTPUT=$(npx playwright test --project=chromium --timeout=120000 2>&1); then
    fail "E2E"
    echo "  FIX: Inspect Playwright report; fix product or test. Prefer POM helpers over waits."
    echo "       A '[WebServer]' line in the output means the dev server itself failed;"
    echo "       playwright.config.ts `webServer` reports that directly (no manual boot)."
    printf "%s\n" "$OUTPUT" >&2
  else
    pass "E2E: OK"
  fi
  printf "\n"
else
  info "Skipping full-tier sensors (WASM, size, E2E, audit). Run without --fast before PR."
  printf "\n"
fi

# ============================================================
# SUMMARY
# ============================================================
if [[ $FAILED -ne 0 ]]; then
  printf "${RED}─────────────────────────────────────────────────────────────────${NC}\n"
  printf "${RED}│ Quality Gate FAILED (tier=%s)%*s│${NC}\n" "$TIER" $((40 - ${#TIER})) ""
  printf "${RED}│ Self-correct using FIX: hints above, then re-run.             │${NC}\n"
  printf "${RED}─────────────────────────────────────────────────────────────────${NC}\n"
  exit 1
fi

printf "${GREEN}─────────────────────────────────────────────────────────────────${NC}\n"
printf "${GREEN}│ All quality gates PASSED (tier=%s)%*s│${NC}\n" "$TIER" $((37 - ${#TIER})) ""
printf "${GREEN}─────────────────────────────────────────────────────────────────${NC}\n"
if $FAST; then
  printf "Next: npm run gate:full before opening a PR.\n"
fi
exit 0
