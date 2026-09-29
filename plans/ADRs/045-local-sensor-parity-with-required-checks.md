# ADR-045: Local Sensor Parity with Required Checks

## Status
Accepted — 2026-09-29. Extends ADR-044 (merge contract) and ADR-037 (harness engineering). Records harness **L-017**.

## Context

ADR-044 made `Codacy Static Code Analysis` a required check, so a non-`SUCCESS` Codacy state blocks a merge. That made Codacy load-bearing for correctness, and it exposed how little the rest of the harness knew about what Codacy does.

Two facts, both established by reading the API on 2026-09-29 rather than by a failing gate:

1. **Codacy's pull-request analysis is diff-scoped.** It reports only *new* issues in the diff. `codacy -o json pull-request gh d-o-hub rust-ascii-canvas 220` returned `newIssues: 0` and `isUpToStandards: true` on a PR whose repository still carried 11 open High findings. So a green required check was **structurally compatible with an arbitrary pre-existing backlog** — the check could not have told us otherwise. `plans/FOLLOW_UPS.md` recorded "41 open issues → 0 actionable" on exactly that evidence and was wrong.
2. **The rule family Codacy reports here was never run locally.** `eslint.config.mjs` and `web/eslint.config.js` did not enable `eslint-plugin-security` at all. `gate:fast` and the `web` CI job were green because the rules were absent, not because the code was clean. Both rules that fire — `security_detect-unsafe-regex` and `security_detect-object-injection` — are enabled by Codacy's **Default coding standard**, verified with `codacy patterns gh d-o-hub rust-ascii-canvas ESLint -e -s High`, and cannot be disabled or configured. There was no config lever, no alert, and no local rule to trip.

This is the fourth instance of one shape. L-013: a `Security audit` sensor that audits crates only. L-014: a lint sensor whose entrypoint structurally cannot see `e2e/`. L-016: sensors added to `quality-gates.sh`, which no CI job runs. Each was "the sensor exists, is green, and does not cover the thing". The generalisation is about **coverage claims**, not about any one tool.

A third failure sat underneath both, and is the reason the previous fix stalled. `codacy login` stores an **account** API token on the developer's machine, encrypted at `~/.codacy/credentials`. The CLI therefore works locally, which reads as "the sensor is wired". A GitHub Actions runner cannot see it — runners read repository secrets and OIDC only — and `gh secret list` is empty here. "The CLI is authenticated" and "CI can reach Codacy" are unrelated facts, and the `codacy` skill documented them as interchangeable alternatives.

## Decision

1. **A green required check is not evidence about its backlog.** Any required third-party check must be read at *both* scopes: the per-PR view and the repository-level view. `scripts/codacy-check.sh` wraps the repository-level read as `npm run codacy:check`, and `gate:full` invokes it.
2. **That script is an agent procedure, not a gate, and says so.** It is not in `ci.yml`, because no credential a runner can use exists yet. `quality-gates.sh` labels it "local only — not a CI gate". Labelling it anything stronger would reproduce the failure it exists to close. What blocks a merge remains the required check.
3. **It never prints a pass it did not earn.** No CLI, no auth or no output produces `[WARN] … NOT checked (this is not a pass)` and exit 0. Absence of a check is reported as absence. JSON is shaped with `python`, never grepped — grepping JSON for a severity is how a sensor starts lying.
4. **Every rule family a required check enforces must also run in a local sensor.** `eslint-plugin-security` is enabled in both ESLint configs. The whole plugin, not only the two rules Codacy fires on, because each is self-gating: `detect-child-process` cannot fire unless `child_process` is called, and this is browser code plus a Node test harness. The existing `web`-job steps already execute both configs, so there is no new CI wiring and no new local-only gap.
5. **Narrowing an analyser is legitimate; removing a required check is a human decision.** Codacy's Biome tool was running four `useQwik*` patterns — Qwik's `$()` serialisation boundary — against a repository with no Qwik dependency, and produced a High finding on a plain arrow function. `biome.json` turns them off. The pattern is locked to the Default coding standard and `codacy tool --help` exposes only `enable | disable | configuration-file`, so a repo config file was the only available lever. Security rules stay **on**: they agree with `eslint-plugin-security`, so a finding from either analyser is real. Removing `Codacy Static Code Analysis` from the ruleset is never a route to a green check.
6. **A third-party fix is verified by a push.** Codacy re-analyses only the new head, so a Codacy fix is the one change the harness cannot self-verify. One finding per commit, never bundled with unrelated work, and the non-disableable cases change the **code**: no suppression, no weakened assertion.
7. **Credentials are documented where the confusion lives.** The `codacy` skill gained G1–G4 (locality, read vs write, project scoping, procedure-≠-sensor). An account token must never be added to CI; the CLI refuses to fall back to one for scoped work, and that refusal is the tool reporting the scope was wrong.

