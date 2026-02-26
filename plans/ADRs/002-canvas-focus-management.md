# ADR-002: Canvas Focus Management for Keyboard Input

## Status

**Proposed** - 2026-02-26

## Context

The ASCII Canvas Editor has a critical UX issue where all keyboard input stops working after clicking any toolbar button (undo, redo, copy, tool buttons, etc.). This breaks the core functionality of the text tool and keyboard shortcuts.

### Current Implementation

In `web/main.ts`, the canvas receives keyboard events:

```typescript
// Line 160-161
canvas.addEventListener('keydown', handleKeyDown);
canvas.addEventListener('keyup', handleKeyUp);
```

The canvas is made focusable via `tabindex="0"` in `web/index.html:99`:

```html
<canvas id="canvas" tabindex="0" aria-label="ASCII Canvas Editor"></canvas>
```

### Problem

When clicking any toolbar button:
1. Browser automatically moves focus to the clicked button
2. Canvas loses focus
3. Keyboard events no longer trigger canvas event handlers
4. User must manually click canvas to restore functionality

### Affected Scenarios

| Scenario | Expected | Actual |
|----------|----------|--------|
| Click undo, then type with Text tool | Text appears | No text, no response |
| Click copy button, press R | Switch to Rectangle tool | Nothing happens |
| Click tool button, press Ctrl+Z | Undo action | Nothing happens |

## Decision

Implement a two-part solution:

### Part 1: Prevent Focus Stealing on Toolbar Buttons

Add `mousedown` event handler to all toolbar buttons with `e.preventDefault()` to prevent the button from taking focus:

```typescript
// Add to button event handlers
btn.addEventListener('mousedown', (e) => e.preventDefault());
```

This is the standard pattern recommended by:
- Yale Usability & Web Accessibility guide
- WCAG 2.1 Success Criterion 2.4.3 (Focus Order)
- BBC Accessibility Guidelines for Focusable Controls

### Part 2: Restore Focus on Canvas Pointer Events

Add focus restoration when user interacts with canvas via pointer:

```typescript
canvas.addEventListener('pointerdown', () => {
    canvas.focus();
});
```

This ensures:
- First click on canvas restores focus
- Focus is maintained during drawing operations
- Works with existing pointer capture

## Consequences

### What This Fixes

1. **Text tool**: Typing works immediately after any button click
2. **Tool shortcuts**: R, T, V, L, A, D, F, E work after button clicks
3. **Keyboard shortcuts**: Ctrl+Z, Ctrl+Shift+Z, Ctrl+C work after button clicks
4. **Accessibility**: Maintains keyboard operability per WCAG guidelines

### Implementation Changes

Modified files:
- `web/main.ts`: Add mousedown handlers and canvas focus restoration

### Trade-offs

1. **Button focus states**: Buttons won't show focus ring via default browser behavior. We should add custom `:focus-visible` CSS to maintain accessibility.

2. **tabindex requirement**: Canvas must remain `tabindex="0"` for this to work.

### Alternative Approaches Considered

1. **Document-level keyboard listener**: Would work but adds complexity and potential conflicts with other inputs.

2. **Restore focus after each action**: Would cause visual flicker and interrupt user flow.

3. **Remove tabindex**: Would break keyboard shortcuts entirely.

## Implementation Plan

1. Add `mousedown` handler with `preventDefault()` to:
   - All tool buttons (lines 164-171)
   - Undo button (line 181)
   - Redo button (line 188)
   - Copy button (line 195)
   - Clear button (line 197)
   - Border style select (line 174) - use `mousedown`

2. Add canvas focus restoration on pointer events (line 148-149):

```typescript
canvas.addEventListener('pointerdown', (e) => {
    canvas.focus();
    // existing handler code
});
```

3. Add CSS for button focus states in `web/style.css`:

```css
.tool-btn:focus-visible,
.action-btn:focus-visible,
.select-input:focus-visible {
    outline: 2px solid #60a5fa;
    outline-offset: 2px;
}
```

## References

- [Yale: Focus & Keyboard Operability](https://usability.yale.edu/web-accessibility/articles/focus-keyboard-operability)
- [BBC: Focusable Controls](https://www.bbc.co.uk/accessibility/forproducts/guides/html/focusable-controls/)
- [MDN: Keyboard Accessibility](https://developer.mozilla.org/en-US/docs/Web/Accessibility/Understanding_WCAG/Keyboard)
- [Stack Overflow: How to keep focus on canvas](https://stackoverflow.com/questions/29992688)
