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
**all five** hold:

| # | Condition | Enforced by |
|---|-----------|-------------|
| 1 | Not a draft | `pr-merge-gate.sh` |
| 2 | Mergeable — no conflicts (and *verified*, not `UNKNOWN`) | `pr-merge-gate.sh` |
| 3 | **Every** status check concluded `SUCCESS` | `pr-merge-gate.sh` + ruleset `required_status_checks` |
| 4 | **Every** review thread resolved | `pr-merge-gate.sh` (report) + ruleset `required_review_thread_resolution` |
| 5 | No `CHANGES_REQUESTED` **currently** (per `reviewDecision`) | `pr-merge-gate.sh` |

## When to Use

- Asked to merge a PR, or to check whether one is ready
- After `verify` + `code-review` are green, before the merge step
- To arm auto-merge so CI — not a human — gates the merge

## Don't Invoke When

- Gates are still red → run `verify` first
- The change is not product or harness work (e.g. merging a bot lockfile bump) — still run the gate, but skip the roast

## Procedure

```bash
# 1. Mechanical check (read-only; never merges)
npm run gate:pr            # or: ./scripts/pr-merge-gate.sh <PR>

# 2. If green, arm auto-merge. CI then performs the merge when it goes green.
gh pr merge <PR> --auto --squash

# 3. Confirm it is armed
gh pr view <PR> --json autoMergeRequest
```

`--auto --squash` is deliberate: the repo requires **linear history**, and
squash keeps one commit per PR. A merge commit is rejected by the ruleset.

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
- **Before** `pr-roast` findings are fixed — a roast verdict of *Request
  changes* is a reason to fix, not to merge
- **Related** `pr-roast`, `production-loop`, `agents-docs/delivery.md`
- **Sensor:** `scripts/pr-merge-gate.sh` (self-test: `--self-test`)
