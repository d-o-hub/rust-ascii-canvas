# PR #22 Regression Plan: Copy/Export Layout Still Breaks in External Editors

Date: 2026-03-25
Status: Proposed plan for implementation

## Goal (GOAP)
Restore reliable copy/export fidelity so ASCII art from the canvas preserves geometry in external editors (notably Notepad/TextEdit plain-text workflows) for PR #22 scenarios.

## Observed Problem
- User reports PR #22 preview still produces "same result" (layout break after copy/paste to external editor).
- Symptom (from provided screenshot): right-side structure is not preserved consistently after paste.

## Current Constraints
- I attempted to validate directly with the provided preview URL using `agent-browser`, as requested.
- Environment limitation prevented browser automation:
  1. `npx -y agent-browser ...` failed with `Chrome not found`.
  2. `npx -y agent-browser install` failed to download Chrome (`Failed to fetch version info` from Google Chrome for Testing endpoint).
- This blocks direct automated repro against the deployed preview in the current environment.

## Hypothesis Tree
1. **Export trimming logic still removes necessary structural spacing in some rows**
   - Potential source: per-line `trim_end_matches(' ')` behavior in export paths.
2. **Selection-region export differs from full-grid export behavior**
   - Potential source: `export_region` and selection copy path diverge from `export_grid`.
3. **Clipboard newline/content normalization mismatch**
   - Potential source: browser clipboard write path differs from internal string generation path.
4. **Monospace dependency not enforced or documented in UX**
   - Even correct text can appear broken in proportional fonts.

## Fix Strategy (Implementation Plan)
1. **Reproduce locally with deterministic unit/integration tests first**
   - Add/extend Rust tests for cases where inner rows are shorter than top/bottom borders.
   - Add tests covering selection-copy export path and region bounds.
2. **Unify export width policy**
   - Compute a stable rightmost non-space column for the exported region and apply it uniformly to all rows.
   - Ensure both `export_grid` (trimmed mode) and `export_region` share the same width algorithm.
3. **Harden clipboard formatting path**
   - Verify line-ending normalization in web clipboard write path and add tests for CRLF conversion.
4. **Add guardrail UX note (if needed)**
   - If rendering differences are font-driven, add explicit "paste in monospace" helper text near copy/export UI.

## Verification Plan
1. Rust unit tests for export and selection copy paths.
2. Web/Vitest tests for clipboard conversion/formatting.
3. Manual verification on preview build using `agent-browser`:
   - Open deploy URL.
   - Draw representative shapes and arrow.
   - Copy output and validate geometry in target editor workflow.

## Agent-Browser Execution Record (Attempted)
- `npx -y agent-browser --session fix22 open https://deploy-preview-22---grand-kheer-862c39.netlify.app && ...`
  - Failed: Chrome not found.
- `npx -y agent-browser install`
  - Failed: could not fetch Chrome version metadata.

## Next Action
Proceed with code/test changes locally (export + clipboard + tests), then re-run `agent-browser` verification once browser runtime becomes available in CI/dev environment.
