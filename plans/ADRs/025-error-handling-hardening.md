# ADR-025: Error Handling Hardening

## Status
Proposed

## Date
2026-03-03

## Context

Several error handling issues exist in production (non-test) code:

### unwrap() in production code
While most of the 32 `unwrap()` calls are in `#[cfg(test)]` blocks (acceptable), approximately 4 exist in production code paths where they could panic:
- `find()` results on iterator searches
- `last()` on potentially empty collections
- Grid `.get()` results that could be `None` on out-of-bounds access

### Known bugs from TECHNICAL_ANALYSIS.md
1. **SelectTool missing boundary clamp** (High): Selection coordinates can exceed grid bounds
2. **ArrowTool Bresenham bug** (High): Negative `dy` causes incorrect arrowhead direction
3. **TextTool wrong boundary check** (High): Uses wrong variable for width check, allowing writes past grid edge
4. **Selection state not synced after cut** (High): Cutting selection doesn't clear selection state
5. **Paste always at origin** (Medium): Paste operation ignores current cursor/selection position

### WASM panic implications
In WASM, a panic causes `unreachable` instruction which terminates the entire WASM instance. The editor becomes unresponsive with no recovery possible. Users must reload the page.

## Decision

### 1. Replace unwrap() with safe alternatives

| Pattern | Replacement |
|---------|-------------|
| `collection.find(...).unwrap()` | `.find(...).unwrap_or(&default)` or `if let Some(item) = ...` |
| `collection.last().unwrap()` | `.last().unwrap_or(&default)` or `if let Some(last) = ...` |
| `grid.get(x, y).unwrap()` | `.get(x, y).unwrap_or_default()` |

### 2. Fix boundary bugs

**SelectTool**: Clamp selection coordinates to `0..grid.width()-1` and `0..grid.height()-1` in `on_pointer_move` and `on_pointer_up`.

**ArrowTool**: Fix arrowhead direction calculation to use `abs(dy)` for direction detection, then apply correct arrow character based on actual direction signs.

**TextTool**: Fix boundary check to use `grid.width()` instead of incorrect variable. Add vertical bounds check for newline handling.

### 3. Add defensive checks in WASM bridge

Wrap all tool dispatch in `bindings.rs` with bounds checking before passing coordinates to tools:
```rust
let x = x.clamp(0, self.state.grid().width() as i32 - 1);
let y = y.clamp(0, self.state.grid().height() as i32 - 1);
```

### 4. Add regression tests

For each bug fix, add a specific test that reproduces the original failure:
- `test_select_tool_boundary_clamp` — selection at grid edges
- `test_arrow_tool_negative_direction` — arrows pointing up-left
- `test_text_tool_edge_writing` — typing at grid boundary

## Consequences

### Positive
- Zero panic potential in production WASM code
- Known bugs resolved with regression test coverage
- More resilient editor — invalid input produces graceful degradation, not crashes
- Better user experience — no mysterious editor freezes

### Negative
- Some `unwrap_or_default()` replacements silently hide errors (acceptable for UI tool, would need logging in a backend service)
- Boundary clamping may change observable behavior for edge cases

### Risks
- Fixing ArrowTool Bresenham might change arrow rendering for some angles — need visual verification
