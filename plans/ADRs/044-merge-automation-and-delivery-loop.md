# ADR-044: Merge Automation and the Delivery Loop

## Status
Accepted — 2026-09-27. Supersedes nothing; extends ADR-037 (harness engineering).

## Context

ADR-037 built a strong sensor suite: fmt, clippy, architecture, LOC, web lint/tsc/vitest, WASM size, E2E, cargo-audit, cargo-deny. But two questions were never asked of it:

1. **Does GitHub actually block on these results?** Auditing the `main` ruleset (`16137545`) on 2026-09-27 showed `required_status_checks` contained exactly one entry: `Codacy Static Code Analysis`. Every product sensor was **advisory**. A PR could merge with E2E red.
2. **Does a resolved review comment gate anything?** `required_review_thread_resolution: false`. Unresolved threads did not block, so review feedback could be silently dropped.

`allow_auto_merge` was already `true`, meaning auto-merge would have fired on Codacy alone — the weakest signal in the repo.

The gap was invisible because both sides looked green: CI ran, and GitHub showed no blocking rule. "CI runs" and "CI gates" are different properties, and only the second one is a control.

Separately, the workflow asked the human to perform each step manually. The cost was not the review itself but the choreography: running gates, chasing red CI, tracking which review comment was still open, re-running the suite against a preview, and finally clicking merge.

## Decision

1. **Adopt an explicit merge contract.** A PR may merge only when: not a draft; no conflicts; every status check `SUCCESS`; every review thread resolved; no outstanding `CHANGES_REQUESTED`; and the adversarial pass is clean.
2. **Enforce it in two places, for two different reasons.**
   - **Ruleset `main` is authoritative** — `CI Success` and `PR Readiness (merge gate)` become required checks, and `required_review_thread_resolution` becomes `true`. It cannot be bypassed without admin.
   - **`scripts/pr-merge-gate.sh` is the local mirror** — read-only, never merges. It exists so the agent discovers a blocked merge *before* pushing.
3. **Keep `required_approving_review_count: 0`.** The gates decide, not a human click. A human is required for scope, ADRs, disputed Blockers, and production rollback — judgement, not choreography.
4. **`CI Success` treats `cancelled` as failure.** Once it is a required check, a cancelled job must never read as green. Concurrency cancellation always triggers a fresh run, so this cannot deadlock the queue. `skipped` still passes, because path filters legitimately skip jobs on docs-only changes.
5. **Thread resolution is enforced by the ruleset, not a status check.** Resolving a thread produces no commit, so a `pull_request` job would never re-run and would block on stale data. The `PR Readiness` job therefore *reports* thread state and *fails* only on the offline self-test.
6. **Self-test the guard-rail.** No PR in this repo's history ever had a review thread, so the blocking branches could not be proven live. `--self-test` exercises every *predicate* against fixtures — including `CANCELLED`, `EXPECTED`, `conclusion:null`, a null GraphQL payload, and each `reviewDecision` — and `quality-gates.sh` runs it in the fast tier so the sensor cannot rot. Scope honestly: this proves the **logic**, not the end-to-end wiring against live GitHub state. A green `--self-test` with a real red PR would still be a defect; the ruleset is the backstop.
7. **Adopt the production loop** `production → failure → reproduce → candidate fix → evaluate → adversarial → shadow → canary → promote/rollback`, with an exit criterion per stage that a machine can check. Full runbook: `agents-docs/delivery.md`.
8. **Split review into two passes.** `code-review` asks "is this sound?"; `pr-roast` asks "how do we break this?", citing official upstream docs for every finding. Green gates are the precondition for the roast, not a substitute for it.
9. **Retire three skills** that no longer matched reality (`my-pull-requests`, `ln-732-cicd-generator`, `create-github-pull-request-from-specification`) and remove their `skills-lock.json` entries so a sync cannot resurrect them.

## Consequences

### Easier

- CI failure now blocks a merge instead of producing a red mark nobody must act on
- Review feedback cannot be dropped: unresolved threads block, server-side
- The agent carries gates → review → roast → merge without a human in the loop
- Shadow E2E is nearly free: `playwright.config.ts` already honours `BASE_URL`
- A guard-rail nobody has seen fire is provably a guard-rail (`--self-test`)

### Harder / trade-offs

- The ruleset is **repository state, not a file** — `git revert` does not undo it.
  The pre-mutation JSON is captured in the PR that introduced it.
- A PR can edit `.github/workflows/**` to drop a job from `ci-success.needs`,
  weakening the very check that requires it. `quality-gates.sh` §2d now fails
  when that job set drifts, but the sensor and the thing it protects live in the
  same file, so this is mitigation, not elimination.
- `strict_required_status_checks_policy` is on, so a PR must be up to date with
  the base branch before merging. That is intentional friction, but it can
  surprise contributors used to merging stale branches.
- Auto-merging without human approval means a **shared sensor defect is a shared
  auto-merge defect**. The self-test and the roast are the compensations, and
  both are inferential — they are not the same guarantee as a human reading the diff.
- The loop's "canary" stage is weaker than its name suggests: this is a static
  app with no traffic splitting, so the canary is an opt-in RC tag, not a
  percentage rollout. This is documented rather than papered over.

## Related

- `agents-docs/harness.md` — controls, tiers, merge contract, steering log (L-008, L-009)
- `agents-docs/delivery.md` — the delivery-loop runbook
- `scripts/pr-merge-gate.sh` — the merge contract as a sensor
- `plans/RELEASING.md` — promote/rollback
- ADR-037 (harness engineering), ADR-021 (production readiness)
