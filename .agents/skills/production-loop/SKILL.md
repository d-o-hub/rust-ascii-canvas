---
name: production-loop
description: Run the production failure, reproduction, fix, evaluation, adversarial review and shadow verification loop. Use for production bugs, risky shipping, canary/promote questions or rollback. Production rollback requires explicit human authorization; RC releases are currently unsupported.
---

# Production Loop

Keep humans responsible for judgment, not repetitive verification. The full
runbook is [delivery.md](../../../agents-docs/delivery.md); release mechanics are
in [RELEASING.md](../../../plans/RELEASING.md).

## Stages and truthful exit criteria

| Stage | Exit criterion |
|-------|----------------|
| Production → failure | Concrete symptom, steps, input, expected/actual behavior and affected deployment. |
| Reproduce | Automated test fails for this specific reason. No repro means escalate, not guess. |
| Candidate fix | Consider at least two approaches; record why the chosen one wins. |
| Evaluate | Focused tests and `npm run gate:fast` pass; `npm run gate:full` before handoff. |
| Adversarial | `pr-roast` clean: no unresolved Blockers/Majors. Green gates are not a substitute. |
| Shadow | E2E matrix green against a production-shaped build/Deploy Preview serving no production users. |
| Canary | **Unsupported today.** RC design exists, but the release workflow rejects prerelease versions. |
| Promote | Human-approved scope, clean roast, `merge-gate`, and supported stable release procedure. |
| Rollback | Explicit human authorization, reviewed squash-revert PR, normal merge checks and deployment verification. |

Do not use this delivery loop to bypass `pr-roast` for docs or bot changes. A
small change may need less shadow coverage, but the merge contract still applies.

## Reproduce and evaluate

```bash
cargo test <module> -- --nocapture
npx playwright test --project=chromium e2e/<file>.spec.ts
npm run gate:fast
npm run gate:full
```

Record the failing test and evidence in the PR. Do not weaken assertions to clear
a gate. Use [verify](../verify/SKILL.md) for the actual sensor inventory.

## Shadow

Playwright owns local server startup/readiness/teardown. For an **already-running**
Netlify Deploy Preview, `BASE_URL` disables local startup:

```bash
BASE_URL=https://<deploy-preview-host> npx playwright test --project=chromium --project=firefox --project=webkit
```

Use the production-build command documented by `verify` for a local optimized
bundle. Do not start an orphan dev server, change directories and then run the
root test command from `web/`, or mistake dev-server E2E for production evidence.

## Canary limitation (not an executable release path)

The opt-in RC concept in ADR-044 is a **future design**, not shipped automation.
Both `.github/workflows/release.yml` and `scripts/release.sh` accept only `x.y.z`,
so `v0.1.5-rc.1` cannot pass the current pipeline. There is no supported RC tag,
prerelease publication or RC promotion command. Do not create a manual GitHub
release as a workaround or describe an RC as a percentage rollout. Use shadow
previews now; implementing RC support needs a separate reviewed release-policy
change and fixtures. Track it in [FOLLOW_UPS.md](../../../plans/FOLLOW_UPS.md).

## Promote / rollback

For a stable release, follow the version-bump PR, preflight and dry-run procedure
in `RELEASING.md`; do not dispatch before release authorization and merge checks.

For production rollback, **ask the human first**, even when a revert seems easy.
After authorization, identify the bad **squash commit**, branch from updated main,
run `git revert <squash-sha>` (**no `-m`**), and open a normal PR. Run gates,
`pr-roast` and `merge-gate` before merging; never push a revert directly to main
or force an `--admin` merge. Verify the resulting deployment with the agreed
reproduction. Revert a release through a new patch version, never by moving a tag.

## Escalation

Stop for unclear blast radius, an unautomatable repro, destructive data/format
risk, a disputed roast Blocker, every production rollback decision, or a release
policy/guard-rail change. Routine checks, repairs, comment tracking and a clean
PR's auto-merge choreography do not need a human click.

Integration: `dogfood` → reproduction → `verify` → `pr-roast` → `merge-gate`.
Use `goap-adr-planner` for decisions; this procedure does not itself authorize
publishing a release, changing a ruleset or altering user data.
