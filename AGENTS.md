# Agent Best Practices — ASCII Canvas

Outer harness for coding agents. Full map: [agents-docs/harness.md](agents-docs/harness.md). Architecture constraints: [agents-docs/architecture.md](agents-docs/architecture.md).

## Core directives

1. **Analyze** — understand requirements and which layers you will touch.
2. **Plan** — multi-step or architectural work → `plans/` + ADR via `goap-adr-planner`.
3. **Execute** — implement; keep business logic in `src/core/`.
4. **Verify** — run **fast** sensors after changes; **full** before PR. Use the `verify` skill.
5. **Document** — update `plans/` / ADRs when decisions or learnings change.
6. **Steer the harness** — if the same failure happens twice, improve a guide or sensor, not only product code.

## Keep quality left (mandatory tiers)

Do **not** run the full E2E suite after every one-line fix. Use tiers:

| Tier | When | Command |
|------|------|---------|
| **fast** | After every meaningful edit | `npm run gate:fast` |
| **full** | Before commit/PR that touches product code | `npm run gate:full` |
| **pr** | Before merging anything | `npm run gate:pr` |
| **focused** | While iterating on one area | relevant `cargo test …` / `cd web && pnpm test` / single Playwright file |

### What fast includes

- `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test` (native)
- Architecture layer check
- LOC limit (see `.loc-allowlist`)
- **WASM pkg for typecheck**: `web/pkg` is gitignored; gate builds it if missing (CI downloads the wasm artifact before `tsc`)
- Web: `pnpm lint`, `tsc --noEmit`, `pnpm test` (when `web/` present)
- Privacy / secret scan

### CI vs local (learned)

