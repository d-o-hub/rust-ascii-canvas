# ADR-012: Grid Size Customization

## Status
Proposed

## Context
The grid size is hardcoded to 80x40 cells in `web/main.ts`. Users cannot create diagrams with different dimensions without modifying code.

### Current Implementation
```typescript
// web/main.ts:66-67
const GRID_WIDTH = 80;
const GRID_HEIGHT = 40;
```

### User Impact
- Cannot create wide diagrams (e.g., flowcharts)
- Cannot create tall diagrams (e.g., vertical sequences)
- Fixed aspect ratio limits diagram flexibility

## Decision
Implement grid customization with UI controls and persistence:

### 1. Grid Resize API
Add to `AsciiEditor`:
```rust
pub fn resize(&mut self, width: usize, height: usize) {
    // Clear existing content (with confirmation in UI)
    self.state.grid = Grid::new(width, height);
    self.history.clear();
    self.dirty_tracker.request_full_redraw();
}
```

### 2. UI Controls
Add grid size controls to the toolbar or a settings panel:

| Control | Location | Behavior |
|---------|----------|----------|
| Width input | Settings panel | Number input, 10-200 range |
| Height input | Settings panel | Number input, 10-100 range |
| Apply button | Settings panel | Resize grid (prompts if content exists) |
| Presets dropdown | Settings panel | Common sizes (80x25, 80x40, 120x50) |

### 3. Persistence
Save grid size preference to localStorage:
```typescript
interface EditorPreferences {
    gridWidth: number;
    gridHeight: number;
    // Future: theme, lastUsed, etc.
}
```

### 4. Constraints
- Min: 10x10 (reasonable minimum)
- Max: 200x100 (performance limit)
- Default: 80x40 (current behavior)

## Consequences

### Positive
- Users can create diagrams of any aspect ratio
- Settings persist across sessions
- Follows standard application conventions

### Negative
- Resize clears existing content (data loss risk)
- Need confirmation dialog
- May affect performance at very large sizes
- E2E tests assume 80x40 grid

### Mitigations
1. Show warning dialog before resize if grid has content
2. Store previous grid in localStorage (undo within session)
3. Performance test at max size (200x100 = 20,000 cells)

## Implementation Plan
1. Add `resize()` method to `AsciiEditor`
2. Create settings panel UI in `index.html`
3. Add preferences storage in `main.ts`
4. Update E2E tests to handle dynamic grid size
5. Add resize confirmation dialog

## Test Cases
- Resize to 100x50, verify grid dimensions
- Draw content, attempt resize, confirm dialog appears
- Refresh page, verify saved size loads
- Test resize with Undo history cleared

## Future Considerations
- "Fit to content" - auto-size to minimum bounding box
- Infinite canvas with viewport window
- Custom cell sizes (not just character cells)
