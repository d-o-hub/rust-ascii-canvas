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
| **focused** | While iterating on one area | relevant `cargo test …` / `cd web && pnpm test` / single Playwright file |

### What fast includes

- `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test` (native)
- Architecture layer check
- LOC limit (see `.loc-allowlist`)
- Web: `pnpm lint`, `tsc --noEmit`, `pnpm test` (when `web/` present)
- Privacy / secret scan

### What full adds

- WASM build (`npm run build:wasm`) + size budget
- E2E Chromium (`npm run test:e2e` / Playwright)
- `cargo audit` when available

Self-correct on red sensors before asking a human to review. Sensor output includes fix hints — follow them.

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

- Build WASM: `npm run build:wasm` (wasm-bindgen **0.2.121**, see `mise.toml`).
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

## Skills (by harness role)

| Need | Skill |
|------|--------|
| Plan / ADR | `goap-adr-planner` |
| Run sensors / self-correct | `verify` |
| Pre-human review | `code-review` |
| Rust implementation | `rust-engineer`, `rust-best-practices`, `rust-wasm` |
| TypeScript / Vite | `typescript-expert`, `vite` |
| Tool QA | `tool-validation` |
| Exploratory UX | `dogfood` |
| Maintain this doc | `agents-md` |

## Reference docs

- [Harness map](agents-docs/harness.md)
- [Architecture](agents-docs/architecture.md)
- [Best practices](agents-docs/best-practices.md)
- [Production learnings](agents-docs/learnings-archive.md)
- [Responsive grid](agents-docs/responsive-grid.md)
- ADRs: `plans/ADRs/` (see **037-harness-engineering**)

## PR / handoff

- Fast gates green on every push-worthy change; full gates green before review.
- PR template checkboxes must reflect reality.
- Call out harness changes (new sensors, allowlist, CI) explicitly in the PR body.
