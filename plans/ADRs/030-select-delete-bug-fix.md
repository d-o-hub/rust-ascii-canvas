# ADR-030: Select Tool Delete Selection Bug Fix

## Status
Accepted

## Context

The Select tool is supposed to allow users to delete selected content by pressing Delete or Backspace. However, the current implementation in `src/wasm/bindings.rs:268-286` has a bug:

```rust
if !ctrl && (key == "Delete" || key == "Backspace") {
    if self.tool_id == ToolId::Select
        && self.active_tool.is_active()  // BUG: This is false after selection is made!
        && self.delete_selection()
    {
        // ...
    }
}
```

The issue is that `SelectTool::is_active()` returns `true` only during dragging/selecting (`self.selecting || self.moving`), but NOT after a selection has been made and the user has released the mouse button.

This means:
1. User drags to create a selection
2. User releases mouse button → `selecting = false`, `is_active() = false`
3. User presses Delete → condition fails because `is_active() == false`
4. Selection is NOT deleted

## Decision

Change the condition to check for the existence of a current selection instead of tool active state:

```rust
if !ctrl && (key == "Delete" || key == "Backspace") {
    if self.tool_id == ToolId::Select
        && self.current_selection.is_some()  // FIXED: Check for selection
        && self.delete_selection()
    {
        let event_result = self.create_event_result();
        return serde_wasm_bindgen::to_value(&event_result).unwrap_or(JsValue::NULL);
    }
    // ...
}
```

## Consequences

### Positive
- Delete/Backspace will work correctly to delete selected content
- User experience matches expected behavior from other editors
- Consistent with how Cut (Ctrl+X) already works

### Negative
- None identified

## Implementation

1. Modify `src/wasm/bindings.rs:268-275` to use `self.current_selection.is_some()` instead of `self.active_tool.is_active()`
2. Add E2E test to verify Select + Delete functionality

## Test Cases

1. Draw a rectangle
2. Switch to Select tool (V)
3. Drag to select the rectangle
4. Press Delete
5. Verify rectangle is erased (selection area is cleared)

## Related Files
- `src/wasm/bindings.rs` - Main fix location
- `src/core/tools/select.rs` - SelectTool implementation
- `e2e/canvas.spec.ts` - E2E tests
