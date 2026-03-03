# ADR-029: Documentation Reconciliation

## Status
Proposed

## Date
2026-03-03

## Context

A thorough review of all files in `plans/` reveals significant inconsistencies and contradictions that undermine trust in the documentation:

### Test Count Contradictions
- **PROJECT_STATUS.md** header: 79 unit + 44 integration + 12 E2E
- **PROJECT_STATUS.md** body: "All 124 tests" (but 79+44+12=135, or 79+44+1=124?)
- **TECHNICAL_ANALYSIS.md** header: 79+44+12+1=136
- **TECHNICAL_ANALYSIS.md** CI section: "84 tests (44 Rust + 40 E2E)" — omits unit tests entirely
- **TECHNICAL_ANALYSIS.md** summary: "125 total Rust tests"
- **e2e-test-enhancement-plan.md**: Claims 40 E2E tests pass
- **Action log**: 79+44+2 doc tests

These numbers were written at different times and never reconciled.

### ADR Status Mismatches
4 ADRs are marked "Proposed" but their decisions are fully implemented:
- ADR-001 (shortcuts disabled when tools active)
- ADR-002 (canvas focus management)
- ADR-004 (GitHub repository configuration)
- ADR-005b (package metadata consistency)

### Contradictory Claims
- TECHNICAL_ANALYSIS.md declares "Production Readiness Achieved" with checkmarks
- current-plan.md (same date) shows Phases 2-6 at 0% completion
- PROJECT_STATUS.md says "All Tests Passing" while TECHNICAL_ANALYSIS.md documents 13 unfixed bugs

### Numbering Conflict
Two ADRs use the `005-` prefix, creating ambiguity in cross-references.

### Zoom Range Inconsistency
- PROJECT_STATUS.md: "0.25x to 4x"
- ADR-003, ADR-012, gap-analysis: "0.3x to 4x"

## Decision

### 1. Establish single source of truth for test counts

PROJECT_STATUS.md is the canonical source. It will be updated with verified counts from `cargo test` and `npx playwright test` output. All other documents reference PROJECT_STATUS.md rather than duplicating counts.

### 2. Update ADR statuses

| ADR | Current Status | New Status |
|-----|---------------|------------|
| 001 | Proposed | Accepted (Implemented 2026-02-26) |
| 002 | Proposed | Accepted (Implemented 2026-02-27) |
| 004 | Proposed | Accepted (Implemented 2026-02-26) |
| 005b | Proposed | Accepted (Implemented 2026-02-26) |

### 3. Correct aspirational language

Remove "Production Readiness Achieved" from TECHNICAL_ANALYSIS.md. Replace with actual status reflecting that Phases 2-6 of production readiness are still pending.

### 4. Fix ADR numbering

Rename `005-package-metadata-consistency.md` to `005b-package-metadata-consistency.md` and update all cross-references.

### 5. Verify and fix zoom range

Check actual code in `canvas_renderer.rs` and update all documentation to match the implemented value.

### 6. Add documentation linting checklist

Add to AGENTS.md:
```markdown
### Documentation Consistency Checklist
- [ ] Test counts in PROJECT_STATUS.md match actual `cargo test` output
- [ ] All ADR statuses reflect implementation reality
- [ ] No aspirational language marked as fact
- [ ] Cross-references between documents are accurate
```

## Consequences

### Positive
- Documentation becomes trustworthy again
- New contributors get accurate picture of project state
- ADR statuses reflect reality, enabling proper decision tracking
- Single source of truth prevents divergence

### Negative
- Initial cleanup effort (~4 hours)
- Requires discipline to maintain going forward
- Some documents may need substantial rewriting

### Risks
- Documentation may drift again without automated checks — mitigate with CI lint step (future ADR)
