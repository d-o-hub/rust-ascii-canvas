# Agent Best Practices - ASCII Canvas Project

This document outlines the core principles and workflows for agents working on this project. Detailed technical references and historical learnings are maintained in the `agents-docs/` directory.

## Core Directives

1. **Analyze** - Understand requirements before acting.
2. **Plan** - Create steps in `plans/` using GOAP and ADR before execution.
3. **Execute** - Run commands and tests autonomously.
4. **Document** - Update `plans/` with findings and results.
5. **Always Verify** - Every change must be followed by a verification tool call.

## Standard Workflow Checklist

- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] Run `cargo test` (unit + integration + doc tests)
- [ ] Build WASM with `npm run build:wasm`
- [ ] Run ESLint check: `cd web && npm run lint`
- [ ] Run E2E tests: `npm run test:e2e`
- [ ] Validate all 8 tools using the **Tool Verification Checklist** below.
- [ ] Update ADRs for any new architectural decisions in `plans/ADRs/`
- [ ] Document technical findings in `plans/TECHNICAL_ANALYSIS.md`
- [ ] Call `pre_commit_instructions` before submitting.

## Reference Documentation

- [Core Best Practices](agents-docs/best-practices.md)
- [Production Readiness Learnings](agents-docs/learnings-archive.md)
- [Phase 1 Learnings](agents-docs/learnings-archive-phase1.md)
- [Responsive Grid Guide](agents-docs/responsive-grid.md)

## Proactive Testing

### Rust Testing Guidelines

1. **Unit Tests**: Place in the same file as the source code, at the bottom, within a `#[cfg(test)] mod tests { ... }` block. This is standard Rust idiomatic practice and allows testing private implementation details.
2. **Integration Tests**: Place in the `tests/` directory to test the crate's public API from the outside.

For any code change, attempt to find and run relevant tests. Practice test-driven development when practical. If you encounter failures, diagnose the root cause before attempting environment changes.

## Tool Verification Checklist

| Tool | Core Requirement |
|------|------------------|
| **Select** | Visible blue highlight; move & delete work. |
| **Rect** | Correct border style corners/lines. |
| **Line** | Continuous lines in all directions. |
| **Arrow** | Visible arrowhead (▲▼◄►) at end. |
| **Diamond** | Uses diagonal characters (╱╲). |
| **Text** | Supports Enter/Backspace/Delete keys. |
| **Free** | Character syncs with Border Style. |
| **Erase** | Radius-based clearing; no out-of-bounds. |
