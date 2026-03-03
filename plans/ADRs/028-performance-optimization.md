# ADR-028: Performance Optimization

## Status
Proposed

## Date
2026-03-03

## Context

While the editor achieves 60 FPS under normal usage, profiling and code review reveal several performance anti-patterns that will become bottlenecks as diagrams grow larger or as features like layers multiply the rendering work:

### Identified Issues

1. **`.to_string()` on every mouse move** (`bindings.rs`): Every pointer event converts strings for tool state, creating garbage that the WASM allocator must handle. At 60 FPS with mouse tracking, this is ~60 allocations/second.

2. **String allocations for static colors** (`canvas_renderer.rs`): Theme colors like `"#1e1e1e"` are allocated as `String` on every render command. These are compile-time constants that should be `&'static str`.

3. **Full grid iteration on every render** (`canvas_renderer.rs`): `build_full_render()` iterates all 80x40 = 3,200 cells even when most are empty. With layers, this becomes N * 3,200.

4. **`Selection` clone on every `get_selection()` call**: Returns `Option<Selection>` by value, cloning the selection struct each time. Called frequently during tool operations.

5. **`SmallVec` not leveraged optimally**: Some tools create `Vec<DrawOp>` when `SmallVec<[DrawOp; 16]>` would avoid heap allocation for small draws.

## Decision

### 1. Eliminate hot-path string allocations

Replace `String` with `&'static str` or `Cow<'static, str>` in render commands:

```rust
// Before
pub struct RenderCommand {
    color: String,      // Heap allocated
}

// After  
pub struct RenderCommand {
    color: Cow<'static, str>,  // Static for known colors, owned for custom
}
```

For tool state strings, use enum variants instead of string matching where possible.

### 2. Sparse grid iteration

Track content bounds and only iterate non-empty regions:

```rust
impl Grid {
    /// Returns the bounding box of all non-empty cells, or None if grid is empty
    pub fn content_bounds(&self) -> Option<Rect> { ... }
    
    /// Iterate only cells within the content bounds
    pub fn iter_content(&self) -> impl Iterator<Item = (usize, usize, &Cell)> { ... }
}
```

Cache the content bounds, invalidating on any `set()` call. Rendering only iterates the content region.

### 3. Return references instead of clones

```rust
// Before
fn get_selection(&self) -> Option<Selection> { self.selection.clone() }

// After
fn get_selection(&self) -> Option<&Selection> { self.selection.as_ref() }
```

### 4. SmallVec for tool draw operations

Standardize on `SmallVec<[DrawOp; 16]>` for all tool results. Most single-click operations produce < 16 draw ops, avoiding heap allocation entirely.

### 5. Benchmark suite

Add `criterion` benchmarks for critical paths:
- Grid iteration (empty, sparse, full)
- Render command generation
- Tool operations (rectangle, line, freehand at various sizes)
- Export (small, medium, large grids)

## Consequences

### Positive
- Reduced GC pressure in WASM — fewer short-lived allocations
- Render time proportional to content, not grid size
- Measurable improvement for large/complex diagrams
- Benchmark suite prevents future regressions

### Negative
- `Cow<'static, str>` is slightly more complex than plain `String`
- Content bounds caching adds invalidation complexity
- Changing `get_selection()` return type is a breaking API change (internal only)
- SmallVec increases stack usage (acceptable for tool lifetime)

### Measurement Plan
1. Establish baseline with `criterion` before changes
2. Apply each optimization independently
3. Measure improvement per optimization
4. Only keep changes that show measurable improvement (>5% on target benchmark)
