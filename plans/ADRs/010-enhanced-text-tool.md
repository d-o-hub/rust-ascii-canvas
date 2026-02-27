# ADR-010: Enhanced Text Tool

## Status
Proposed

## Context
The current Text tool only places single characters per click. Users cannot efficiently type multi-character labels, correct mistakes, or see where text will appear.

### Current Implementation
- `src/core/tools/text.rs` handles `on_key` for single character insertion
- No text cursor visualization
- No backspace/delete support
- No multi-character buffer

### User Impact
Typing "Hello World" requires 11 separate clicks and key presses with no way to correct mistakes.

## Decision
Transform the Text tool into a proper text editor with cursor and buffer:

### 1. Text Buffer in Tool State
```rust
pub struct TextTool {
    cursor_pos: Option<(i32, i32)>,
    text_buffer: String,
    is_typing: bool,
}
```

### 2. Enhanced Behaviors
| Action | Behavior |
|--------|----------|
| Click | Position cursor, start text entry |
| Type char | Insert character at cursor, advance cursor |
| Backspace | Delete previous character, move cursor back |
| Delete | Delete character at cursor |
| Escape | Cancel text entry, clear buffer |
| Enter | Commit text to grid, advance to next line |

### 3. Visual Feedback
- Render cursor position with blinking underscore or block
- Show text buffer as preview (semi-transparent) before commit
- Different cursor style when tool is active

### 4. Cursor Movement
- Arrow keys to move cursor within text buffer (optional, Phase 2)
- Home/End to jump to start/end (optional, Phase 2)

## Consequences

### Positive
- Natural text editing experience
- Ability to correct mistakes before committing
- Consistent with standard text input conventions
- Better UX for diagram labels and annotations

### Negative
- Increased tool state complexity
- Need cursor rendering in preview layer
- More keyboard event handling

### Trade-offs
- **Buffer vs Direct**: Store text in buffer before commit (chosen) vs write directly to grid
  - Buffer approach allows backspace correction
  - Direct approach is simpler but less user-friendly

## Implementation Plan
1. Add `text_buffer` and `cursor_pos` to `TextTool`
2. Modify `on_key` to handle Backspace/Delete/Enter/Escape
3. Return preview ops for text buffer rendering
4. Add cursor blink animation in TypeScript (CSS animation)
5. Add E2E tests for multi-character text entry

## Test Cases
- Type "Hello", see all characters
- Type "Hello", Backspace, see "Hell"
- Type "Hello", Enter, see text on grid, cursor moves to next line
- Type "Hello", Escape, see nothing committed
