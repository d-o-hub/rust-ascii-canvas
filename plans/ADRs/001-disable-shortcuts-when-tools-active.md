# ADR: Disable keyboard shortcuts when tools are active

## Status

**Proposed** - 2026-02-26

## Context

The ASCII Canvas editor has a keyboard shortcut system where pressing single keys (R, T, V, L, A, D, F, E) switches between tools. However, there is a critical bug where these shortcuts are processed **before** checking if the current tool is busy/active.

### Current Implementation

In `src/wasm/bindings.rs`, the `on_key_down()` method processes tool shortcuts at lines 288-293:

```rust
// Handle tool shortcuts
if !ctrl && !shift {
    if let Some(tool_id) = ToolId::from_shortcut(key_char) {
        self.set_tool_by_id(tool_id);
        return serde_wasm_bindgen::to_value(&self.create_event_result())
            .unwrap_or(JsValue::NULL);
    }
}
```

This runs **BEFORE** checking if the tool is active at lines 297-303:

```rust
// Handle text input
if self.tool_id == ToolId::Text && self.active_tool.is_active() {
    let ctx = self.create_tool_context();
    let result = self.active_tool.on_key(key_char, &ctx);
    if result.modified {
        self.commit_ops(&result.ops);
    }
}
```

### Problem Scenarios

1. **Text Tool Active**: User clicks to set cursor, then presses 'R' to type 'R' character. Instead, tool switches to Rectangle.

2. **Select Tool Active**: User drags to create a selection, then accidentally presses 'T'. Tool switches to Text, losing the selection.

3. **Drawing Tools Active**: User starts dragging a Rectangle/Line, then presses another shortcut mid-drag. The tool switches unexpectedly.

### Tool `is_active()` Implementation

The `Tool` trait (in `src/core/tools/mod.rs`) defines `is_active()` method. Each tool implements it:

| Tool | Active Condition | File |
|------|-----------------|------|
| TextTool | `cursor.is_some()` | `text.rs:139-141` |
| SelectTool | `selecting \|\| moving` | `select.rs:133-135` |
| RectangleTool | `start.is_some()` | `rectangle.rs:129-131` |
| LineTool | `start.is_some()` | `line.rs:117-119` |
| FreehandTool | `drawing` | `freehand.rs:143-145` |
| EraserTool | `erasing` | `eraser.rs:160-162` |

## Decision

Add a check for `!self.active_tool.is_active()` before processing tool shortcuts in `on_key_down()`.

### Proposed Code Change

In `src/wasm/bindings.rs`, modify lines 288-294 from:

```rust
// Handle tool shortcuts
if !ctrl && !shift {
    if let Some(tool_id) = ToolId::from_shortcut(key_char) {
        self.set_tool_by_id(tool_id);
        return serde_wasm_bindgen::to_value(&self.create_event_result())
            .unwrap_or(JsValue::NULL);
    }
}
```

To:

```rust
// Handle tool shortcuts (only when tool is not active)
if !ctrl && !shift && !self.active_tool.is_active() {
    if let Some(tool_id) = ToolId::from_shortcut(key_char) {
        self.set_tool_by_id(tool_id);
        return serde_wasm_bindgen::to_value(&self.create_event_result())
            .unwrap_or(JsValue::NULL);
    }
}
```

## Consequences

### What This Fixes

1. **Text tool**: Typing characters like 'R', 'T', 'V', etc. will work correctly when cursor is active
2. **Select tool**: Accidental shortcut presses won't cancel active selections
3. **Drawing tools**: Mid-drag tool switches are prevented
4. **Consistent behavior**: All tools respect their active state during keyboard input

### Potential Side Effects

1. **No way to cancel active operation**: Users cannot press a shortcut to cancel an active operation (e.g., pressing Escape to cancel text input is not implemented). Consider adding Escape key handling to cancel/reset tools.

2. **Locked-in state**: If a tool gets stuck in active state (bug), users cannot switch tools via keyboard. Should verify `reset()` is called properly in all cases.

### Additional Improvements Needed

1. **Add Escape key handling**: Add support for Escape key to cancel/reset active tools:
   ```rust
   if key == "Escape" {
       self.active_tool.reset();
       self.preview_ops.clear();
       return serde_wasm_bindgen::to_value(&self.create_event_result())
           .unwrap_or(JsValue::NULL);
   }
   ```

2. **Verify reset() behavior**: Ensure all tools properly reset their active state, particularly the Text tool which has cursor state.

3. **Consider active state indicator**: The UI could show when a tool is active (e.g., cursor visible for text tool).

## References

- Tool trait definition: `src/core/tools/mod.rs:222-246`
- Bindings implementation: `src/wasm/bindings.rs:256-306`
- ToolId shortcuts: `src/core/tools/mod.rs:64-77`
