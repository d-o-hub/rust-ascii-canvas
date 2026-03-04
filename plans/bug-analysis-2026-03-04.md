# Bug Analysis - 2026-03-04 Evening Session

## Issue 1: Select Tool Move Functionality Missing

**User Report**: "select should select and move the selected object"

**Analysis**:
- Select tool CAN create selections (visual highlight works after recent fix)
- Select tool has move infrastructure (`moving` flag, `move_offset`, etc.) in `select.rs`
- BUT: `on_pointer_move` when `moving=true` returns empty ToolResult
- No ops are generated to actually cut/paste the selection to a new location

**Root Cause**: The move functionality is **stubbed out** - comment at line 110 says "Moving is handled by the editor state" but nothing in EditorState actually handles it.

**Fix Required**:
1. When user clicks inside selection and drags, need to:
   - Store original content from selection bounds
   - Clear original location  
   - Draw content at new location (offset by drag delta)
2. On pointer_up, commit the move as a single undo operation

## Issue 2: Eraser Tool

**User Report**: "erase should works"

**Test Result**: ✅ Eraser test PASSES in E2E suite

**Analysis**:
- Eraser tool code looks correct
- E2E test "Eraser tool clears content" passes
- Tool is marked as incremental, commits ops during drag

**Conclusion**: Eraser IS working. User may be experiencing a different issue:
- Confusion about which tool is selected?
- Eraser size too small (size=1, only clears 1 cell)?
- Border style characters not being erased? (They should - eraser sets cell to ' ')

**Action**: Need user clarification on what specifically isn't working.

## Issue 3: Drawing Boundary at x=80

**User Report**: "drawing works only when the cursor is in 79,x - after 80 is not drawing"

**Analysis**:
- Grid is 80x40 (columns 0-79, rows 0-39)
- `clamp_to_grid` correctly clamps x to `0..(width-1)` i.e., `0..79`
- For x=80, it gets clamped to x=79 (expected behavior)

**Possible Interpretations**:
1. User expects grid to be 81 wide? (columns 0-80)
2. User is seeing visual misalignment where they click at position 80 pixels but it draws at 79?
3. User means something different by "79,x"?

**Action**: Need clarification on expected vs actual behavior.

---

## Priority

1. **HIGH**: Implement select tool move functionality (clear missing feature)
2. **MEDIUM**: Clarify eraser issue (may be working as designed)
3. **LOW**: Clarify grid boundary issue (likely working as designed, 80-wide = indices 0-79)
