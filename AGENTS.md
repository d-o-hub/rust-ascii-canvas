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
- [ ] Run TypeScript check: `cd web && npx tsc --noEmit`
- [ ] Run E2E tests: `npx playwright test`
- [ ] Update ADRs for any new architectural decisions in `plans/ADRs/`
- [ ] Document technical findings in `plans/TECHNICAL_ANALYSIS.md`
- [ ] Call `pre_commit_instructions` before submitting.

## Reference Documentation

- [Core Best Practices](agents-docs/best-practices.md) - Task execution, documentation standards, and file organization.
- [Production Readiness Learnings](agents-docs/learnings-archive.md) - Collected technical learnings on Rust, GitHub Actions, Playwright, and TypeScript.
- [Responsive Grid Guide](agents-docs/responsive-grid.md) - Mandatory patterns for handling the responsive WASM grid.

## Proactive Testing

For any code change, attempt to find and run relevant tests. Practice test-driven development when practical. If you encounter failures, diagnose the root cause before attempting environment changes.

---
*Last Updated: 2026-03-19*