## Consequences

- `gate:fast`/`gate:full` and the `web` CI job can now see the class of finding that was previously visible only on a required third-party check. Mutation-tested in both workspaces: injected sinks turn `security/detect-*` red (exit 1) and clean on revert. A green sensor that has never failed is not evidence.
- The repo-level backlog is now checkable before a PR is opened rather than only after it merges. Current state: **0 open issues**, from Codacy's own re-analysis of `main` after #222.
- **PR 6 (Codacy coverage) stays held.** It needs `CODACY_PROJECT_TOKEN`, which does not exist. It is recorded as blocked-pending-secret rather than written as if it works. `codacy info` reports `isAdmin: false` for the org, so token creation may need the org owner.
- `biome.json` has **no local oracle** — Biome is not installed and must not become a dependency of a project that does not use it. It was verified the only way available: the diff carried a real closure over `page`, the exact shape that had produced the High finding on #222, and Codacy reported `newIssues: 0`. Any future edit to that file is verified the same way, or not at all.
- Codacy's three quality goals remain unmet and are **not** in scope here: coverage `None` (goal ≥ 60), complex files 18 % (goal ≤ 10), duplication 20 % (goal ≤ 10). Coverage needs the token; the other two are the LOC and module-size work.
- Six skills are upstream-synced (`skills-lock.json`: `codacy-cloud-cli`, `codacy-analysis-cli`, `codacy-code-review`, `configure-codacy`, `configure-codacy-cloud`, `setup-coverage`) and must not be edited locally — a sync overwrites the edit. The `codacy` **policy** skill is not in the lock file and is where this repo's rules live. The auth-locality gap originates upstream (`codacy/codacy-skills` README presents `CODACY_API_TOKEN` and `codacy login` as bare alternatives) and is reported there rather than patched locally.

## Alternatives rejected

- **Drop Codacy from the required checks** and rely on local sensors. This is exactly what the `codacy` skill permits *if* local sensors supersede the analyser — and the premise was false, since the rule family was not running locally. It is also a human decision, not an agent's.
- **Suppress the findings in Codacy's configuration.** Not available: both patterns are locked to the Default coding standard, and the CLI answers *"Pattern enforced by Default coding standard, can't be modified."*
- **Add an ESLint disable comment** per finding. Hides the finding locally, leaves Codacy red, and is the same "make the linter quiet" move that produced the vacuous assertion documented in L-017.
- **Wire `codacy-check.sh` into CI now** and let it fail on a missing secret. A permanently red or silently skipped required-adjacent step is worse than an honestly labelled local procedure: it looks like a control and is not one.
- **A scheduled CI intake job with no token.** Same objection, with a schedule that reads as coverage it does not have.

## References

- harness **L-017** (`agents-docs/harness.md`) — the steering entry, with symptom, root cause, prevention and mutation-test evidence
- **L-013**, **L-014**, **L-016** — the three earlier instances of the same shape
- ADR-037 (harness engineering), ADR-044 (merge automation and the delivery loop)
- #219, #220 — the `e2e/` scope gap and its L-016 correction
- #222 — the 11 findings, the vacuous assertion, and the Biome false positive
