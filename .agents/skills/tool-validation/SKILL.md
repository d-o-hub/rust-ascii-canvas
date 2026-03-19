# Tool Validation Skill

This skill provides a standardized methodology for validating the 8 drawing tools in the ASCII Canvas editor.

## Verification Checklist

### 1. Select Tool (V)
- **Selection**: Dragging should show a semi-transparent blue highlight.
- **Move**: Clicking inside an active selection and dragging should move the content.
- **Delete**: Pressing `Delete` or `Backspace` while a selection is active should clear the region.

### 2. Rectangle Tool (R)
- **Drawing**: Dragging should preview a box with the current border style.
- **Commit**: Releasing the mouse should commit the rectangle to the grid.

### 3. Line Tool (L)
- **Drawing**: Supports horizontal, vertical, and diagonal lines.
- **Direction**: Should respect "Auto", "Horizontal", or "Vertical" overrides if set.

### 4. Arrow Tool (A)
- **Head**: Should draw a directional arrowhead (▲▼◄►) at the end of the line.
- **Overlap**: The arrowhead must not be overwritten by the line character.

### 5. Diamond Tool (D)
- **Shape**: Should draw a rhombus using diagonal lines (╱╲).
- **Small Shapes**: Should fallback to `◆` for single-point or very small drags.

### 6. Text Tool (T)
- **Input**: Clicking on the canvas should set a cursor.
- **Typing**: Alphanumeric keys should appear at the cursor.
- **Controls**: `Enter` (newline), `Backspace` (delete previous), and `Delete` (delete current) must work.

### 7. Freehand Tool (F)
- **Style**: Must use the character associated with the current Border Style (e.g., `·` for Single, `*` for ASCII).
- **Interpolation**: Drawing fast should still produce a continuous line.

### 8. Eraser Tool (E)
- **Function**: Should clear cells within its radius to spaces.
- **Bounds**: Must not emit operations outside the grid coordinates.

## Automation
Use the provided Playwright scripts in `e2e/tools-drawing.spec.ts` for automated regression testing.
