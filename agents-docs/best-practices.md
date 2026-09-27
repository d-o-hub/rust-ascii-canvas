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
4. `npm run gate:pr` before merging (the merge contract)
5. Tool/UI: `tool-validation` + relevant Playwright

### Review workflow
1. `code-review` — is this sound? (after full gates)
2. `pr-roast` — how do we break this? (adversarial, cited)
3. `merge-gate` — may it merge? then `gh pr merge --auto --squash`
4. `production-loop` — shadow → canary → promote for anything risky

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

### CI/CD (updated 2026-09-27)

1. Path filters: **rust**, **web**, **product**, **harness** — web-only PRs still run web + WASM/E2E as needed
2. Jobs: fmt, clippy, architecture, rust, security, deny, **wasm+size** (before web), **web** (download pkg → eslint/tsc/vitest), e2e
3. **`web/pkg` is gitignored** — never run CI `tsc` without the `wasm-pkg` artifact (harness L-001)
4. Release: `gh release create --target ${{ github.sha }}`; `GH_TOKEN: ${{ github.token }}`
5. wasm-opt: install recent binaryen; flags `--enable-sign-ext`, `--enable-nontrapping-float-to-int`, `--enable-simd`, `--enable-bulk-memory`
6. Local mirror of CI left side: `npm run gate:fast` (auto-builds pkg if missing)
7. Steering: recurring CI fail → append `agents-docs/harness.md` Learned failure modes + harden sensor
8. **Merge contract** (ADR-044 / L-008): ruleset `main` requires `CI Success` + `PR Readiness (merge gate)` and requires review-thread resolution. `CI Success` treats `cancelled` as failure. Local mirror: `npm run gate:pr`

## Rust testing architecture

1. **Unit tests (`src/`)**: same file, `#[cfg(test)] mod tests`
2. **Integration tests (`tests/`)**: public API only
