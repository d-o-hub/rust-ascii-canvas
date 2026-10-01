# Delivery Loop Runbook

**Scope**: getting a change from "something is broken" to "shipped or rolled
back" with the minimum possible human involvement.
**Skill**: `production-loop` · **Decision**: [ADR-044](../plans/ADRs/044-merge-automation-and-delivery-loop.md)
**Harness map**: [harness.md](harness.md) · **Release**: [RELEASING.md](../plans/RELEASING.md)

## The loop

```
production
   ↓  failure
reproduce
   ↓  candidate fix(s)
evaluate
   ↓  adversarial evaluation
shadow
   ↓  canary
promote / rollback
```

The point is not the diagram — it is that **each stage has an exit criterion
that a machine can check**. A stage you cannot verify is a stage you are guessing.

| # | Stage | Exit criterion | Who |
|---|-------|----------------|-----|
| 1 | **production** | `main` deployed, sensors green | CI + Netlify |
| 2 | **failure** | concrete symptom: steps, input, expected vs actual | reporter / `dogfood` |
| 3 | **reproduce** | an automated test that fails for this reason | agent |
| 4 | **candidate fix** | ≥2 considered, one chosen with reasoning | agent |
| 5 | **evaluate** | `gate:fast` green | `verify` |
| 6 | **adversarial** | no Blocker/Major outstanding | `pr-roast` |
| 7 | **shadow** | E2E matrix green against a production-shaped build | agent |
| 8 | **canary** | **Unsupported** until prerelease policy/workflow is implemented | future design, not an available command |
| 9 | **promote / rollback** | stable release deployed, or human-authorized revert PR merged and verified | agent + normal merge contract |

## Stage notes

### 2 → 3. Failure → reproduce

A bug without an automated reproduction cannot be *proven* fixed, only
believed fixed. Write the failing test first.

```bash
cargo test <module> -- --nocapture                    # core behaviour
npx playwright test --project=chromium e2e/<file>.spec.ts   # browser behaviour
```

Record the repro in the PR body. If you cannot automate it, **stop and escalate**
(see *Escalation* below).

### 4. Candidate fix

State at least two approaches and why the chosen one wins. One candidate is not
a decision.

### 5. Evaluate

```bash
npm run gate:fast          # iterate here
npm run gate:full          # before the PR is mergeable
```

### 6. Adversarial evaluation

```bash
npm run gate:pr            # mechanical merge contract
# then the inferential pass:
#   skill pr-roast
```

Green gates are the **precondition** for roasting, not a substitute for it.
Every defect the roast finds is, by construction, one the sensors missed.

### 7. Shadow

Run the real suite against a build that serves no users.
`playwright.config.ts` already honours `BASE_URL`, so no config change is needed.

Use the local production-build procedure in the
[verify skill](../.agents/skills/verify/SKILL.md). Playwright owns server startup,
readiness and teardown; do not leave a separately started Vite server running.
For an already-running deployment, run from the repository root:

```bash
BASE_URL=https://<deploy-preview-host> npx playwright test --project=chromium --project=firefox --project=webkit
```

For a Netlify **Deploy Preview** (created automatically per PR), point
`BASE_URL` at the deploy URL and run the full matrix. This stage is nearly free
and catches the class of bug that only appears in a bundled, optimized build.

### 8. Canary — what it honestly means here

**RC publication/promotion is unsupported today.** ADR-044 describes an opt-in
RC design, but `release.yml` and `scripts/release.sh` both validate only `x.y.z`
and reject prereleases. Do not create an RC manually to bypass those guards.

This static app has no traffic splitting. A future RC would be an opt-in smoke
test with a wider audience, not "5% of users". Until a separately reviewed
release-policy change adds prerelease fixtures and workflow support, use Deploy
Previews for shadow testing and report the missing canary stage honestly.
See the active backlog in [FOLLOW_UPS.md](../plans/FOLLOW_UPS.md).

### 9. Promote / rollback

```bash
# Promote
./scripts/release.sh                                   # preflight (read-only)
gh workflow run release.yml -f dry_run=true && gh run watch
gh workflow run release.yml && gh run watch
gh release view vX.Y.Z

# Roll back ONLY after explicit human authorization, on a new branch
# Follow RELEASING.md: git revert <squash-sha>, verify, open PR, roast, merge gate.
# Never push a rollback directly to main.
```

`git revert` takes **no** `-m` flag here: this repo mandates squash merges
(`AGENTS.md`), so each PR is a single commit. `-m 1` is for merge commits and
will fail. Full runbook: [RELEASING.md](../plans/RELEASING.md#rollback).

## What the human still decides

Keep these in the loop — they are judgement, not chore:

- scope and specification
- architecture decisions (ADR)
- a **disputed** `pr-roast` Blocker
- **every production rollback authorization**, including an apparently simple revert;
  destructive/data-format risk requires an explicit recovery plan too

Remove these from the loop — the harness handles them:

- running gates and self-correcting
- the adversarial pass
- tracking and resolving review comments
- shadow E2E
- clicking merge (`gh pr merge --auto --squash`)

## Escalation

Stop and ask a human when:

- the repro cannot be automated
- a production rollback is proposed (always obtain human authorization)
- rollback would itself be destructive (format change, data migration)
- a `pr-roast` Blocker is disputed
- a change relaxes a **merge guard-rail** — that needs an ADR and an explicit
  human decision, per `goap-adr-planner`

## Related

- `AGENTS.md` → *Merge & ship*
- [harness.md](harness.md) → controls, tiers, and the steering log (L-008)
- Skills: `verify`, `code-review`, `pr-roast`, `merge-gate`, `production-loop`
