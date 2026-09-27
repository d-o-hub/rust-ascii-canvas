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
| 8 | **canary** | RC published, opt-in consumers green | Release workflow |
| 9 | **promote / rollback** | released & deployed, or reverted | agent + ruleset |

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

```bash
# Local production build (optimized bundle, not the dev server)
cd web && pnpm run build && pnpm run preview
BASE_URL=http://localhost:4173 npx playwright test --project=chromium
```

For a Netlify **Deploy Preview** (created automatically per PR), point
`BASE_URL` at the deploy URL and run the full matrix. This stage is nearly free
and catches the class of bug that only appears in a bundled, optimized build.

### 8. Canary — what it honestly means here

This is a **static** app: no server, no traffic splitting, no feature-flag
service. So:

- **Do**: publish an opt-in **RC tag** (e.g. `v0.1.5-rc.1`) with the optimized
  WASM attached, and have a small set of testers/consumers take it explicitly.
- **Do not**: describe an RC tag as "5% of users". A true percentage rollout
  needs Netlify Split Testing, which is a paid feature this repo does not use.

An RC tag is a **smoke test with a wider audience**, not a statistics-based
canary. Say so when reporting.

### 9. Promote / rollback

```bash
# Promote
./scripts/release.sh                                   # preflight (read-only)
gh workflow run release.yml -f dry_run=true && gh run watch
gh workflow run release.yml && gh run watch
gh release view vX.Y.Z

# Roll back
git revert -m 1 <merge-sha> && git push origin main     # redeploys prior state
```

Rollback detail, including the versioned WASM asset, is in
[RELEASING.md](../plans/RELEASING.md#rollback).

## What the human still decides

Keep these in the loop — they are judgement, not chore:

- scope and specification
- architecture decisions (ADR)
- a **disputed** `pr-roast` Blocker
- production rollback when it is itself destructive

Remove these from the loop — the harness handles them:

- running gates and self-correcting
- the adversarial pass
- tracking and resolving review comments
- shadow E2E
- clicking merge (`gh pr merge --auto --squash`)

## Escalation

Stop and ask a human when:

- the repro cannot be automated
- rollback would itself be destructive (format change, data migration)
- a `pr-roast` Blocker is disputed
- a change relaxes a **merge guard-rail** — that needs an ADR and an explicit
  human decision, per `goap-adr-planner`

## Related

- `AGENTS.md` → *Merge & ship*
- [harness.md](harness.md) → controls, tiers, and the steering log (L-008)
- Skills: `verify`, `code-review`, `pr-roast`, `merge-gate`, `production-loop`
