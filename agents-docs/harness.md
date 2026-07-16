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
| Human acceptance | FB | Human | PR review |

## Timing: keep quality left

```
Agent edit
  → [fast sensors] fmt, clippy, cargo test, web lint/tsc/vitest, architecture, LOC
  → self-correct loop
  → [full sensors] wasm build, size, e2e, audit (pre-PR / CI)
  → human review (inferential)
  → merge → CI re-runs full sensors
  → continuous: dependabot, allowlist debt, periodic dogfood
```

| Tier | When | Command |
|------|------|---------|
| **fast** | After every meaningful change; pre-commit | `npm run gate:fast` or `./scripts/quality-gates.sh --fast` |
| **full** | Before opening PR; CI | `npm run gate:full` or `./scripts/quality-gates.sh` |
| **tools** | UI/tool behaviour changes | `tool-validation` skill + relevant E2E |
| **review** | Before asking human to review | `code-review` skill |

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
| `code-review` | Inferential feedback before human review |
| `tool-validation` | Behaviour harness for 8 tools |
| `dogfood` | Behaviour / UX exploratory sensor |
| `agents-md` | Maintain harness documentation |
| `technical-writing` | Specs and architecture docs |

## Coherence rules

- Guides and sensors must not contradict (e.g. AGENTS checklist must match `quality-gates` tiers and CI).
- Prefer **computational** sensors for anything structural; reserve inferential for semantics and UX.
- Sensor failure messages should tell the agent **how to fix** (see quality-gates output).
- If a sensor never fires, suspect weak detection — not perfect quality.
