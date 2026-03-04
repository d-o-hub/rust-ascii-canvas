# ADR 005: E2E Test Enhancement for ASCII Canvas

## Status
Accepted

## Date
2026-02-26

## Context
The current e2e tests cover basic functionality but miss several key areas:
- Drawing verification for all tools
- Output correctness validation
- Complete button/function coverage

## Decision
We will enhance the e2e tests to:
1. Test all 8 drawing tools (select, rectangle, line, arrow, diamond, text, freehand, eraser)
2. Verify output correctness using `export_ascii()` method
3. Test all UI buttons (undo, redo, copy, clear, zoom controls)
4. Test all keyboard shortcuts including 'B' for border cycle and '?' for modal
5. Test border styles switching
6. Test zoom and pan functionality

## Consequences
- More robust test suite that catches regressions
- Clear documentation of expected behavior for each feature
- Tests serve as executable specification

## Implementation Notes
- Use Playwright's clipboard API to verify copy functionality
- Use canvas bounding box to calculate grid positions for drawing
- Verify ASCII output contains expected characters for each shape type:
  - Rectangle: corners (┌┐└┘) and edges (─│)
  - Line: line characters (─, │)
  - Arrow: arrow characters (→, ←, ↑, ↓)
  - Diamond: diamond characters (◇)
  - Text: the entered text content
  - Freehand: multiple characters along path
