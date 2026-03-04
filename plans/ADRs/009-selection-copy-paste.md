# ADR-009: Selection Copy/Paste Operations

## Status
Proposed

## Context
The Select tool can create selections and move them, but lacks copy/paste functionality. This is a critical gap for a diagram editor where users frequently need to duplicate shapes or copy content between areas.

### Current Implementation
- `src/core/selection.rs` defines `Selection` struct with bounds/translation methods
- `src/core/tools/select.rs` handles selection creation and moving state
- No internal clipboard or copy/paste operations exist

### User Impact
Users must redraw identical shapes manually, which is inefficient and error-prone.

## Decision
Implement a grid-internal clipboard system with the following operations:

### 1. Internal Clipboard Structure
```rust
pub struct SelectionClipboard {
    cells: Vec<(i32, i32, Cell)>,  // Relative positions with cell content
    width: i32,
    height: i32,
}
```

### 2. Operations to Add
| Operation | Shortcut | Behavior |
|-----------|----------|----------|
| Copy | Ctrl+C | Copy selection to internal clipboard |
| Cut | Ctrl+X | Copy selection, clear original cells |
| Paste | Ctrl+V | Insert clipboard content at cursor |
| Delete | Delete/Backspace | Clear selection cells |

### 3. API Changes
Add to `AsciiEditor`:
- `copy_selection() -> bool`
- `cut_selection() -> bool`
- `paste() -> bool`
- `delete_selection() -> bool`

### 4. Implementation Approach
1. Store clipboard as relative coordinates from selection origin
2. Paste inserts at current cursor position (not selection center)
3. Clear clipboard on grid resize
4. Selection is cleared after paste operation

## Consequences

### Positive
- Dramatically improved productivity for diagram creation
- Consistent with standard editor conventions (Ctrl+C/V/X)
- Reuses existing Selection infrastructure
- No external dependencies needed

### Negative
- Internal clipboard is separate from system clipboard
- Need to handle bounds checking on paste
- Adds complexity to keyboard event handling

### Risks
- Large selections could consume significant memory
- Need to handle edge cases (empty selection, out-of-bounds paste)

## Implementation Plan
1. Create `SelectionClipboard` struct in `src/core/selection.rs`
2. Add clipboard field to `AsciiEditor`
3. Implement copy/cut/paste/delete methods
4. Add keyboard handlers in `on_key_down`
5. Add E2E tests for each operation

## Related
- ADR-001: Disable shortcuts when tools active (keyboard handling pattern)
- ADR-003: Enhanced keyboard UI
