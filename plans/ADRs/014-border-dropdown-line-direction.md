# ADR-014: Border Style Dropdown Fix & Line Direction Options

## Status
Proposed

## Context

Two issues have been identified:

1. **Border Style Dropdown Not Working**: The native `<select>` element for border styles in `web/index.html:65-72` has a `mousedown` handler with `preventDefault()` that may be interfering with the select functionality. The change event listener at `web/main.ts:207-212` should handle style changes but isn't being triggered properly.

2. **Line Tool Direction Options**: The frontend-ui-modernization-plan.md mentions missing horizontal/vertical line direction options. While the Line tool automatically detects direction based on drag (horizontal/vertical/diagonal), users may want explicit control to force a horizontal or vertical line regardless of drag angle.

## Decision

### 1. Border Style Dropdown Fix
- Remove or adjust the `mousedown` handler on the border style select to allow proper dropdown behavior
- Ensure change event fires correctly

### 2. Line Direction Options
Add explicit line direction buttons to the toolbar when Line tool is active:
- **Auto** (default): Current behavior - detect direction from drag
- **Horizontal (━)**: Force horizontal line regardless of drag angle
- **Vertical (│)**: Force vertical line regardless of drag angle

## Implementation Plan

### Phase 1: Border Style Dropdown Fix
1. Modify `web/main.ts` - remove `mousedown` preventDefault on borderStyleSelect
2. Add click handler or ensure change event works
3. Test that selecting a border style updates the editor

### Phase 2: Line Direction Options
1. Add direction state variable to LineTool in Rust
2. Add WASM method to set line direction
3. Add HTML buttons for Auto/Horizontal/Vertical in toolbar
4. Add JavaScript handlers to toggle direction
5. Update LineTool to respect forced direction

## Consequences

### Positive
- Border style selector becomes functional
- Users can draw exact horizontal/vertical lines
- Maintains backward compatibility (Auto is default)

### Negative
- Additional UI complexity in toolbar
- More state to manage in LineTool

## Test Plan
1. Click border style dropdown - should show options
2. Select a style - should apply to next rectangle/line
3. Click Line tool - should show direction buttons
4. Select Horizontal - draw diagonal drag - should get horizontal line
5. Select Vertical - draw diagonal drag - should get vertical line
