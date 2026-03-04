# ADR-016: Text Tool Click-to-Insert Position Bug

## Status
Proposed

## Context
The text tool is not inserting characters at the clicked cursor/grid position. When users click on the canvas and type, the text doesn't appear at the clicked position or stops working after a few insertions.

## Analysis

### Issue 1: Text Not Appearing at Clicked Position
- When clicking on canvas, onPointerDown is called with screen coordinates
- These are converted to grid coordinates via screen_to_grid
- The cursor position is set in TextTool

### Issue 2: Text Stops Working
- The is_active() check might be failing after first insertion
- The tool might be getting reset incorrectly

### Issue 3: Focus Issues
- Canvas might lose focus after pointer events
- Keyboard events might not reach the canvas

## Root Causes Identified

1. **Missing canvas focus before typing**: Tests need to ensure canvas is focused before typing
2. **TextTool state not persisting**: The cursor might be getting cleared incorrectly
3. **Preview not rendering**: Text isn't shown until committed

## Decision

### Fix 1: Ensure Canvas Focus
Add explicit canvas focus check before keyboard events in tests.

### Fix 2: Fix TextTool State Management
Ensure cursor and buffer state persists correctly between keystrokes.

### Fix 3: Add Comprehensive E2E Tests
Create tests that verify:
- Text appears at exact clicked position
- Multiple characters insert sequentially
- Backspace works at any position
- Click in new location starts fresh
- Works with zoom levels

## Implementation Plan

1. Fix any state management issues in TextTool
2. Add detailed E2E tests with position verification
3. Add test for zoom + text combination
4. Add test for multi-line text

## Test Cases
1. Click at (5,5) type "A" -> verify "A" at row 5
2. Click at (10,5) type "B" -> verify "B" at row 10 
3. Type 5 chars at (5,5) -> verify all 5 appear in sequence
4. Type, click elsewhere, type -> verify fresh start
5. Zoom in, type -> verify text appears correctly
6. Type, backspace, type -> verify correct behavior
