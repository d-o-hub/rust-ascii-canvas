# ADR-026: Layer System

## Status
Proposed

## Date
2026-03-03

## Context

The ASCII Canvas Editor currently operates on a single flat `Grid`. Every tool draws to the same surface, making it impossible to:

1. **Isolate components** — diagrams with multiple logical sections (e.g., network topology + annotations) cannot be independently edited
2. **Non-destructive editing** — drawing over existing content permanently replaces it
3. **Toggle visibility** — no way to show/hide parts of a diagram for presentations or comparisons
4. **Organize complex diagrams** — large ASCII art projects (100+ characters) become unmanageable

This is the single most-requested feature pattern in diagram editors (Figma, draw.io, Excalidraw all have layers).

## Decision

### Data Model

```rust
pub struct Layer {
    pub id: u32,
    pub name: String,
    pub grid: Grid,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f32,  // 0.0-1.0, for future use (ASCII has no real opacity)
}

pub struct LayerStack {
    layers: Vec<Layer>,
    active_layer_id: u32,
    next_id: u32,
}
```

### API Surface

```rust
impl LayerStack {
    pub fn new(width: usize, height: usize) -> Self;
    pub fn add_layer(&mut self, name: &str) -> u32;  // returns id
    pub fn remove_layer(&mut self, id: u32) -> bool;
    pub fn active_layer(&self) -> &Layer;
    pub fn active_layer_mut(&mut self) -> &mut Layer;
    pub fn set_active(&mut self, id: u32);
    pub fn move_layer(&mut self, id: u32, new_index: usize);
    pub fn toggle_visibility(&mut self, id: u32);
    pub fn toggle_lock(&mut self, id: u32);
    pub fn flatten(&self) -> Grid;  // composite all visible layers
    pub fn merge_down(&mut self, id: u32);  // merge layer with one below
}
```

### Rendering Strategy

Compositing is bottom-up: iterate layers from index 0 (bottom) to N (top). For each visible layer, overlay non-empty cells onto the composite grid. The renderer receives the flattened grid, so no changes to `CanvasRenderer` are needed.

### Tool Integration

All tools operate on `layer_stack.active_layer_mut().grid` instead of `editor_state.grid()`. Locked layers reject all tool operations. The active layer is visually indicated in the UI.

### Undo/Redo

Commands store the `layer_id` they operated on. Undo/redo targets the specific layer. Layer operations themselves (add, remove, reorder) are also undoable commands.

### WASM Bindings

```rust
// New methods on AsciiEditor
pub fn add_layer(&mut self, name: &str) -> u32;
pub fn remove_layer(&mut self, id: u32) -> bool;
pub fn set_active_layer(&mut self, id: u32);
pub fn move_layer(&mut self, id: u32, new_index: usize);
pub fn toggle_layer_visibility(&mut self, id: u32);
pub fn toggle_layer_lock(&mut self, id: u32);
pub fn get_layers_json(&self) -> JsValue;  // [{id, name, visible, locked}]
pub fn merge_layer_down(&mut self, id: u32);
```

### UI Design

A collapsible layer panel on the right side (replacing or augmenting the existing side panel):
- Layer list with drag-to-reorder
- Eye icon for visibility toggle
- Lock icon for lock toggle
- "+" button to add layer
- Context menu: rename, delete, merge down, duplicate
- Active layer highlighted

## Consequences

### Positive
- Complex diagrams become manageable
- Non-destructive editing possible
- Professional-grade feature parity with other diagram tools
- Foundation for future features (layer groups, blend modes, templates)

### Negative
- Memory usage increases linearly with layer count (each layer is a full Grid)
- Compositing adds overhead to every render (mitigated by dirty-rect system)
- Undo/redo complexity increases significantly
- Export must decide: flatten or per-layer?
- Increases learning curve for simple use cases

### Migration
- Default state: single layer named "Layer 1" — backwards compatible
- Existing saved diagrams (if any) import as single-layer

### Risks
- Layer-aware undo/redo is complex and bug-prone — needs thorough testing
- Compositing with non-ASCII characters (Unicode box drawing) may have edge cases with overlapping
