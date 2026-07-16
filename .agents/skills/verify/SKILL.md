---
name: verify
description: >
  Run tiered computational quality sensors and self-correct. Use after code changes,
  before commits/PRs, when asked to "verify", "run gates", "quality check", or when
  AGENTS.md requires verification. Prefer gate:fast during iteration and gate:full
  before handoff.
---

# Verify (Computational Feedback)

Part of the project **harness** ([agents-docs/harness.md](../../../agents-docs/harness.md)). Runs deterministic sensors and drives the agent self-correction loop.

## When to Use

- After implementing or refactoring product code
- Before opening a PR or asking for human review
- When CI failed and you need the local equivalent
- User says verify / gate / quality check / pre-commit

## Don't Invoke When

- Pure documentation-only edits with no scripts/CI change (optional smoke only)
- You only need a single focused unit test mid-TDD (run that test first, then verify)

## Tiers (keep quality left)

| Tier | Command | Use |
|------|---------|-----|
| **fast** | `npm run gate:fast` | Default after edits |
| **full** | `npm run gate:full` | Before PR |
| **architecture only** | `./scripts/check-architecture.sh` | Layer/import changes |
| **focused** | `cargo test …` / `cd web && pnpm test` / one Playwright file | Tight loop |

## Procedure

1. **Choose tier** — fast unless shipping or touching WASM/E2E behaviour → full.
2. **Run sensor**:
   ```bash
   npm run gate:fast
   # or
   npm run gate:full
   ```
3. **On failure** — read `[FAIL]` and `FIX:` lines. Fix root cause. Re-run the same tier.
4. **Do not** disable sensors, add blanket `#[allow]`, skip tests, or expand `.loc-allowlist` without an ADR.
5. **Behaviour changes** to tools — also run `tool-validation` skill / relevant E2E.
6. **Before human review** — run `code-review` skill after full gates are green.

## What fast covers

- rustfmt, clippy `-D warnings`, build, `cargo test`
- architecture layer rules
- LOC (non-allowlisted)
- **ensures `web/pkg` exists** (builds WASM if missing — pkg is gitignored)
- web ESLint, `tsc --noEmit`, Vitest
- privacy / secret scan

## What full adds

- cargo audit / deny (if installed)
- `pnpm run build:wasm` + `check-size` (≤ 1.5MB) if not already built
- Playwright Chromium E2E

## CI parity checks (do not skip)

| CI job | Local equivalent | Gotcha |
|--------|------------------|--------|
| Web tsc | `cd web && pnpm exec tsc --noEmit` | Needs `web/pkg` (gitignored). Clean tree: `npm run build:wasm` first. See harness **L-001**. |
| Architecture | `./scripts/check-architecture.sh` | Needs `rg` |
| WASM size | `npm run check-size` | After `build:wasm` |
| E2E | `npx playwright test --project=chromium` | Needs pkg + dev server |

If CI fails but local gates passed, treat it as a **harness bug**: update sensors so local fails the same way (append to `agents-docs/harness.md` Learned failure modes).

## Steering loop

If you hit the **same** failure class twice in a session (or it recurred from a past PR):

1. Fix product / CI code.
2. Strengthen harness: new test, clearer `AGENTS.md` rule, sensor message, or CI job dependency.
3. Append a short entry under **Learned failure modes** in `agents-docs/harness.md`.
4. Note in `plans/TECHNICAL_ANALYSIS.md` when non-obvious.

## Integration

- **Handoff from** `rust-engineer` / `typescript-expert` → verify
- **Handoff to** `code-review` → human PR
- **Related** `tool-validation`, `dogfood`