If CI is red and local is green, **do not** only re-run CI. Diff assumptions (artifacts, path filters, env). Fix the **sensor** so local fails the same way next time — see [harness learned failures](agents-docs/harness.md#learned-failure-modes-steering-log) (e.g. L-001 `web/pkg`).

### What full adds

- WASM build (`npm run build:wasm`) + size budget
- E2E Chromium (`npm run test:e2e` / Playwright)
- `cargo audit` when available

Self-correct on red sensors before asking a human to review. Sensor output includes fix hints — follow them.

`gate:pr` is the **only** thing that answers "may this merge?" — it is read-only
and never merges.

## Architecture constraints (non-negotiable)

```
core (pure) ← render, ui ← wasm ← web/
```

- `src/core/**` must **not** import `wasm`, `render`, or `ui`.
- Prefer extracting modules over growing files past **500 LOC**.
- Details and allowed edges: [agents-docs/architecture.md](agents-docs/architecture.md).
- Computational check: `./scripts/check-architecture.sh`.

## Rust testing

1. **Unit tests**: same file, `#[cfg(test)] mod tests { … }` (can touch private items).
2. **Integration tests**: `tests/` for public API only.
3. On failure: fix root cause; do not weaken assertions or skip CI to “make it green”.

## Web / WASM

- Build WASM: `npm run build:wasm` (wasm-bindgen **0.2.128**, see `mise.toml`).
- Web lint/test: `cd web && pnpm lint && pnpm exec tsc --noEmit && pnpm test`.
- E2E: Playwright from repo root; prefer `--project=chromium` locally.

## Tool behaviour checklist (behaviour harness)

When changing drawing tools or canvas interaction, validate:

| Tool | Core requirement |
|------|------------------|
| **Select** | Visible blue highlight; move & delete work |
| **Rect** | Correct border style corners/lines |
| **Line** | Continuous lines in all directions |
| **Arrow** | Visible arrowhead (▲▼◄►) at end |
| **Diamond** | Diagonal characters (╱╲) |
| **Text** | Enter / Backspace / Delete |
| **Free** | Character syncs with Border Style |
| **Erase** | Radius-based clear; no OOB |

Use the `tool-validation` skill for the full procedure.

## Merge & ship (less human in the loop)

A PR may be merged when **all** of these hold — nothing else:

| # | Condition | Enforced locally | Enforced by GitHub |
|---|-----------|------------------|--------------------|
| 1 | Not a draft, no conflicts | `npm run gate:pr` | — |
| 2 | **Every** CI check `SUCCESS` | `npm run gate:pr` | ruleset: `CI Success`, `PR Readiness (merge gate)` |
| 3 | **Every** review thread resolved | `npm run gate:pr` | ruleset: `required_review_thread_resolution` |
| 4 | No outstanding `CHANGES_REQUESTED` | `npm run gate:pr` | — |
| 5 | Adversarial pass clean | `pr-roast` | — |

```bash
npm run gate:fast          # while iterating
npm run gate:full          # before the PR is mergeable
npm run gate:pr            # the merge contract (read-only, never merges)
gh pr merge <PR> --auto --squash   # CI performs the merge when it goes green
```

**Do not** use `--admin` to force past a red check, and do not treat
"could not verify" as "verified" — `pr-merge-gate.sh` errors rather than
assumes. Squash is required: the repo enforces linear history.

### The delivery loop

```
production → failure → reproduce → candidate fix → evaluate
           → adversarial → shadow → canary → promote / rollback
```

| Stage | This repo |
|---|---|
| **reproduce** | an automated failing test — no repro, no fix |
| **evaluate** | `gate:fast` green |
| **adversarial** | `pr-roast` skill, cited against official docs |
| **shadow** | full E2E matrix against a build/preview (`BASE_URL`) |
| **canary** | opt-in RC tag — this is a static app, so no true % split |
| **promote / rollback** | Release workflow; revert the merge to roll back |

Runbook: [agents-docs/delivery.md](agents-docs/delivery.md) · skill `production-loop`.

**Human judgment stays** for scope/spec, ADRs, disputed roast blockers, and
production rollback. **Human chore is gone** for gates, self-correction, the
roast, comment tracking, shadow E2E, and clicking merge.

## Skills (by harness role)

| Need | Skill |
|------|--------|
| Plan / ADR | `goap-adr-planner` |
| Run sensors / self-correct | `verify` |
| Semantic review | `code-review` |
| Adversarial review ("roast") | `pr-roast` |
| Decide + perform the merge | `merge-gate` |
| Ship safely (shadow → canary → promote) | `production-loop` |
| Rust implementation | `rust-engineer`, `rust-best-practices`, `rust-wasm` |
| TypeScript / Vite | `typescript-expert`, `vite` |
| Tool QA | `tool-validation` |
| Exploratory UX | `dogfood` |
| Maintain this doc | `agents-md` |

## Reference docs

- [Harness map](agents-docs/harness.md)
- [Delivery loop runbook](agents-docs/delivery.md)
- [Architecture](agents-docs/architecture.md)
- [Best practices](agents-docs/best-practices.md)
- [Release runbook](plans/RELEASING.md)
- [Production learnings](agents-docs/learnings-archive.md)
- [Responsive grid](agents-docs/responsive-grid.md)
- ADRs: `plans/ADRs/` (see **037-harness-engineering**, **044-merge-automation-and-delivery-loop**)

## Bot-generated PRs

- Automated PRs (e.g., from Jules, Dependabot, or other bots) are held to the
  **same** merge contract: the gates decide, not the bot.
- A human is **not** required to click merge (ADR-044). A human is required when
  a `pr-roast` **Blocker** is disputed, or when the diff touches a merge
  guard-rail, a release pin, or the document format.
- ADR-only PRs must have status changes verified against actual commit history — not just PROJECT_STATUS.md.
- ADR `## Status` changes must preserve cross-references and implementation scope notes.
- `plans/` changes trigger harness CI checks (fmt, architecture, lint).
- Learn observed bot failures in `agents-docs/harness.md` (steering log).

## PR / handoff

- Fast gates green on every push-worthy change; full gates green before review.
- PR template checkboxes must reflect reality.
- Call out harness changes (new sensors, allowlist, CI) explicitly in the PR body.
- Never bypass a red check with `--admin`; fix the cause or ask a human.
