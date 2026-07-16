# Plan: Harness Engineering Optimization (2026-07-16)

## Goal

Increase confidence in coding-agent output by systematising **guides** (feedforward) and **sensors** (feedback) per [Harness engineering](https://martinfowler.com/articles/harness-engineering.html).

## Goals (desired state)

| ID | Goal |
|----|------|
| G1 | Agents use tiered verification (fast left, full right) |
| G2 | CI covers Rust **and** web path changes |
| G3 | Architecture layers enforced computationally |
| G4 | Skills map to harness roles (verify, review, plan, implement) |
| G5 | Steering loop documented: repeated failure → improve harness |

## Actions

| # | Action | Status |
|---|--------|--------|
| A1 | ADR-037 | done |
| A2 | `agents-docs/harness.md` + `architecture.md` | done |
| A3 | Rewrite root `AGENTS.md` | done |
| A4 | Tiered `quality-gates.sh` + `check-architecture.sh` | done |
| A5 | CI path filters, web job, size, architecture | done |
| A6 | Skills: `verify`, `code-review` | done |
| A7 | package.json scripts + CONTRIBUTING | done |
| A8 | Dry-run fast gates | done (fixed vitest/config tsc) |

## Non-goals

- Splitting `helpers.rs` (tracked as allowlisted debt)
- Mutation testing / full behaviour harness rewrite
- Installing mandatory local git hooks for all contributors (script provided; optional)
