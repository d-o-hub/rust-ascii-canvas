# Agent Harness Map

Mental model: **Agent = Model + Harness** ([Harness engineering](https://martinfowler.com/articles/harness-engineering.html)).

This document is the inventory of **our outer harness** — everything outside the model that raises first-try success and enables self-correction before human review.

## Goals of the harness

1. Increase probability the agent gets it right on the first attempt (**guides / feedforward**).
2. Let the agent detect and fix issues before humans see them (**sensors / feedback**).
3. Keep humans focused on specification, architecture trade-offs, and behaviour that sensors cannot judge.

## Control types

| | **Feedforward (guides)** | **Feedback (sensors)** |
|--|--------------------------|------------------------|
| **Computational** | clippy/rustfmt config, `deny.toml`, typecheck configs, bootstrap scripts | `scripts/quality-gates.sh`, `scripts/check-architecture.sh`, CI, tests, size budget |
| **Inferential** | `AGENTS.md`, skills, ADRs, architecture docs, plans | `code-review` skill, `dogfood`, `tool-validation`, human review |

## Regulation categories

### Maintainability

| Control | Direction | Type | Location |
|---------|-----------|------|----------|
| Coding conventions | FF | Inferential | `AGENTS.md`, `agents-docs/best-practices.md` |
| Rust idioms | FF | Inferential | `.agents/skills/rust-best-practices`, `rust-engineer` |
| TypeScript standards | FF | Inferential | `.agents/skills/typescript-expert`, ADR-018 |
| fmt / clippy / eslint / tsc | FB | Computational | quality-gates, CI |
| LOC limit (500) | FB | Computational | quality-gates + allowlist |
| Release process | FF | Inferential | `plans/RELEASING.md`, `scripts/release.sh` (preflight) |
| Privacy / secret scan | FB | Computational | quality-gates |
| cargo audit / deny | FB | Computational | CI, quality-gates full |
| Dead-code / debt allowlist | Continuous | Computational | `.loc-allowlist` |

### Architecture fitness

| Control | Direction | Type | Location |
|---------|-----------|------|----------|
| Layer rules | FF | Inferential | `agents-docs/architecture.md` |
| Layer import check | FB | Computational | `scripts/check-architecture.sh` |
| WASM size ≤ 1.5MB | FB | Computational | `npm run check-size`, CI |
| Performance notes | FF | Inferential | ADRs / TECHNICAL_ANALYSIS |
| **Merge contract (ruleset)** | FB | Computational | `main` ruleset, `scripts/ruleset-check.sh`, `.github/ruleset-main.json` |

### Behaviour

| Control | Direction | Type | Location |
|---------|-----------|------|----------|
| Specs / issues / ADRs | FF | Inferential | `plans/`, GitHub issues |
| Rust unit + integration tests | FB | Computational | `cargo test` |
| Vitest (web) | FB | Computational | `cd web && pnpm test` |
| Playwright E2E | FB | Computational | `npm run test:e2e` |
| 8-tool checklist | FB | Inferential | `AGENTS.md`, `tool-validation` skill |
| Dogfood / exploratory QA | FB | Inferential | `dogfood` skill |
| **Merge contract** (checks + threads) | FB | Computational | `scripts/pr-merge-gate.sh`, `main` ruleset |
| **Adversarial PR review** | FB | Inferential | `pr-roast` skill |
| Human acceptance (scope, disputed blockers, rollback) | FB | Human | PR review / ADR |

## Timing: keep quality left

```
Agent edit
  → [fast sensors] fmt, clippy, cargo test, web lint/tsc/vitest, architecture, LOC
  → self-correct loop
  → [full sensors] wasm build, size, e2e, audit (pre-PR / CI)
  → [inferential] code-review, then pr-roast (adversarial)
  → [merge contract] gate:pr → auto-merge when CI is green
  → merge → CI re-runs full sensors
  → [continuous] dependabot, allowlist debt, periodic dogfood
```

| Tier | When | Command |
|------|------|---------|
| **fast** | After every meaningful change; pre-commit | `npm run gate:fast` or `./scripts/quality-gates.sh --fast` |
| **full** | Before opening PR; CI | `npm run gate:full` or `./scripts/quality-gates.sh` |
| **pr** | Before merging; by the agent or CI | `npm run gate:pr` or `./scripts/pr-merge-gate.sh` |
| **pr:test** | After editing the merge gate | `npm run gate:pr:test` |
| **tools** | UI/tool behaviour changes | `tool-validation` skill + relevant E2E |
| **review** | Before merging | `code-review` skill, then `pr-roast` |

## The merge contract

A PR may be merged only when **all** of these hold:

| # | Condition | Local sensor | Server-side (ruleset `main`) |
|---|-----------|--------------|-----------------------------|
| 1 | Not a draft | `pr-merge-gate.sh` | — |
| 2 | No conflicts with base | `pr-merge-gate.sh` | — |
| 3 | Every status check `SUCCESS` | `pr-merge-gate.sh` | required checks `CI Success`, `PR Readiness (merge gate)` |
| 4 | Every review thread resolved | `pr-merge-gate.sh` (reports) | `required_review_thread_resolution: true` |
| 5 | No outstanding `CHANGES_REQUESTED` | `pr-merge-gate.sh` | — |
| 6 | Adversarial pass clean | — (`pr-roast` skill) | — |

Enforcement is split deliberately: the **ruleset** is authoritative (it cannot
be bypassed without admin), and the **script** is the local mirror so the agent
discovers a blocked merge before pushing rather than after.

Two consequences worth remembering:

- **Thread resolution cannot be a status check.** Resolving a thread produces no
  commit, so a `pull_request` job would never re-run and would block on stale
  data. The ruleset evaluates it at merge time; the CI job only *reports*.
- **`CI Success` treats `cancelled` as a failure.** It is a required check, so a
  cancelled job must never read as green. Concurrency cancellation always
  triggers a fresh run, so this cannot deadlock the queue.

`required_approving_review_count` is **0**: CI and thread resolution are the
gates, not a human click (ADR-044). A human is required only for scope, ADRs,
disputed `pr-roast` Blockers, and production rollback.

### The ruleset is repository state, not a file

Everything above about the merge contract lives in GitHub, not in git. That has
three consequences the harness must own:

1. **`git revert` does not undo a ruleset change.** A revert looks clean while
   the gate stays weakened.
2. **A code review never sees it.** The diff of a PR is blind to the one
   setting that decides whether the PR can merge.
3. **Nothing else notices.** CI does not read the ruleset, so drift is silent.

`scripts/ruleset-check.sh` closes all three: `.github/ruleset-main.json` is the
committed canonical projection of the contract, and the `PR Readiness` job
compares it against live GitHub state as a **required** check.

```bash
npm run gate:ruleset              # fail on drift
./scripts/ruleset-check.sh --diff # what changed, snapshot vs live
./scripts/ruleset-check.sh --restore  # put the committed contract back
```

Intentional changes need an ADR *and* an updated snapshot in the same change.
It exits **2** (not 1) when it cannot verify — "unverified" must never read as
"intact". A **missing** ruleset is reported as a Blocker: with no ruleset,
nothing blocks a merge at all.

## Steering loop

When the **same class of failure** appears twice (agent or CI):

1. Fix the immediate bug/test.
2. **Improve the harness in the same effort** when cheap:
   - Add a computational sensor (test, lint rule, architecture check), or
   - Strengthen a guide (`AGENTS.md`, skill, architecture note) with the concrete rule and a self-fix hint.
3. Log the learning in `plans/TECHNICAL_ANALYSIS.md` or a short ADR if architectural.

Do **not** only patch product code and hope the agent remembers next time.

## Skill → harness role

| Skill | Role |
|-------|------|
| `goap-adr-planner` | Feedforward planning / ADRs |
| `rust-engineer` / `rust-best-practices` / `rust-wasm` | Feedforward implementation |
| `typescript-expert` / `vite` | Feedforward web |
| `verify` | Computational feedback loop (run sensors, self-correct) |
| `code-review` | Inferential feedback: is this sound? |
| `pr-roast` | Inferential feedback, adversarial: how do we break this? |
| `merge-gate` | Merge contract: decide and arm auto-merge |
| `production-loop` | Stage-by-stage rollout: shadow → canary → promote |
| `tool-validation` | Behaviour harness for 8 tools |
| `dogfood` | Behaviour / UX exploratory sensor |
| `agents-md` | Maintain harness documentation |
| `technical-writing` | Specs and architecture docs |

**Retired** (2026-09-27, ADR-044): `my-pull-requests` (superseded by `gh`),
`ln-732-cicd-generator` (generated .NET/Python CI and would overwrite this
repo's `ci.yml`), `create-github-pull-request-from-specification` (referenced
tool syntax that does not exist in this harness). Their `skills-lock.json`
entries were removed too — otherwise a sync resurrects them.

## Coherence rules

- Guides and sensors must not contradict (e.g. AGENTS checklist must match `quality-gates` tiers and CI).
- Prefer **computational** sensors for anything structural; reserve inferential for semantics and UX.
- Sensor failure messages should tell the agent **how to fix** (see quality-gates output).
- If a sensor never fires, suspect weak detection — not perfect quality.
- **Local sensors must match CI assumptions** (same inputs, same missing artifacts). A green `gate:fast` that cannot fail the way CI fails is a harness bug — fix the sensor.

## Learned failure modes (steering log)

Append here when the same class of failure hits CI or agents twice (or once with high impact). Each entry must name the **sensor/guide change** that prevents recurrence.

### L-001 — `tsc` without `web/pkg` (PR #128, 2026-07-16)

| | |
|--|--|
| **Symptom** | CI `Web (Lint + Types + Unit)` fails: `Cannot find module './pkg/ascii_canvas.js'` |
| **Root cause** | `web/pkg/` is **gitignored**. Local machines often have a leftover build so `gate:fast` / `tsc` pass; clean CI checkout does not. |
| **Why harness failed** | Web job ran `tsc` in parallel with WASM build and never downloaded the `wasm-pkg` artifact. Local gate did not require pkg presence. |
| **Prevention** | (1) CI `web` **needs** `wasm` + `download-artifact` to `web/pkg` before tsc. (2) `quality-gates.sh` builds WASM if `web/pkg` is missing before typecheck. (3) FIX hints mention gitignored pkg. |
| **Agent rule** | Never assume `./pkg` exists after a clean clone. Before claiming web typecheck green on a machine without pkg, run `npm run build:wasm` or `gate:fast` (which ensures pkg). |

### L-002 — Web-only / harness PRs skipping product sensors (pre-#128)

| | |
|--|--|
| **Symptom** | Frontend-only PRs got no meaningful CI (path filter was Rust-only). |
| **Prevention** | Path filters `rust` / `web` / `product` / `harness`; web + wasm + e2e wired to those outputs. |

### L-003 — `plans/` changes invisible to CI (PR #147, 2026-07-27)

| | |
|--|--|
| **Symptom** | ADR-only PR #147 triggered zero CI jobs — `plans/` was not in any path filter. CI reported green without running any checks. |
| **Root cause** | Path filters in `.github/workflows/ci.yml` omitted `plans/**` from `harness` and `product` categories. |
| **Why harness failed** | No sensor fired on docs-only changes. The architecture and format of ADR files had no automated validation. |
| **Prevention** | (1) Added `plans/**` to `harness` path filter so fmt, clippy, and architecture checks run on ADR changes. (2) AGENTS.md now requires human review for bot-generated ADR PRs. (3) ADR status changes must be verified against actual commits. |
| **Agent rule** | After touching `plans/ADRs/`, run `git log --oneline` to confirm status dates match real commit history. Do not mark ADRs as `Implemented` without corroborating evidence. |

### L-004 — Release guard-rails checked git tags, not GitHub Releases (2026-08-04)

| | |
|--|--|
| **Symptom** | Version bump PRs merged to main but no GitHub Release existed. Guard-rails passed because git tags existed, but actual GitHub Releases were missing. |
| **Root cause** | Guard-rails used `git describe --tags --abbrev=0` to find the "last version". Git tags can exist without a corresponding GitHub Release (e.g., from failed workflow runs or manual tag creation). |
| **Why harness failed** | The release workflow's version validation only checked local git state, not the actual GitHub Release API. |
| **Prevention** | (1) Guard-rails now query `gh release list` to find the latest actual GitHub Release. (2) Version comparison uses whichever is higher: latest release or latest tag. (3) Blocks release if VERSION matches an existing release tag. |
| **Agent rule** | Before creating a release, always verify the latest GitHub Release via `gh release list --limit 1`. Never assume git tags reflect published releases. |

### L-005 — wasm-bindgen dep vs CLI version skew (PR #186, 2026-09-11)

| | |
|--|--|
| **Symptom** | `WASM Build` fails: `rust Wasm file schema version: 0.2.128 / this binary schema version: 0.2.126 — must exactly match` |
| **Root cause** | Dependabot bumps `wasm-bindgen` in `Cargo.toml` alone; repo pins the CLI separately in `mise.toml`, CI wasm job, and `package.json` `netlify:build`. Dep and CLI must move together. Recurred: #173 closed unmerged, #179 merged only because pins aligned, #186 red. |
| **Prevention** | Coordinated bump covering `Cargo.toml` + `mise.toml` + CI wasm-bindgen-cli install + `netlify:build` + `Cargo.lock`, verified by `npm run build:wasm`. Candidate computational sensor: `quality-gates.sh` greps the pins for equality and fails fast with FIX hint. |
| **Agent rule** | Never merge a lone `wasm-bindgen` dep bump. Check `mise.toml`, CI, and `netlify:build` pins match first; if not, close with pointer to coordinated bump. |
| **Resolution** | 2026-09-17: coordinated upgrade to 0.2.128 (ADR-042). `quality-gates.sh` §2b pin-parity sensor now fails fast on skew. |

### L-006 — pnpm v11 `approve-builds` gate blocks web sensors (2026-09-17)

| | |
|--|--|
| **Symptom** | `cd web && pnpm run lint` fails before linting: `[ERR_PNPM_IGNORED_BUILDS] Ignored build scripts: esbuild@0.27.7`, caused by pnpm's build-approval policy (`pnpm approve-builds` is interactive; pnpm 11 no longer auto-approves). Direct `./node_modules/.bin/eslint` passes; `pnpm run lint` does not. |
| **Root cause** | esbuild is in the tree only as an **unused optional peer** of `vite@8.3.0` (nothing in `web/` calls it), auto-installed by an earlier pnpm resolve; its native postinstall then trips pnpm's build-approval gate, and `pnpm run` re-triggers the install check. |
| **Why harness failed** | `quality-gates.sh` web section runs `pnpm run lint`, which inherits the gate. CI may hit the same failure on clean install. |
| **Prevention** | (1) Never rely on manifest-level `pnpm` settings — pnpm 11 ignores them, so a `pnpm.overrides` / approval written there is a silent no-op; (2) single-source the pnpm major (`packageManager` + CI), because CI installs pnpm 10 while the agent shell ran 11.7.0 and the two disagree on where settings live; (3) the durable fix is to stop installing esbuild at all (optional-peer prune, `FOLLOW_UPS.md` R-05), which removes the gate's trigger. |
| **Agent rule** | When `pnpm run` fails with `ERR_PNPM_IGNORED_BUILDS`, first check which pnpm major is running and whether the package is actually needed (`pnpm why <pkg>`). Do **not** approve interactively and do not assume an approval persists in the store (it does not, for this project). Prefer removing the unused dependency; verify behaviour with a direct `./node_modules/.bin/<tool>` run in parallel. |
| **Resolution** | 2026-09-25 (corrected): a verification swarm proved the earlier entry wrong on two points — the trigger is esbuild's auto-installed optional peer, not the approval workflow, and the "approval persists in the store" claim was false. Superseded by R-03 (re-scoped) and R-05 (prune). **2026-09-28: trigger removed** — R-05 pruned the optional peers (esbuild and jsdom are no longer installed on either pnpm major), so `pnpm run lint` no longer trips the gate. See L-012 for the lockfile trap hit while doing it. |

### L-007 — Release dispatched without a version bump (2026-08-08, 2026-09-22)

| | |
|--|--|
| **Symptom** | Release workflow fails in Guard Rails → `Determine version`: `Error: Version 0.1.3 already has a GitHub Release (v0.1.3)`. `Build WASM` / `Create Release` / `Publish WASM` are skipped; nothing is released. Observed twice, 45 days apart (runs `31246709284`, `35771785803`). |
| **Root cause** | `Release` is `workflow_dispatch`-only and evaluates the `VERSION` committed on `main`. Dispatching it right after a code merge — without a release-prep PR that bumps `VERSION`, `Cargo.toml`, `package.json`, and `web/package.json` together — must fail the L-004 guard-rail by design. |
| **Why harness failed** | No guide stated the required order (bump PR → merge → dispatch). The old `scripts/release.sh` *implied* a one-shot release path: it never wrote the version files, never ran the workflow, and pushed tag/`main` directly — bypassing guard-rails entirely. |
| **Prevention** | (1) `plans/RELEASING.md` runbook: bump 4 pins → PR → gates → merge → `dry_run=true` → real dispatch. (2) `scripts/release.sh` repurposed as a **read-only preflight** that validates pin parity + GitHub Release state and prints the exact fix; it no longer tags, pushes, or publishes. (3) Linked from `AGENTS.md` reference docs. |
| **Agent rule** | Never dispatch `Release` against an unreleased `VERSION`. Run `./scripts/release.sh` first: if it reports the version is already released, open/merge a release-prep bump PR, then dispatch with `dry_run=true` before the real run. |

### L-008 — Green CI was never a precondition for merging (2026-09-27)

| | |
|--|--|
| **Symptom** | The `main` ruleset required exactly one status check: `Codacy Static Code Analysis`. Format, Clippy, Architecture, Rust, Security, cargo-deny, WASM, Web and E2E were all **advisory** — a PR could merge with any of them red. Simultaneously `required_review_thread_resolution: false` and `required_approving_review_count: 0`, so unresolved review comments also did not block. `allow_auto_merge` was `true`, so auto-merge would have fired on Codacy alone. |
| **Root cause** | The harness had strong sensors but no **merge gate**. Every prior ADR added a check to CI; none asked whether GitHub was actually blocking on the result. The gap between "CI runs" and "CI gates" was invisible because both looked green. |
| **Why harness failed** | `quality-gates.sh` reported pass/fail for the *working tree*, which is a different question from "may this PR merge?". No control covered PR-level state (checks, threads, reviews), and no ruleset change was ever reviewed. |
| **Prevention** | (1) `scripts/pr-merge-gate.sh` — read-only merge contract, `npm run gate:pr`, with `--self-test` fixtures so the blocking branches are proven offline (no historical PR here ever had threads to test against). (2) Ruleset `main` now requires `CI Success` + `PR Readiness (merge gate)` and sets `required_review_thread_resolution: true`. (3) `CI Success` now treats **cancelled** as failure — it is a required check, so a cancelled job must never read as green. (4) New `PR Readiness` job runs the self-test on every PR so the guard-rail cannot silently rot into a rubber stamp. |
| **Agent rule** | Before merging anything, run `npm run gate:pr`. Never use `gh pr merge --admin` to force past a red check. When a sensor or the ruleset is changed, capture the ruleset JSON first — it is repository state, not a file, so `git revert` does not undo it. |
| **Resolution** | 2026-09-27: ADR-044. Required checks: `Codacy Static Code Analysis`, `CI Success`, `PR Readiness (merge gate)`. Thread resolution required. Approvals remain 0 (gates, not a human click). |

### L-010 — The merge gate was itself untracked repository state (2026-09-27)

| | |
|--|--|
| **Symptom** | ADR-044 hardened the `main` ruleset and then admitted in its own Consequences that "the ruleset is repository state, not a file — `git revert` does not undo it". The fix therefore shipped with **no rollback path**: the pre-mutation JSON existed only in a session temp file, and nothing in the repo recorded what the contract was supposed to be. |
| **Root cause** | Every other control in this harness is a committed file that a sensor can read. The ruleset was the single exception, so it had none of the properties the harness depends on: no diff, no review, no drift detection, no restore. |
| **Why harness failed** | The ADR treated the ruleset as a one-time configuration step rather than as a control needing the same treatment as `quality-gates.sh` or `check-architecture.sh`. A change made through the API is indistinguishable from one made by hand — and nothing detected either. |
| **Prevention** | (1) `.github/ruleset-main.json` — the committed canonical projection of the merge contract, so a change to the real ruleset shows up in a PR diff. (2) `scripts/ruleset-check.sh` (`npm run gate:ruleset`) — drift check with `--diff`, `--update`, `--restore`; wired into the **required** `PR Readiness` job. (3) It exits **2** when it cannot verify, so "unverified" never reads as "intact", and treats a **missing** ruleset as a Blocker rather than a no-op. |
| **Agent rule** | Never mutate the ruleset without (a) capturing current state, (b) writing an ADR, and (c) running `--update` in the same change. If you change a merge guard-rail, `git revert` is not your rollback — `--restore` is. |
| **Verification** | Six drift cases proven by tampering the snapshot: thread-resolution off, required checks gutted, approvals raised, enforcement disabled, linear history off — each detected; clean state and restore both byte-identical to the committed snapshot. |

### L-011 — A feature merged by accident under a `chore(harness):` title (2026-09-27)

| | |
|--|--|
| **Symptom** | The F-13 layer-history implementation (commit `8edf869`, PR #212) landed on `main` as part of #213, whose title is `chore(harness): make CI green a merge precondition`. Verification: **0** product files differ between `8edf869` and `origin/main`. |
| **Root cause** | The agent branched with `git checkout -b chore/harness-merge-automation` while `HEAD` was `feat/f13-layer-undo` at `8edf869` — a commit **not yet on `main`**. The branch was therefore `main + 8edf869 + harness work`, and the PR against `main` carried the feature along. |
| **Why harness failed** | The pre-commit check "no product code staged" was correct and passed every time — the feature was never *staged*. The defect lived one level up, in the branch's commit history, and no control looked there. **A per-commit check cannot see foreign commits that arrived before the commit existed.** |
| **Prevention** | (1) `pr-merge-gate.sh` now reports **which commits a PR would merge**, so foreign commits are visible in the gate output instead of only in `git log`. (2) `AGENTS.md` states the branch rule and the pre-PR check. (3) #212 closed with a full account; `CHANGELOG.md` and ADR-043 record the true attribution. |
| **Agent rule** | **Branch from the base branch, never from another feature branch.** Before opening a PR, run `git log origin/main..HEAD` and confirm every commit listed is one you authored this session. If the list is longer than your work, the branch is wrong — rebase it. Cost of the check: one command. Cost of skipping it: a feature ships mislabeled and unreviewed. |
| **Consequence** | #213's CI ran on the combined content and passed 20 checks including E2E on three browsers, so nothing was *caught* — but the feature was in fact defective: its layer-history commands recorded a **positional index** and replayed it on undo, so a reorder or delete between record and replay made the undo target a different layer. The missing `pr-roast` pass is the likely reason. Fixed in the follow-up that landed stable layer ids. |

### L-009 — `agents-md` mandated a length its own repo violated (2026-09-27)

| | |
|--|--|
| **Symptom** | `.agents/skills/agents-md/SKILL.md` stated "120 LOC max for AGENTS.md" in one section and "Keep SKILL.md under 300 lines" in another, while root `AGENTS.md` was already 121 lines. The same file also mandated `REFERENCES/` and `TEMPLATES/` (uppercase) when every skill in the repo uses lowercase `references/` and `templates/`. |
| **Root cause** | The skill was written generically and never reconciled against the repo it maintains — a guide that no agent can satisfy teaches agents to ignore guides. |
| **Prevention** | `agents-md` now states both budgets explicitly and separates them (AGENTS.md ≈120 as an *index*; SKILL.md ≤300 with detail in `references/`), lowercases the directory names, and adds a coherence check: changing a threshold requires confirming `quality-gates.sh` and CI agree. |
| **Agent rule** | When maintaining `AGENTS.md` or a skill, verify the claims against the files, not from memory. A guide that contradicts a sensor is a bug in the guide. |

### L-012 — Deleting `pnpm-lock.yaml` does not force a fresh resolve (2026-09-28)

| | |
|--|--|
| **Symptom** | Deleting `web/pnpm-lock.yaml` and re-running `pnpm install --lockfile-only` regenerated a lockfile that was **byte-identical** to the old one — old versions (vite 8.3.0), esbuild and jsdom still present. The prune that R-05 depended on appeared not to work. A control resolve of the same `package.json` in an empty scratch dir produced the expected result (vite 8.3.1, no esbuild), which proved the manifest was fine and the repo directory was not. |
| **Root cause** | pnpm keeps a **second** copy of the lockfile at `node_modules/.pnpm/lock.yaml` (the "current" lockfile) and prefers already-installed resolutions from it. It was byte-identical to the committed lock, so pnpm re-resolved nothing. Two traps compounded it: `--lockfile-only` re-created the lockfile from that stale current copy, and a later `rm -rf node_modules` was **not** enough on its own because the `--lockfile-only` run had already re-written `pnpm-lock.yaml`. |
| **Why it nearly misled** | The obvious conclusion — "the prune does not work on pnpm 10" — was wrong. The fix was to delete `node_modules/` **and** `pnpm-lock.yaml` together and then install, which pruned esbuild 91 → 2 references and jsdom 7 → 2 (the survivors are only vite/vitest `peerDependencies` *declarations*, not installed packages). |
| **Prevention** | When a task is "re-resolve dependencies", treat the lockfile as **two** files. Use the scratch-dir control (empty dir + the same `package.json`) to separate "the manifest resolves differently" from "this directory is caching an old answer". Assert on the *installed* tree (`ls node_modules/.pnpm`), not on grep counts in the lockfile — peer declarations survive the prune and make a grep look like a failure. |
| **Agent rule** | `rm pnpm-lock.yaml` is not a fresh resolve. Delete `node_modules/` and the lockfile together, then `pnpm install`. When a resolve "does not take", reproduce it in an empty directory before concluding the tool is at fault. |

### L-013 — The "Security audit" sensor only audits Rust (2026-09-28)

| | |
|--|--|
| **Symptom** Found by the R-05 roast | `quality-gates.sh` prints `[PASS] Audit: OK` — but §"Security audit" runs **`cargo audit` only** (`scripts/quality-gates.sh:387-395`). The PR touched ~250 npm dependency resolutions and the green "Audit: OK" said nothing about any of them. `grep -rn 'pnpm audit\|npm audit' .github/ scripts/` returns **nothing**: the repo has no npm-side advisory sensor at all. |
| **Root cause** | The sensor was added for the Rust half of the tree (ADR-044 era) and its name is ecosystem-neutral, so a web-dependency PR reads as "audited" when only crates were checked. npm advisories were only ever caught reactively, by Dependabot opening an alert (e.g. #11) — a *reporting* channel, not a gate, and it never fails a build. |
| **Why the gates missed it** | Dependabot is configured and working, so the gap is invisible: a new npm advisory produces an alert rather than a red check. The blast radius is exactly the class of change R-05 is — a lockfile refresh, where a new transitive version can carry a vulnerability and no sensor objects. |
| **Prevention** | Add `pnpm audit --audit-level=high` (or `moderate`) for **both** the root and `web/` lockfiles to `quality-gates.sh`, wire it into the `ci-success` aggregator, and rename the existing step so "Audit" cannot be read as covering both ecosystems. Manual check at R-05 time: `pnpm audit` is **clean** on root and `web/`, so this closes a gap rather than fixing a live finding. |
| **Agent rule** | Before trusting a green sensor, confirm which ecosystem it actually inspects. A lockfile PR for `web/` is **not** covered by `cargo audit` — say so in the PR body rather than implying the audit passed. |

### L-014 — `e2e/` was never linted locally; findings surfaced only in Codacy (2026-09-28)

| | |
|--|--|
| **Symptom** | Codacy reported **17** ESLint issues across `e2e/` (object-injection, unsafe-regex, `no-mixed-html`, `no-explicit-any`, `no-unnecessary-condition`). `gate:fast` was green throughout and no local command could reproduce a single one. The prior commit (c4e62c9) dismissed them as "correct test code; suppressing them is the fix" — on the belief that they were stylistic noise. They were not: they were invisible. |
| **Root cause** | The lint entrypoint is `cd web && eslint .` (`web/eslint.config.js`, `tsconfigRootDir: web/`). That config *structurally cannot* see anything outside `web/`, so the 8 spec files + page object were never linted and never type-checked — there was no `e2e/tsconfig.json` either, so `e2e/` sat outside every TypeScript project. The findings were only ever observable through the required Codacy check. |
| **Why the gates missed it** | A green `gate:fast` was read as "linted". The gap is invisible precisely because a sensor *appears* to cover TypeScript. Same class as L-013 (audit covering only Rust): an ecosystem-neutral sensor name covering half the tree. |
| **Prevention** | Added `eslint.config.mjs` at the repo root (ignoring `web/`, so the two configs never overlap) plus `e2e/tsconfig.json`, and wired both into `quality-gates.sh` as `ESLint (root)` and `TypeScript (e2e)`. **And into the `web` CI job**, as `ESLint (root — covers e2e/)` and `TypeScript (e2e)` — see L-016 for why the gate script alone was not enough. Fixes were real, not suppressions: `as HTMLCanvasElement` on a nullable `querySelector` became `querySelector<HTMLCanvasElement>` (which is what made the null-guard "redundant" in the first place), 7 `@ts-ignore` and 5 `(window as any)` casts were replaced by a typed `Window.editor` in `e2e/globals.d.ts` plus a `requireAsciiContent` helper, and two duplicated local helpers were deleted in favour of one in `helpers.ts`. `TOOL_INFO` also moved from `Record<string, …>` to a closed record, so a typo like `'rect'` is now a compile error. |
| **Verified** | Both new sensors were mutation-tested: a deliberately injected `any` and a deliberate type error each turned the corresponding step red (exit 1) and clean again on revert, under **both** pnpm 10 (CI's major) and 11. Note: the mutation pass ran before the L-016 correction (pnpm 11 dev tree, where a stale `node_modules/.bin/tsc` masked the clean-install 254) — the clean-install behaviour above is the verified CI claim. |
| **Lesson for the next sensor** | "The gate is green" is not evidence about code the gate cannot see. When a third-party analyser flags a file the local tooling supposedly covers, check the *scope* of the local command before dismissing the finding — `eslint .` inside `web/` means `web/` only. |
| **Also fixed here** | The 10 Semgrep `third-party-action-not-pinned-to-commit-sha` findings: all third-party actions are now pinned to full SHAs with a `# vX.Y.Z` comment. `dtolnay/rust-toolchain` needed an explicit `toolchain: stable` input, because it infers the toolchain **from the `@ref`** — pinning it to a SHA without that input silently installs no toolchain. `actions/*` are GitHub-owned and out of the rule's scope. |

### L-015 — Annotated tags: `git/ref/tags/<tag>` returns the tag object, not the commit (2026-09-28)

| | |
|--|--|
| **Symptom** | While SHA-pinning actions, 3 of 7 resolved SHAs (`Swatinem/rust-cache`, `EmbarkStudios/cargo-deny-action`, `pnpm/action-setup`) were **invalid commits**. `gh api repos/<r>/git/ref/tags/<tag>` returned a SHA, so the pin looked correct; `gh api repos/<r>/commits/<sha>` then failed with `422 No commit found for SHA`. Shipping those pins would have broken CI at every one of those steps. |
| **Root cause** | For an **annotated** tag, `git/ref/tags/<tag>` resolves to the *tag object*, whose SHA differs from the commit it points at. Lightweight tags return the commit directly, which is why some actions looked fine and others did not — an intermittent-looking failure that is actually deterministic per repo. |
| **Prevention** | Resolve action pins with `gh api repos/<owner>/<repo>/commits/<tag>` (dereferences to the commit) and then **verify** each with `gh api repos/<owner>/<repo>/commits/<sha>` before writing it. A pin that was never round-tripped through `commits/<sha>` is not verified. The same applies to `git ls-remote` output for annotated tags. |
| **Agent rule** | Never trust a SHA because a lookup returned 40 hex characters. Confirm the SHA is a commit (`commits/<sha>` resolves) before pinning CI to it — and remember the tag-vs-commit distinction is per-repository, so verify every pin rather than a sample. |


### L-016 — A sensor added only to `quality-gates.sh` runs in CI *nowhere* (2026-09-28, post-merge roast of #219)

| | |
|--|--|
| **Symptom** | #219 added `ESLint (root)` and `TypeScript (e2e)` to `scripts/quality-gates.sh` and the PR body claimed the e2e/ blind spot was closed. The `pr-roast` pass found **no CI job runs `quality-gates.sh` at all** — `grep 'quality-gates\|gate:fast\|gate:full' .github/workflows/*.yml` returns nothing. The `web` job runs only `cd web && pnpm run lint`. So the two new sensors were **local-only**: a regression in `e2/` would pass every required check and merge. The claim was false, and it was false in exactly the way L-014 was about. |
| **Second defect in the same fix** | Wiring it into CI needs care around *which* `tsc` resolves: `typescript` is not a root dependency (it is aliased to the TypeScript 6 preview in `web/package.json`), so the naive `pnpm exec tsc` at the root only works if a hoisted copy happens to be present. Verified: in a **clean** pnpm-10 install (`pnpm-lock.yaml` + `package.json` only) `pnpm exec tsc --version` fails with `ERR_PNPM_RECURSIVE_EXEC_FIRST_FAIL` (exit 254) — the `typescript@6.0.3` package is in the store (via `typescript-eslint`) but no `tsc` binary is linked at the root. (My dev tree *does* resolve `pnpm exec tsc` → 6.0.3 via a stale `node_modules/.bin/tsc` from an older install — that is exactly the local-vs-CI drift this entry warns about.) The `web` job also never installed **root** dependencies, so the root ESLint had nothing to run against. |
| **Why the gates missed it** | The sensor is real, runs, and is green — locally. `gate:fast`/`gate:full` exercise the script on a developer machine, so "the sensor passes" was mistaken for "the sensor gates". Nothing in the local path can distinguish "wired into CI" from "wired into my shell". |
| **Prevention** | (1) The `web` CI job now also runs `Install root dependencies`, `ESLint (root — covers e2e/)`, and `TypeScript (e2e)`, using **`web/node_modules/.bin/tsc`** rather than `pnpm exec tsc` — one compiler, not two. (2) When adding a sensor, state *where it runs* (script, CI job, or both) and verify by grepping the workflow, not by running the script. (3) Any new CI step must be executed in a clean pnpm-10 tree before it is claimed to work. |
| **Agent rule** | `quality-gates.sh` is a **developer convenience, not a gate**. `AGENTS.md` lists it as a tier, but only what `.github/workflows/ci.yml` runs can block a merge. Adding a check to the script alone is a local-only check — say so, or wire it into the workflow. |


### L-017 — A green required check is compatible with an arbitrary backlog (2026-09-29, Codacy)

| | |
|--|--|
| **Symptom** | `plans/FOLLOW_UPS.md` recorded R-07 as taking the Codacy backlog from 41 open issues to **0 actionable**. It had not. `codacy -o json issues` on 2026-09-29 returned **11 open High findings** (4 × `ESLint8_security_detect-unsafe-regex`, 7 × `ESLint8_security_detect-object-injection`) in `e2e/tools-drawing.spec.ts`, `web/ux.test.ts`, `e2e/pages/EditorPage.ts` and `e2e/canvas.spec.ts`. #222 then merged with the `Codacy Static Code Analysis` check **green**. |
| **Root cause — two independent gaps.** (1) **Codacy's PR analysis is diff-scoped.** It reports only *new* issues in the diff: `codacy -o json pull-request … 220` returned `newIssues: 0` / `isUpToStandards: true` while those 11 existed. A PR-scoped read of a third-party analyser is *structurally* incapable of seeing a pre-existing issue, so "the check is green" said nothing about the backlog. (2) **Neither local ESLint config enabled `eslint-plugin-security` at all** — the rule family Codacy reports in this repo was never run locally, so `gate:fast` and the `web` CI job were green by construction, not by merit. |
| **Why the gates missed it** | Both patterns are enforced by Codacy's **Default coding standard** and cannot be disabled or configured (`codacy patterns gh d-o-hub rust-ascii-canvas ESLint -e -s High` lists both with `enabledBy: Default coding standard`). So there was no config to change, no alert to receive, and no local rule to trip. This is L-013, L-014 and L-016 for the fourth time, one level down: not directories, not ecosystems, but **rule families**. |
| **Prevention** | (1) `eslint-plugin-security` is enabled in **both** ESLint configs — `eslint.config.mjs` for `e2e/`, `web/eslint.config.js` for `web/`. Whole plugin, not just the two rules: each is self-gating, so the Node-only ones cost nothing. Already CI-executing via existing `web`-job steps, so no new wiring. (2) `scripts/codacy-check.sh` reads the **repo-level** list; `npm run codacy:check` and `gate:full` invoke it, labelled *local only — not a CI gate*. (3) `biome.json` turns off four `useQwik*` rules — a Qwik serialisation rule set was judging a repo with no Qwik dependency. (4) The `.codacy.yml` `exclude_paths` rationale stands: an exclusion should only cover code the repo does not own. |
| **Verified** | Every new sensor was **mutation-tested**, because a green sensor is not evidence until it has failed once. `security/detect-*` went red in *both* workspaces on injected sinks (exit 1 each) and clean on revert. `codacy-check.sh` was exercised on all five paths including a shimmed High finding (exit 1), `--level Critical` on the same shim (exit 0), `codacy` off `PATH` (exit 0 + `[WARN] … this is not a pass`), and a forced non-zero inside `gate:full` (`[FAIL]` + fix hint). `biome.json` has **no local oracle** — verified the only way possible: the PR carried a real closure over `page`, the exact shape that had produced the High finding on #222, and Codacy reported `newIssues: 0`. |
| **Bonus defect this found** | Removing the `security/detect-object-injection` sink in `e2e/canvas.spec.ts` made an assertion unconditional — and it **failed**. The test had never evaluated its expectation: `exportAscii()` trims empty borders (`ExportOptions::default().trim_borders`, `src/core/ascii_export.rs:26`), so `if (lines[10])` was false for content at row 10. A test whose failure mode is silence is not a test. Fixed with a known-origin anchor, mutation-tested (a one-column shift now fails it). **A guarded assertion is a skipped assertion.** |
| **Agent rule** | For every required third-party check, ask two questions: (1) *what does its analysis actually cover* — a per-PR, per-diff or per-push scope cannot see a backlog, so read the repository-level list too; (2) *is every rule family it enforces also run by a local sensor* — if not, the local green is vacuous. And: a credential that exists on one machine is not a sensor. `codacy login` stores an **account** token encrypted at `~/.codacy/credentials`; a GitHub Actions runner cannot see it, so CI needs `CODACY_API_TOKEN` / `CODACY_PROJECT_TOKEN` as a repository secret (`gh secret list` is empty here, so no CI wiring exists and none is claimed). |

### L-018 — A one-way subset check is not a guard; make the sensor bidirectional (2026-09-29)

| | |
|--|--|
| **Symptom** | While adding the `security-npm` job (L-013), the merge-gate coherence sensor still printed `[PASS] ci-success aggregates all 9 sensors + changes` — with ten sensors in the workflow. It had not noticed the new job at all. |
| **Root cause** | The sensor held a hand-maintained list, `MERGE_JOBS`, and checked only that it was a **subset** of `ci-success.needs`. That catches *removal* from the aggregator and nothing else. It could not see: a job added to both the workflow and `needs` (so the new sensor had **no guard at all** — deleting it from `needs` later would be invisible); a job aggregated but never added to `MERGE_JOBS`; or a job **renamed** in the workflow while `MERGE_JOBS` kept the old name, which the stale name satisfied. A hand-maintained list plus a one-way comparison is a list that can rot silently — the same shape as L-013 (a sensor covering half a tree) and L-014 (a sensor that could not see a directory). |
| **Prevention** | The check is now **bidirectional and existence-checked**: (a) every `MERGE_JOBS` entry appears in `needs`, (b) every job in `needs` is a known `MERGE_JOBS` entry, (c) every `MERGE_JOBS` entry is a real job key in `ci.yml`. The pass line now states the count it actually verified, so a stale constant is visible in the output. |
| **Verified** | Mutation-tested, all four restored to green afterwards: **M1** drop `security-npm` from `needs` → caught; **M2** aggregate a job absent from `MERGE_JOBS` → caught *(invisible to the old check)*; **M3** rename the job in `ci.yml` only → caught *(invisible to the old check)*; **M4** delete it from `MERGE_JOBS` so it stops being required → caught *(invisible to the old check)*. The parser was itself wrong twice on the way — it first counted the literal `needs:` token as a job name — which is why each mutation was run against the real sensor rather than eyeballed. |
| **Agent rule** | When a sensor keeps its expectations in a list, the check must run **both ways** and confirm the list still names things that exist. "Everything I listed is present" is not "nothing unlisted is there". And a sensor that reports a **count** must derive it, never print a constant — a stale number in a PASS line is worse than no line, because it reads as verification. |

### L-019 — Two hand-rolled copies of one boot sequence, and the copy that never failed (2026-09-29)

| | |
|--|--|
| **Symptom** | The e2e suite could not start without a dev server running first. `npx playwright test` failed with `ERR_CONNECTION_REFUSED` at `http://localhost:3003` on a clean machine, and `quality-gates.sh` had to hand-boot one. The CI e2e job had a **third** requirement hidden in a shell block. |
| **Root cause** | The dev-server boot was implemented twice by hand — a `nohup pnpm run dev` + `curl` poll in the CI e2e job, and a `setsid nohup` + poll + `lsof`/`kill`-by-port teardown in `scripts/quality-gates.sh` — instead of using Playwright's own `webServer`. Duplicated code drifts, and this pair had. **The CI copy could not fail**: `for i in {1..60}; do … if curl …; then break; fi; done` has no `exit 1`, so a server that never came up produced a three-minute Playwright timeout blaming the tests. |
| **Why the gates missed it** | Nothing was broken locally, so nothing went red. The duplicated block was invisible precisely because both copies worked on the path anyone happened to exercise. The tell was a bare `npx playwright test` failing on a machine with nothing running — the assumption that the server is somebody else's problem. |
| **Prevention** | `playwright.config.ts` owns the boot. `webServer` is **omitted when `BASE_URL` is set** (the ADR-044 shadow path), because spawning vite for a run that targets a Netlify preview would waste a server and, with `strictPort` (R-06), fail on a port it does not need. `reuseExistingServer: !process.env.CI` is what makes CI strict: Playwright **throws** on a held port instead of adopting a stranger's server. `e2e/helpers.ts` and `e2e/responsive.spec.ts` now `page.goto('/')` against `use.baseURL`, removing a second hardcoded host literal that had already disagreed with the config. |
| **Verified** | All three config paths read out of the config object, not inferred: local → `reuse=true`; `CI=true` → `reuse=false`; `BASE_URL=…` → `webServer: undefined`. `CI=true` with a server already on 3003 → `Error: http://127.0.0.1:3003 is already used`, **exit 1**. `BASE_URL` set → nothing booted. Bare `npx playwright test` with nothing running → **91/91**, where it previously failed with `ERR_CONNECTION_REFUSED`. No listener on 3003 after a run or after `gate:full` — the old teardown could leak a process group. |
| **Agent rule** | Before hand-rolling "start a server, wait, tear it down", check whether the test runner already does it. And in any readiness poll, the **failure** path must be as explicit as the success path: a loop that breaks on ready and falls through when not is a wait, not a check. |

### L-020 — The LOC sensor did not look at the web half of the repo (2026-09-29)

| | |
|--|--|
| **Symptom** | `gate:fast` printed `[PASS] LOC: no new oversized files` while `web/events.ts` stood at **850** lines and `web/ui.ts` at **540**, both over the 500-line guideline the repo sets for itself. The pass line was not a lie about the files it scanned — it was a lie about the files that existed. |
| **Root cause** | The check ran `find ./src -name '*.rs'`. `web/` was never in scope, so every TypeScript file was invisible to it. A sensor that reports a clean result over an incomplete scope is worse than no sensor: it converts an unmeasured risk into an apparently-measured one. Same shape as L-013 (audit covering crates only) and L-014 (lint covering only `web/`). |
| **Second defect, found while fixing it** | The allowlist was a **permission slip, not a ratchet**. An entry printed `warn "… do not grow; extract modules"` — and nothing enforced *do not grow*. An allowlisted file could grow without limit and the sensor would keep warning politely. Meanwhile there was no way to notice an entry becoming unnecessary, so the list could rot into a list of exemptions. |
| **Prevention** | `scripts/check-loc.sh` now owns the policy, scans `src/**/*.rs` **and** `web/*.ts` (excluding `*.test.ts`), and turns the allowlist into a **ratchet**: an entry is `path` (acknowledge) or `path=lines` (acknowledge *and* freeze — the file may shrink, never grow). Stale entries are reported: an allowlisted file now under the limit, or an entry for a file that no longer exists. The current budgets are the files' present sizes, so shrinking without updating the file is a failure rather than a silently renewed excuse. |
| **Where it runs** | **Both** runners: `scripts/quality-gates.sh` (locally) and a `LOC limits (src + web)` step in the `web` CI job. The check used to be inline in `quality-gates.sh`, which is local-only by construction (L-016) — three jobs now share one implementation, which is the L-019 fix applied up front rather than after the drift. |
| **Third defect: this check existed three times.** Not two. Besides the copy in `quality-gates.sh` and the fact that `web/` was never scanned, the `architecture` CI job carried its **own inline copy** — same loop, own hardcoded `max=500`, own allowlist lookup via `grep -qxF "$rel" .loc-allowlist`. It was invisible to the gate script and to any grep for `quality-gates.sh` (L-016's blind spot in a new place). **It failed on this very PR**: changing the allowlist format to `path=lines` made `grep -x` stop matching, so CI reported `FAIL: src/wasm/helpers.rs has 948 lines (max 500)` for a file that was legitimately allowlisted. The red was *correct* and the duplication was the bug. Removed; the single implementation is `scripts/check-loc.sh`. |
| **Where the CI job lives, and why** | A dedicated `loc` job gated on `rust OR web OR harness`, not a step in `web` (which would skip it for a Rust-only PR) and not in `architecture` (which would skip it for a web-only PR). The check is cross-cutting by construction — `src/**/*.rs` **and** `web/*.ts` — so its gate has to be too. |
| **Verified** | Mutation-tested, each reverted: **M1** grow an allowlisted file by 5 lines → `FAIL … budget 850 (grew by 5)`, exit 1; **M2** a new 600-line non-allowlisted file → `FAIL … (max 500)`, exit 1; **M3** an allowlisted file that drops under the limit → `STALE … drop the entry`, exit 0; **M4** an entry for a deleted file → `STALE … no such file`, exit 0. Clean state: 59 files checked, none over 500. The ratchet logic was itself wrong on the first pass — the lookup grepped for a bare path while the file stores `path=lines` under `-x` — which is why each mutation was run against the real script instead of read. |
| **Agent rule** | A passing sensor is a claim about a **scope**. When you read one, ask what it did *not* look at before you believe it. And an "allowlist" that only warns is not a control: if a file may grow, the entry needs a number. |

### L-021 — The commit-scope sensor read the checkout, not the PR (2026-10-01)

| | |
|--|--|
| **Symptom** | `gate:pr -- 230` printed `Commit scope: 1 commit(s) would merge into main`, then the line `eb3ce8f refactor(web): extract the shared event-result processor (first step of R-09)`, then told me "every commit above ships with this PR". PR #230 ships five `feat(drafts)` commits and has never contained `eb3ce8f` — that is this clone's `HEAD`, reproduced on demand from the dirty working copy. The inverse appears in a CI-shaped clone: shallow checkout, no `origin/main`, `git log origin/main..HEAD` fails → `Commit scope: 0 commits ahead of origin/main (local branch not pushed?)` — wrong in the other direction, and an unpushed *local* branch is not a merge contract fact at all. |
| **Root cause** | `git log --oneline "origin/$BASE..HEAD"` describes **this clone's** HEAD. The gate is routinely run for a PR while sitting on a different branch (the intended workflow: cut each PR from a `/tmp` worktree, read the verdict from the dirty working copy), so the sensor answered about the machine while its wording promised the PR. L-011's guard — "the branch is wrong, rebase it" — was silently reporting the wrong object, which is the one failure mode a guard cannot survive. |
| **Second defect, found while fixing it** | Only `ok`/`fail`/`warn` honoured the "`--json` → stdout is the verdict document" rule written above them. Every bare `echo "  FIX: …"`, indented list line and the commit list went to stdout, so `gate:pr -- --json | jq .` — documented in the merge-gate skill as *safe to pipe* — died with `Invalid numeric literal` on any failing run, and CI's advisory report step was surviving only on its `\|\| cat /tmp/gate.log` fallback. A contract stated in a comment is not enforced; it was enforced nowhere but in the reader's imagination. |
| **Third defect, found by the adversarial pass on this PR** | Two, both in the replacement code — which is why the roast runs *after* the sensors are green. (a) The new fail-closed check tested the field's *type* but not its **elements**: under `set -euo pipefail`, a `commits: [1, 2]` payload (gh schema drift) made `commits_list` die mid-report, so the run exited **without emitting a verdict at all** — fail-closed in the worst direction, silently. (b) The checkout line compared *branch names before OIDs*, so a detached checkout sitting on the PR's exact commit was reported as `Local checkout '…' is not the PR head`, i.e. it claimed the content differed when it did not. Both are the same lesson as the original: a check that answers a nearby question confidently is worse than one that says it cannot answer. |
| **Prevention** | Scope and context are now separate functions, and that split is the fix: **`commit_scope_state`** reads only the PR payload (`gh pr view --json commits,headRefName,headRefOid,baseRefName`) and returns `unreadable / empty / ok`, so it is *structurally incapable* of consulting this clone; **`checkout_state`** is the only thing that touches `git`, takes four scalars, and returns one of six tokens (`head / detached-at-head / stale / other / unknown-head / no-git`) that the report merely prints as **context, never as evidence**. `commits_data_ok` validates element **shape**, not just presence, so a malformed payload is rejected *with* a FIX line instead of aborting the report; an empty/absent/null `commits` fails closed rather than reading as "0 commits". The base branch comes from `.baseRefName`, not an assumed `main`. Both predicates use `printf`, because `echo` is shadowed for the JSON contract — which itself now holds by construction rather than at ~40 call sites. |
| **Where it runs** | **Both** runners. Locally: `npm run gate:pr` (`package.json:43`) and `npm run gate:pr:test` (`:44`); `scripts/quality-gates.sh:217` runs the `--self-test` in the **fast** tier. In CI: `ci.yml:468` runs the same `--self-test` in the `PR Readiness (merge gate)` job, and `ci.yml:492` runs the live `--json` report as an advisory step. |
| **Verified** | **Live A/B** on merged PR #230, run from this clone whose `HEAD` is `eb3ce8f`: old script → `Commit scope: 1 commit(s) would merge into main` + `eb3ce8f refactor(web): extract the shared event-result processor (first step of R-09)` (and from a clone with no `origin/main`: `0 commits ahead of origin/main (local branch not pushed?)`); new script → PR #230's real five commits with subjects and `Local checkout 'fix/merge-gate-commit-scope' is not the PR head — scope above is the PR's (L-021)`. The commit-scope section is the only behavioural difference between the two runs (both exit 1 solely because the PR is already merged → `Mergeability is UNKNOWN`). **Offline fixtures** in `--self-test`: one commit (short+subject), five commits (total + lines), empty array, `commits:null`, `commits` absent, non-object elements, element without `oid`, element without `messageHeadline` (last three all rejected — that is defect (a)); all six `checkout_state` outcomes including `detached-at-head` — defect (b); and three probes of the stdout contract (json keeps stdout clean / json sends the report to stderr / plain still prints). **Mutation, to show the fixtures bite:** the first draft of `commit_scope_state` read `$1` while every sibling predicate reads stdin — six fixtures went red immediately instead of silently passing. Live on this PR: `MERGEABLE — all guard-rails satisfied`, and `--json` stdout parses under `jq -e` while the human report still lands on stderr. `bash -n`, `shellcheck`, `gate:fast`, `gate:full` green. |
| **Agent rule** | Never read the object under test from the machine you are standing on: a ref in the local clone is not the same claim as an identifier fetched for the thing you are judging. And when a pipe is documented as safe, make it *impossible* to break — the rule lived in a comment above `ok/fail/warn` while `echo` walked around it. |
### L-022 — Warning text cannot make an exit-zero audit unverified (2026-09-30)

- **Observed:** the npm audit returned 0 when neither lockfile was checked; prose
  containing `network` could even classify an advisory as a transport failure.
  The WASM size command similarly passed when the artifact did not exist.
- **Prevention:** structured audit counts and exit status must agree, both
  lockfiles must be checked, and missing/unavailable evidence exits nonzero.
  Artifact validity precedes size comparison. Retained positive/negative fixtures
  run through `test-sensors.py`, locally in fast and directly in CI architecture;
  the actual audit/artifact checks run in full and CI security-npm/wasm.
- **Scope:** not a claim of zero vulnerabilities forever. The corrected audit
  immediately found two High advisories in root brace-expansion 5.0.9; a targeted
  locked resolution to 5.0.12 cleared them without ignores or overrides.

### L-023 — Test files and generated bindings are not execution evidence (2026-09-30)

- **Observed:** `tests/wasm/` was not a Cargo target; Node-only CI also disagreed
  with its browser configuration. Existing JS/d.ts files could satisfy local
  checks even when their Rust inputs changed. E2E tested development, not dist.
- **Prevention:** `test-wasm.sh` requires nonzero passing library **and** registered
  integration execution. Input/output fingerprints are checked before consuming
  bindings and travel with the CI artifact. Full/CI E2E build dist and let
  Playwright own a strict production preview; focused runs retain dev startup.
- **Where:** shared runner in full/CI rust; freshness in fast and CI web/e2e;
  production build/E2E in full/CI e2e. Retained build/config fixtures run in fast
  and CI architecture/web. CI path/job/result-map fixtures cover applicability,
  not just the presence of a script name. See ADR-047.

### L-024 — Mixing pnpm majors can make `run` attempt an implicit install (2026-09-30)

- **Observed twice during integration:** a pnpm-10 frozen install followed by
  shell pnpm 11 caused `ERR_PNPM_ABORTED_REMOVE_MODULES_DIR_NO_TTY`; the second
  instance blocked the production build after every fast check passed.
- **Prevention:** root and web `packageManager` pin pnpm 10.34.5, matching each
  CI setup. Retained `test-ci.py` fixtures reject missing/ranged/mismatched pins
  and a drifted CI setup version. The checker runs in fast and CI architecture.
- **Recovery:** explicitly install both workspaces with the pinned manager and
  `CI=true pnpm install --frozen-lockfile`; never delete the lockfile or fall
  back to an unlocked install to clear this error. See CONTRIBUTING.md.

### L-025 — Prose QA and green unit tests missed boundary data loss (2026-09-30)

- **Observed:** typing Space/unknown keys, undoing long strokes or an edited added
  layer, loading during a tool session, and resizing the viewport lost or
  resurrected cells. A 33-layer document could not load its own save.
- **Prevention:** core stroke/document invariants, binding-level key/session
  regressions, and browser viewport/draft/cancellation/PNG regressions now cover
  these boundaries. Core tests run fast/CI rust; WASM tests run full/CI rust;
  browser tests run full/CI e2e. Tool-validation names these exact contracts.
- **Adversarial follow-up:** accepted paste could overlap a provisional stroke,
  disappear on cancel and resurrect that stroke on undo. Both paste paths now
  capture their origin then cancel the stroke before recording undo values;
  retained WASM/browser regressions verify cancel, completion and exact replay.
  No-op strokes preserve redo and consume no history; independence fixtures seed
  real content for both eraser gestures instead of counting an empty erase.
- **Guidance:** assert content **and** history/restoration, not only a tool's
  modified flag. Test failure paths (quota, conflict, rejected replacement) as
  well as successful saves. Local drafts are not a durable backup or a CAS
  protocol; known stale writes are refused and recovery stays explicit.

### L-026 — Codacy's Bandit family had no local sensor (2026-10-02, PR #241 follow-up)

- **Observed:** Codacy runs Bandit across `scripts/*.py`. PR #241 arrived with
  17 new Bandit findings (B603 subprocess-without-shell-equals-true and B404
  import subprocess) that neither `gate:fast` nor `gate:full` reproduced; the
  check landed in `ACTION_REQUIRED` and the only route to a verdict was the
  Codacy CLI. Same shape as L-013 (audit covering only Rust), L-014 (lint
  missing `e2e/`), L-017 (rule families a required check enforces but nothing
  local does) — the ecosystem-level gap is now closed for Python too.
- **Prevention:** `scripts/bandit-check.py` mirrors `npm-audit.py`'s contract
  (0 clean / 1 findings ≥ HIGH / 2 unverified). Missing `bandit` and missing
  `uvx` both fail closed; a non-empty `errors[]` array, a prose payload, an
  unknown severity string, or a findings/status cross-check mismatch are all
  UNVERIFIED rather than pass. Retained fixtures in `test-sensors.py` cover
  clean, HIGH, CRITICAL, sub-threshold advisories, missing tool, scanner
  errors, unknown severity, prose output, both mismatch directions, and a
  stale installed bandit whose `--version` does not match the CI pin.
- **Version drift:** the sensor prefers `uvx --from bandit==1.9.4` (exact
  pin) and, when falling back to an installed `bandit`, verifies its
  `--version` matches before trusting it. A stale 1.7 bandit on a dev PATH
  is a **different sensor** than CI's and cannot report PASS for the
  required check — the same L-005 shape (dep vs CLI skew), applied to the
  harness itself.
- **Where it runs:** **both** runners — `scripts/quality-gates.sh` full tier
  and a `Bandit parity for the required Codacy check` step in the `architecture`
  CI job, immediately after a `python3 -m pip install --break-system-packages
  "bandit==1.9.4"` install step. `check-ci.py` `DIRECT['architecture']`
  requires **both** the install and the sensor invocation, so removing
  either fails coherence (the very first CI run of this PR caught the
  install gap by correctly reporting UNVERIFIED, which is L-016 firing on
  the author). `ci-paths.json` records `scripts/bandit-check.py → architecture`.
- **Threshold:** HIGH + CRITICAL. Codacy's own PR gate blocks in the same
  band (`npm run codacy:check` also fails on Critical/High). LOW/MEDIUM
  findings are printed as an advisory count so a developer sees them without
  the sensor going red on the 33 currently-accepted subprocess/argv patterns
  the sensors structurally require. Triaging those to zero locally would mean
  either 33 `# nosec` annotations (a code change for a non-defect) or a
  baseline file (a second source of truth). Neither buys signal; a *real*
  HIGH/CRITICAL regression is what this sensor exists to catch.
- **Guidance:** when adding a required third-party check, enumerate every
  rule family it runs and ask which of those families have no local sensor.
  A green Codacy PR check on `main` still means nothing for regressions in
  a rule family nothing local invokes — L-017's "green check ≠ clear backlog"
  lesson extended to a new scanner.


### L-027 — GitHub Actions workflows had no local sensor (2026-10-03, PR #243 follow-up)

- **Observed:** The `pr-roast` on PR #243 (the two-commit `release.yml`
  actionlint fix) surfaced a `grep -RIn actionlint scripts/ .github/workflows/
  package.json` returning empty. Nothing local or in CI invokes `actionlint`,
  so today's green state on `ci.yml` / `release.yml` / `issue-closer.yml` can
  silently regress the next time a `needs:` reference, an expression type,
  or a `run:` block changes. Same shape as L-013 (audit covering only Rust),
  L-014 (lint missing `e2e/`), L-017 (rule families a required check enforces
  but nothing local does) and L-026 (Python sensors had no Bandit parity):
  the ecosystem-level gap is now closed for GitHub Actions workflow files.
- **Prevention:** `scripts/actionlint-check.py` mirrors `npm-audit.py` and
  `bandit-check.py`'s contract (0 clean / 1 findings / 2 unverified). Missing
  `actionlint` on PATH, an installed version whose `--version` first line does
  not exactly equal `1.6.26`, a non-array JSON payload, a finding missing any
  of `message`/`filepath`/`line`/`column`/`kind`, a malformed (non-object)
  finding entry, a prose payload, or a findings/exit-code cross-check mismatch
  in either direction are all UNVERIFIED rather than pass. Retained fixtures
  in `test-sensors.py` cover every one of these cases (9 new tests, suite now
  39 total).
- **Threshold:** **all** findings — unlike Bandit, actionlint does not carry a
  severity field that maps cleanly onto Codacy's Critical/High band, and every
  finding class (parser, expression, shellcheck, schedule, property) has
  caused a real regression here (`release.yml:310` expression error shipped
  broken across multiple PRs before #243). A strict "any finding fails" sensor
  is the honest contract; future tuning would need an explicit allowlist
  fixture rather than a severity-band heuristic.
- **Version drift:** the sensor checks `actionlint --version`'s first line
  against `PIN = '1.6.26'` exactly. A dev with `1.7.x` on PATH is a different
  sensor than CI's and cannot report PASS for the required check — same L-005
  shape applied to the harness itself.
- **Where it runs:** **both** runners — `scripts/quality-gates.sh` full tier
  and a `Workflow lint parity (L-027)` step in the `architecture` CI job,
  immediately after an `Install actionlint (pinned)` step that downloads
  `actionlint_1.6.26_linux_amd64.tar.gz` and its `checksums.txt` from the
  release, verifies the SHA-256 with `sha256sum -c`, extracts and installs to
  `/usr/local/bin/actionlint`. `check-ci.py` `DIRECT['architecture']` requires
  the sensor invocation; `ci-paths.json` records
  `scripts/actionlint-check.py → architecture`.
- **Guidance:** when adding a required third-party check, enumerate every
  rule family it runs **and** every file type it analyses, and ask which of
  those has no local sensor. L-017 already said this for rule families; L-027
  is the same lesson applied to *file types*: Rust, JS/TS, Python and now
  YAML/GHA each need a local sensor to reach parity.

