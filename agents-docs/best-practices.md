# Agent Best Practices - ASCII Canvas Project

## Agent system overview

Specialized skills + an explicit **outer harness** (guides + sensors). Start at root [`AGENTS.md`](../AGENTS.md) and [harness.md](harness.md). Document lasting work in `plans/`.

## Best practices

### Task execution
1. **Analyze** — requirements and layers touched ([architecture.md](architecture.md))
2. **Plan** — multi-step work → `plans/` + ADR (`goap-adr-planner`)
3. **Execute** — implement; keep business logic in `src/core/`
4. **Verify** — `npm run gate:fast` (iterate) / `gate:full` (handoff); skill `verify`
5. **Document** — update `plans/` / ADRs; improve harness when failures recur

### Documentation standards
- Architectural decisions → `plans/ADRs/`
- Status → `plans/PROJECT_STATUS.md`
- Technical findings → `plans/TECHNICAL_ANALYSIS.md`
- Harness inventory → `agents-docs/harness.md`

### Testing workflow (tiered)
1. Focused tests while TDD’ing
2. `npm run gate:fast` after meaningful edits
3. `npm run gate:full` before PR (WASM, size, E2E)
4. Tool/UI: `tool-validation` + relevant Playwright

### File organization
```
plans/           # status, ADRs, analysis
agents-docs/     # harness, architecture, learnings
.agents/skills/  # verify, code-review, rust-*, etc.
scripts/         # quality-gates, check-architecture
```

### Code quality
- ≤500 LOC per file (exceptions only in `.loc-allowlist`)
- Computational sensors must stay green; do not skip or weaken
- Public APIs documented; naming consistent with neighbours

### Communication
- Structured summaries; explicit harness changes in PRs
- Prefer FIX hints from quality-gates when self-correcting

### CI/CD (updated 2026-07-16)

1. Path filters: **rust**, **web**, **product**, **harness** — web-only PRs still run web + WASM/E2E as needed
2. Jobs: fmt, clippy, architecture, rust, security, deny, **web** (eslint/tsc/vitest), wasm+size, e2e
3. Release: `gh release create --target ${{ github.sha }}`; `GH_TOKEN: ${{ github.token }}`
4. wasm-opt: install recent binaryen; flags `--enable-sign-ext`, `--enable-nontrapping-float-to-int`, `--enable-simd`, `--enable-bulk-memory`
5. Local mirror of CI left side: `npm run gate:fast`

## Rust testing architecture

1. **Unit tests (`src/`)**: same file, `#[cfg(test)] mod tests`
2. **Integration tests (`tests/`)**: public API only
