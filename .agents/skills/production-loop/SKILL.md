---
name: production-loop
description: >
  Run the production→failure→reproduce→fix→evaluate→adversarial→shadow→canary→
  promote/rollback loop for this repo. Use when a production bug arrives, when
  shipping a risky change, when asked to "roll this out safely", "canary",
  "promote", or "roll back". Defines the exit criteria for each stage so the
  agent carries the work without a human in the loop.
---

# Production Loop (shadow → canary → promote)

Nine stages from *something broke* to *shipped, or rolled back*. The goal is
that the human is needed for **judgment**, not for **choreography**.

Full runbook with commands: [agents-docs/delivery.md](../../../agents-docs/delivery.md).

## When to Use

- A bug is reported against production (issue, `dogfood` finding, broken flow)
- A change is risky enough to want staged exposure
- Asked to roll out, canary, promote, or roll back

## Don't Invoke When

- The change is docs-only or a trivial fix — run `verify` and merge
- Nothing is deployed yet (no production to protect)

## The loop

| # | Stage | Exit criterion | Automated by |
|---|-------|----------------|--------------|
| 1 | **production** | main deployed; sensors green | CI + Netlify |
| 2 | **failure** | a concrete symptom is captured (steps, input, expected vs actual) | reporter / `dogfood` |
| 3 | **reproduce** | an automated failing test that fails for this reason | you, TDD |
| 4 | **candidate fix** | ≥2 candidates considered; one chosen with reasoning | you |
| 5 | **evaluate** | `gate:fast` green | `verify` |
| 6 | **adversarial** | no Blockers/Majors outstanding | `pr-roast` |
| 7 | **shadow** | full E2E matrix green against a preview build | `BASE_URL` + Netlify |
| 8 | **canary** | RC tag published; opt-in consumers green | Release workflow |
| 9 | **promote / rollback** | released & deployed, or reverted | Release workflow |

### Rules that make this work

- **Stage 3 is non-negotiable.** No automated reproduction → no fix. A fix you
  cannot reproduce will not be verifiable as fixed, and "it seems to work" is
  how regressions ship.
- **Never skip 6 because 5 is green.** Gates passing is the precondition for
  roasting, not a substitute for it.
- **One stage at a time.** Do not promote on a red shadow.
- **Rollback must be cheaper than debugging.** If rollback is hard, the canary
  is not a real safety net — say so rather than pretending it is.

## Stage details

### 2–3. Failure → reproduce

```bash
# Rust core: pin the bug with a unit/integration test first
cargo test <module> -- --nocapture
# Browser behaviour: pin it in E2E
npx playwright test --project=chromium e2e/<file>.spec.ts
```

Record the repro in the PR body. A bug report without a repro becomes a
guessing game, and guesswork is what this loop exists to replace.

### 4. Candidate fix

Consider at least two approaches (e.g. minimal patch vs. structural fix) and
record why the chosen one wins. One candidate is not a decision, it is a reflex.

### 7. Shadow

Run the real test suite against a production-shaped build that serves no users.
`playwright.config.ts` already honours `BASE_URL`, so no config change is needed:

```bash
# Local production build
cd web && pnpm run build && pnpm run preview   # serves the built bundle
BASE_URL=http://localhost:4173 npx playwright test --project=chromium
```

For a Netlify **Deploy Preview** (per-PR), point `BASE_URL` at the deploy URL.
This is the highest-value stage here: it is nearly free and catches the class of
bug that only appears in a bundled, optimized build.

### 8. Canary

This is a **static** app with no traffic-splitting infrastructure, so be honest
about what a canary is here:

- Publish an opt-in **RC tag** (e.g. `v0.1.5-rc.1`) with the optimized WASM
  attached, and have a small set of testers/consumers take it explicitly.
- True percentage rollout would need Netlify Split Testing (paid). That is a
  known limitation — do not describe an RC tag as "5% of users".

### 9. Promote / rollback

```bash
# Promote
./scripts/release.sh                                   # preflight: pins + not already released
gh workflow run release.yml -f dry_run=true && gh run watch
gh workflow run release.yml && gh run watch

# Roll back
gh release create vX.Y.Z-rc.1 --notes "rollback"   # or revert the merge on main
git revert -m 1 <merge-sha> && git push origin main # redeploys the prior state
```

Rollback path in detail: [plans/RELEASING.md](../../../plans/RELEASING.md).

## Escalation — when to stop and ask a human

Stop and ask when:

- The repro cannot be automated (you cannot prove the fix)
- Rollback would itself be destructive (data migration, format change)
- A `pr-roast` Blocker is disputed
- The failure is in production and blast radius is unclear

Do **not** ask about: running gates, fixing red CI, resolving threads, arming
auto-merge, or publishing an RC. Those are the loop's job.

## Integration

- `verify` (stage 5) → `pr-roast` (6) → `merge-gate` (9) → `production-loop`
- `dogfood` discovers failures (stage 2)
- `goap-adr-planner` for the architecture decisions this loop surfaces
