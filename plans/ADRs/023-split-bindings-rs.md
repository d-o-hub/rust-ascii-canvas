# ADR-023: Split bindings.rs Into Focused Modules

## Status
Proposed

## Date
2026-03-03

## Context

`src/wasm/bindings.rs` is 722 lines — the largest file in the codebase and well beyond the project's own 500 LOC guideline. It currently handles:

1. **AsciiEditor struct definition** (fields, constructor)
2. **Tool management** (setTool, setBorderStyle, setLineDirection, tool creation)
3. **Event handling** (onPointerDown/Move/Up, onKeyDown/Up, onWheel)
4. **Rendering** (getRenderCommands, getDirtyRenderCommands, requestRedraw)
5. **Clipboard operations** (copySelection, cutSelection, paste, deleteSelection)
6. **History** (undo, redo)
7. **Export** (exportAscii)
8. **Selection state management** (update_select_tool_selection)

This monolithic design makes the file hard to navigate, test, and modify.

## Decision

Split `bindings.rs` into 4 focused modules while keeping `AsciiEditor` as the single `#[wasm_bindgen]` public type:

### New module structure:
```
src/wasm/
├── mod.rs              # Re-exports
├── bindings.rs         # AsciiEditor struct + constructor + wasm_bindgen facade (~200 LOC)
├── tool_manager.rs     # Tool creation, switching, border/direction config (~150 LOC)
├── event_handlers.rs   # Pointer, keyboard, wheel event processing (~200 LOC)
└── render_bridge.rs    # Render commands, export, clipboard bridge (~150 LOC)
```

### Implementation approach:
- `tool_manager.rs`: Contains `impl AsciiEditor` block for `set_tool()`, `set_border_style()`, `set_line_direction()`, tool instantiation logic
- `event_handlers.rs`: Contains `impl AsciiEditor` block for `on_pointer_down/move/up()`, `on_key_down/up()`, `on_wheel()`, commit logic
- `render_bridge.rs`: Contains `impl AsciiEditor` block for `get_render_commands()`, `get_dirty_render_commands()`, `request_redraw()`, `export_ascii()`, clipboard methods
- `bindings.rs`: Retains struct definition, `new()`, and `#[wasm_bindgen]` method delegation

Rust allows `impl` blocks in different modules for the same struct within the same crate, so this is idiomatic.

## Consequences

### Positive
- All files under 500 LOC
- Clear separation of concerns
- Easier to find and modify specific functionality
- Each module can have focused unit tests
- Better code review — changes to event handling don't touch rendering code

### Negative
- Slightly more files to navigate
- `#[wasm_bindgen]` attribute must remain on methods in `bindings.rs` or be re-exported (may require delegation pattern)
- Fields of `AsciiEditor` must be `pub(crate)` for other modules to access

### Technical Note
Since `wasm_bindgen` requires all `#[wasm_bindgen]` methods to be on the struct definition or in the same module, the split modules will contain regular Rust methods, and `bindings.rs` will have thin `#[wasm_bindgen]` wrappers that delegate to them.
