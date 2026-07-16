# ADR-037: Harness Engineering for Coding Agents

## Status
Accepted

## Date
2026-07-16

## Context

Coding agents produce non-deterministic output and lack organisational memory. This repo already has partial agent guidance (`AGENTS.md`, skills, quality-gates, CI) but the controls are uneven:

- **Feedforward** (guides) is thin: root `AGENTS.md` is a checklist without architecture constraints, tiered verification, or self-correction rules.
- **Feedback** (sensors) is incomplete on the left and in CI: web ESLint/Vitest/tsc and WASM size are not in CI; path filters ignore `web/**` so frontend-only PRs get no checks.
- **No architecture fitness function**: layered `core → render/ui → wasm` is documented in comments but not enforced.
- **No explicit steering loop**: recurring agent failures are not systematically promoted into guides/sensors.
- Quality gates are all-or-nothing rather than **keep quality left** (fast local sensors vs expensive post-integration).

We adopt the mental model from [Harness engineering for coding agent users](https://martinfowler.com/articles/harness-engineering.html) (Birgitta Böckeler / martinfowler.com): **Agent = Model + Harness**, with guides + sensors that are either computational or inferential.

## Decision

1. **Treat the outer harness as a first-class system** documented in `agents-docs/harness.md` and driven from root `AGENTS.md`.
2. **Organise controls by direction and execution type**:
   - Feedforward guides: `AGENTS.md`, `agents-docs/architecture.md`, skills, ADRs
   - Feedback sensors: `scripts/quality-gates.sh` (tiered), `scripts/check-architecture.sh`, CI, review skills
3. **Keep quality left** with tiers:
   - **fast** (pre-commit / agent loop): fmt, clippy, unit tests, web lint/typecheck/vitest, architecture, LOC, privacy
   - **full** (pre-PR / CI): + WASM build, size budget, cargo audit/deny, E2E
4. **Fix CI path filters** so `web/**`, `e2e/**`, scripts, and agent docs trigger the right jobs; add web quality and size sensors.
5. **Architecture fitness**: computational check that `core` does not import `wasm`/`render`/`ui`; oversized files only via explicit allowlist.
6. **Steering loop**: when the same failure happens twice, update a guide or sensor in the same change (or a follow-up PR), not only the product code.
7. **Inferential sensors**: `code-review` and `verify` skills with LLM-oriented failure messages (positive “prompt injection” for self-correction).

## Consequences

### Easier
- Agents self-correct more often before human review
- Frontend-only changes are validated in CI
- Clear language for maintainability vs architecture vs behaviour harnesses
- Faster agent loops use `gate:fast` instead of full E2E every time

### Harder / trade-offs
- Harness files themselves need maintenance when conventions change
- Known debt (`src/wasm/helpers.rs` LOC) must stay allowlisted until split
- More CI jobs increase matrix complexity (mitigated by path filters + concurrency)

## Related

- `agents-docs/harness.md` — operational map of guides and sensors
- `agents-docs/architecture.md` — architecture feedforward
- ADR-024 (tests), ADR-018 (TS standards), ADR-021 (production readiness)
