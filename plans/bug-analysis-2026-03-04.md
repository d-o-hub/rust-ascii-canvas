# Bug Analysis - 2026-03-04 Evening Session

## Issue 1: Select Tool Move Functionality Missing ✅ FIXED

**User Report**: "select should select and move the selected object"

**Analysis**:
- Select tool CAN create selections (visual highlight works after recent fix)
- Select tool has move infrastructure (`moving` flag, `move_offset`, etc.) in `select.rs`
- BUT: `on_pointer_move` when `moving=true` returns empty ToolResult
- No ops are generated to actually cut/paste the selection to a new location

**Root Cause**: The move functionality was **stubbed out** - comment at line 110 said "Moving is handled by the editor state" but nothing in EditorState actually handled it.

**Fix Implemented** (2026-03-04):
1. Added `move_clipboard`, `move_original_selection`, and `is_moving_selection` fields to `AsciiEditor`
2. In `on_pointer_down`: Detect click inside selection → set `is_moving_selection=true` → capture content via `start_selection_move()`
3. In `on_pointer_move`: Generate preview ops showing cleared original area + content at new position
4. In `on_pointer_up`: Commit move as single undo operation via `commit_selection_move()`
5. Select tool's `on_pointer_move` updates selection bounds during move

**Files Changed**:
- `src/core/tools/select.rs`: Added helper methods `get_move_offset()`, `is_moving()`, updated `on_pointer_move()` to update selection bounds during move
- `src/wasm/bindings.rs`: Added move state tracking, `start_selection_move()`, `generate_move_preview_ops()`, `commit_selection_move()`

**Test Result**: ✅ All 63 E2E tests pass, including "should select and move a shape"

## Issue 2: Eraser Tool ✅ VERIFIED WORKING

**User Report**: "erase should works"

**Investigation Result** (2026-03-04):
- ✅ Unit tests pass: `test_eraser_tool`, `test_eraser_at`, `test_larger_eraser`
- ✅ E2E test passes: "Eraser tool clears content" 
- ✅ Tool code is correct: marked as incremental, commits ops during drag
- ✅ All 63 E2E tests pass including eraser functionality

**Analysis**:
The eraser tool IS working correctly. Possible sources of user confusion:
1. **Eraser size = 1** (default): Only clears single cell at cursor position
2. **Tool selection**: User may not have eraser actually selected (E shortcut or button click)
3. **Visual feedback**: Eraser clears cells to space character ' ' which may not be visibly different on empty canvas

**Conclusion**: No bug found. Eraser functionality works as designed. User may need guidance on:
- How to select the eraser tool (E key or click eraser button)
- Understanding that size=1 eraser only clears 1 cell
- Future enhancement: Add UI for adjusting eraser size (currently hardcoded to 1)

## Issue 3: Drawing Boundary at x=80 ✅ NOT A BUG

**User Report**: "drawing works only when the cursor is in 79,x - after 80 is not drawing"

**Investigation Result** (2026-03-04):
- Grid is 80 columns × 40 rows (defined in `web/main.ts`: `GRID_WIDTH = 80`, `GRID_HEIGHT = 40`)
- Valid column indices: **0-79** (80 columns total, zero-indexed)
- Valid row indices: **0-39** (40 rows total, zero-indexed)

**Analysis**:
This is **expected behavior**, not a bug. The grid has 80 columns numbered 0-79.

- `clamp_to_grid()` correctly clamps x to `0..(width-1)` → `0..79`
- When user tries to draw at column 80, it gets clamped to column 79
- This is correct: **column 80 does not exist** in an 80-column grid

**Explanation for User**:
The grid is 80 cells wide, with columns numbered 0-79 (not 1-80). This is standard zero-based indexing. Drawing "stops" at column 79 because that's the rightmost column. Column 80 would be outside the grid bounds.

**Status**: Working as designed. No fix needed.

---

## Summary

| Issue | Status | Action Taken |
|-------|--------|--------------|
| Select tool move | ✅ FIXED | Implemented move functionality with preview and commit |
| Eraser not working | ✅ VERIFIED | Confirmed working - user education needed |
| Drawing stops at x=80 | ✅ NOT A BUG | Expected behavior - grid is 0-indexed |

**Test Results After Fixes**:
- ✅ 78 unit tests pass
- ✅ 63 E2E tests pass (Chromium)
- ✅ Select and move test specifically verified
- ✅ Eraser test specifically verified

**Code Quality**:
- No compiler errors
- No clippy warnings
- All existing functionality preserved
- New move functionality integrates cleanly with undo/redo system
