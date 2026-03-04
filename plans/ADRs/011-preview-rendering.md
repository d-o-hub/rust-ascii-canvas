# ADR-011: Preview Operations Rendering

## Status
Proposed

## Context
During drag operations (drawing rectangles, lines, etc.), the editor stores `preview_ops` but doesn't render them with visual distinction. Users cannot see what they're drawing until they release the mouse button.

### Current Implementation
- `src/wasm/bindings.rs:197-198` stores `preview_ops = result.ops.clone()`
- Preview ops are not included in render commands
- Blue-tinted preview was planned but not implemented

### User Impact
Drawing complex shapes requires "blind" operation - users only see result after releasing. This makes precise drawing difficult.

## Decision
Implement preview rendering with distinct visual style:

### 1. Preview Render Commands
Add a new render command type for previews:
```rust
pub enum RenderCommand {
    // ... existing commands
    DrawPreview { ops: Vec<PreviewOp> },
}

pub struct PreviewOp {
    x: f64,
    y: f64,
    char: char,
    is_preview: bool,  // For styling
}
```

### 2. Visual Distinction Options
| Option | Description | Pros | Cons |
|--------|-------------|------|------|
| Blue tint | Render preview characters in `#569CD6` (blue) | Matches selection color | May conflict with content |
| Semi-transparent | 50% opacity preview | Doesn't obscure content | May be hard to see |
| Dashed border | Show bounds with dashed line | Clear indication | Doesn't show content |
| Combination | Blue tint + semi-transparent | Best visibility | More complex |

**Chosen**: Blue tint with semi-transparency (`rgba(86, 156, 214, 0.7)`)

### 3. Implementation Approach
1. Add `get_preview_commands()` method to `AsciiEditor`
2. Return preview ops as separate render commands
3. TypeScript renders preview layer on top of grid
4. Clear preview when operation completes

### 4. Preview vs Selection
- **Selection preview**: Blue rectangle overlay (`#264f78`)
- **Drawing preview**: Blue-tinted characters at 70% opacity
- Both should be distinguishable from committed content

## Consequences

### Positive
- Users see what they're drawing before committing
- Better precision for complex diagrams
- Matches user expectations from other drawing tools
- Improved accessibility for users with visual impairments

### Negative
- Additional render pass per frame during drag
- More complex render pipeline
- Need to handle preview clearing on tool switch

### Performance Considerations
- Preview rendering should use same dirty-rect optimization
- Consider batching preview ops into single draw call
- May need to limit preview complexity for very large shapes

## Implementation Plan
1. Add `preview_commands` to `EditorEventResult`
2. Modify `create_event_result()` to include preview ops
3. Update TypeScript render to handle preview commands
4. Add CSS class for preview styling
5. Add E2E tests for preview visibility

## Related
- `plans/TECHNICAL_ANALYSIS.md` - Blue-Tinted Preview (Deferred) section
- Selection uses `#264f78` color - should use similar palette
