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
| **Resolution** | 2026-09-25 (corrected): a verification swarm proved the earlier entry wrong on two points — the trigger is esbuild's auto-installed optional peer, not the approval workflow, and the "approval persists in the store" claim was false. Superseded by R-03 (re-scoped) and R-05 (prune). |

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

### L-009 — `agents-md` mandated a length its own repo violated (2026-09-27)

| | |
|--|--|
| **Symptom** | `.agents/skills/agents-md/SKILL.md` stated "120 LOC max for AGENTS.md" in one section and "Keep SKILL.md under 300 lines" in another, while root `AGENTS.md` was already 121 lines. The same file also mandated `REFERENCES/` and `TEMPLATES/` (uppercase) when every skill in the repo uses lowercase `references/` and `templates/`. |
| **Root cause** | The skill was written generically and never reconciled against the repo it maintains — a guide that no agent can satisfy teaches agents to ignore guides. |
| **Prevention** | `agents-md` now states both budgets explicitly and separates them (AGENTS.md ≈120 as an *index*; SKILL.md ≤300 with detail in `references/`), lowercases the directory names, and adds a coherence check: changing a threshold requires confirming `quality-gates.sh` and CI agree. |
| **Agent rule** | When maintaining `AGENTS.md` or a skill, verify the claims against the files, not from memory. A guide that contradicts a sensor is a bug in the guide. |
