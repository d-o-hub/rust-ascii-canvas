---
name: merge-gate
description: >
  Decide and perform the merge of a pull request against the repo's merge
  contract. Use when asked to "merge the PR", "can this merge", "is it ready to
  merge", "arm auto-merge", or before invoking `gh pr merge`. Runs
  scripts/pr-merge-gate.sh (computational), then decides. Enforces "all CI green
  + all review threads resolved"; never merges a red PR.
---

# Merge Gate

The **merge contract** for this repo, in one place. A PR may be merged when
**all six checks below** hold (the first five are mechanical; the roast is an agent procedure):

| # | Condition | Enforced by |
|---|-----------|-------------|
| 1 | Not a draft | `pr-merge-gate.sh` |
| 2 | Mergeable — no conflicts (and *verified*, not `UNKNOWN`) | `pr-merge-gate.sh` |
| 3 | **Every** status check concluded `SUCCESS` | `pr-merge-gate.sh` + ruleset `required_status_checks` |
| 4 | **Every** review thread resolved | `pr-merge-gate.sh` (report) + ruleset `required_review_thread_resolution` |
| 5 | No `CHANGES_REQUESTED` **currently** (per `reviewDecision`) | `pr-merge-gate.sh` |
| 6 | Adversarial pass clean: no unresolved Blockers/Majors | `pr-roast` (not encoded by `gate:pr`) |

## When to Use

- Asked to merge a PR, or to check whether one is ready
- After `verify` + `code-review` are green, before the merge step
- To arm auto-merge so CI — not a human — gates the merge

## Don't Invoke When

- Gates are still red → run `verify` first
- No exception for bot, dependency, or documentation PRs: run the mechanical gate **and** the adversarial pass for every PR.

## Procedure

```bash
# 1. Mechanical check (read-only; never merges)
npm run gate:pr            # or: ./scripts/pr-merge-gate.sh <PR>

# 2. Run pr-roast on the current head; fix every Blocker/Major and re-verify.
#    Disputed Blocker: stop for human judgment. No bot/dependency exemption.
#    Only after BOTH gate:pr and the roast are clean, arm auto-merge.
gh pr merge <PR> --auto --squash

# 3. Confirm it is armed
gh pr view <PR> --json autoMergeRequest
```

`--auto --squash` is deliberate: the repo requires **linear history**, and
squash keeps one commit per PR. A merge commit is rejected by the ruleset.

## The ruleset is state, not a file

Conditions 1–4 above are enforced by the **`main` ruleset**, which lives in
GitHub rather than in git. Three consequences you must respect:

- `git revert` does **not** undo a ruleset change. Rollback is
  `./scripts/ruleset-check.sh --restore`.
- A PR diff never shows a ruleset change, so it cannot be reviewed as one.
- Nothing else detects drift, so it is snapshotted in
  `.github/ruleset-main.json` and checked by `npm run gate:ruleset` — a
  **required** status check.

```bash
npm run gate:ruleset                  # fail on drift
./scripts/ruleset-check.sh --diff     # snapshot vs live
./scripts/ruleset-check.sh --restore  # roll the contract back
```

Never change the merge contract without an ADR (`goap-adr-planner`) **and** an
updated snapshot in the same change. Relaxing a guard-rail here is the L-008
failure mode.

## Arming vs. merging directly

| Situation | Command |
|---|---|
| CI is green **now** and should stay green | `gh pr merge <PR> --auto --squash` |
| Prefer a human click | `gh pr merge <PR> --squash` |
| Merge is blocked and you disagree with the reason | **Do not bypass.** Fix it, or ask the human. |

Never use `--admin` to force past a failing check. That defeats the entire
harness and is the one action this skill exists to prevent.

## Known states

- **`ZERO status checks`** → the sensor fails on purpose. A PR that ran no
  sensors is *unverified*, not green (harness L-002 / L-003).
- **`mergeable: UNKNOWN`** → blocks. GitHub returns this while it computes
  conflicts (routine right after a push). The gate fails closed rather than
  guessing; re-run in ~10s.
- **A cancelled check** → blocks. It is not a pass and not "pending" — it is
  unverified. Wait for the fresh run.
- **Checks still running** → re-run in a minute; a queued check is not a pass.
- **Unresolved threads** → address the code, reply, then *Resolve thread*.
  The ruleset blocks the merge server-side, so it cannot be skipped.
- **Unauthenticated `gh`** → the gate errors rather than assuming "fine".
  Never treat "could not verify" as "verified".
- **A dismissed `CHANGES_REQUESTED`** → does *not* block. The gate uses
  GitHub's `reviewDecision` roll-up, which already accounts for dismissal and
  staleness. Counting raw reviews would deadlock a fixed PR forever.

## Output format

```
MERGEABLE / MERGE BLOCKED
- unmet conditions (with the PR, path, or check name that caused each)
- the exact next command
```

`--json` prints the machine-readable verdict to **stdout** and all human-facing
lines to **stderr**, so `npm run gate:pr -- --json | jq .` is safe to pipe.

## Integration

- **After** `verify` and `code-review`
- **After** `pr-roast` findings are fixed and the current head is re-verified —
  a roast verdict of *Request changes* blocks merging and arming auto-merge
- **Related** `pr-roast`, `production-loop`, `agents-docs/delivery.md`
- **Sensor:** `scripts/pr-merge-gate.sh` (self-test: `--self-test`)
- **Server-side state:** `scripts/ruleset-check.sh` + `.github/ruleset-main.json`
