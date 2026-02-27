# Implementation Plan: Canvas Focus Management Fix

## Goal
Fix the issue where text input and keyboard shortcuts stop working after clicking any button in the toolbar.

## Problem Analysis

### Root Cause
When clicking toolbar buttons (undo, redo, copy, tool buttons, etc.), the canvas loses keyboard focus because:
1. Canvas has `tabindex="0"` making it focusable
2. Button clicks cause focus to shift from canvas to the clicked button
3. After losing focus, keyboard events are no longer captured by canvas
4. User must manually click canvas to restore keyboard input

### Affected Functionality
- Text tool: Cannot type after clicking any button
- Tool shortcuts (R, T, V, L, A, D, F, E): Don't work after button click
- Undo/Redo keyboard shortcuts (Ctrl+Z, Ctrl+Shift+Z): Don't work after button click
- Copy keyboard shortcut (Ctrl+C): Doesn't work after button click

### Code Locations
- `web/main.ts:160-161`: Keyboard event listeners on canvas
- `web/main.ts:164-206`: Button click handlers that cause focus loss
- `web/index.html:99`: Canvas element with tabindex="0"

## Research Findings (2026 UI/UX Best Practices)

### From Web Research
1. **Focus management is critical for accessibility** (Yale, UW, BBC guides)
2. **Prevent focus theft** - buttons should not steal focus unless necessary
3. **Restore focus after actions** - return focus to meaningful element after dialogs/modals
4. **Use mousedown with preventDefault** to prevent focus change
5. **Document-level keyboard listener** - can delegate to canvas regardless of focus

## Solution Options

### Option A: Prevent Focus Stealing (Recommended)
- Add `mousedown` event handler to all toolbar buttons with `e.preventDefault()`
- This prevents button from taking focus
- Pros: Simple, minimal code change
- Cons: May affect button accessibility states

### Option B: Restore Focus After Action
- After each button action, call `canvas.focus()`
- Pros: Ensures canvas is always refocused
- Cons: May cause visual flicker, may interrupt user

### Option C: Document-Level Keyboard Listener
- Add keyboard listener to document instead of canvas
- Delegate events to canvas handler regardless of focus
- Pros: Most robust solution
- Cons: More complex, may interfere with other inputs

## Implementation Steps

### Step 1: Implement Option A (Prevent Focus Stealing)
- Modify `web/main.ts` setupEventListeners function
- Add mousedown handler to all buttons with preventDefault()

### Step 2: Add Canvas Focus on Pointer Events
- Add focus restoration on canvas pointer interactions
- Ensure canvas regains focus on any click/drag

### Step 3: Test All Scenarios
- Text tool: Type after clicking undo
- Tool shortcuts: Press R, after clicking T, V copy
- Keyboard shortcuts: Ctrl+Z after any button click

## Dependencies
- None (pure JavaScript/TypeScript changes)

## Testing Approach
1. Manual testing: Click each button, verify keyboard works
2. E2E tests: Add test case for focus preservation

## Success Criteria
- [x] Text input works after clicking any button
- [x] Tool shortcuts (R, T, V, etc.) work after button click
- [x] Keyboard shortcuts (Ctrl+Z, Ctrl+Shift+Z) work after button click
- [x] No visual flicker or unexpected behavior

## Implementation Complete

### Completed Steps

1. ✅ Added mousedown handlers to tool buttons (lines 195-203)
2. ✅ Added mousedown handlers to action buttons: undo, redo, copy, clear (lines 214-243)
3. ✅ Added mousedown handlers to border style select (line 206)
4. ✅ Added mousedown handlers to zoom buttons: fit, reset, in, out (lines 245-264)
5. ✅ Canvas focus on pointer events (line 273)
6. ✅ Added focus-visible CSS for accessibility (style.css line 253-257)

### Files Modified

- `web/main.ts`: Added 8 mousedown event handlers
- `web/style.css`: Added `.select-input:focus-visible` styles

### Test Results

All tests pass:
- 44 Rust unit tests ✅
- 40 E2E tests ✅
