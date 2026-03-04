# ADR-017: Rust Critical Fixes

## Status
**Proposed** - 2026-03-01

## Context

Comprehensive codebase analysis identified 4 critical issues in the Rust codebase that must be fixed before production deployment.

### Issues Identified

1. **Duplicate `delete()` method definitions** in `text.rs`
2. **Unsafe pointer cast** in `bindings.rs` with potential UB
3. **Panic-prone `unwrap()` calls** in production paths
4. **Potential panic on character conversion** in `mod.rs`

## Decision

Fix all critical issues following Rust best practices 2026:

### Fix 1: Remove Duplicate Methods

```rust
// BEFORE: Three delete() methods in TextTool
// AFTER: Single correct implementation

pub fn delete(&mut self) -> Option<DrawOp> {
    // Delete removes the character AFTER the cursor (if any)
    if let Some((x, y)) = self.cursor {
        if let Some(start) = self.start_pos {
            let idx = (x - start.0) as usize;
            if idx < self.buffer.len() {
                self.buffer.remove(idx);
                return Some(DrawOp::new(x, y, ' '));
            }
        }
    }
    None
}
```

### Fix 2: Replace Unsafe with Downcast

```rust
// BEFORE: Unsafe pointer cast
fn set_line_direction(&mut self, direction: String) {
    let tool_ptr = &mut *self.active_tool as *mut dyn Tool;
    unsafe {
        let line_tool = &mut *(tool_ptr as *mut LineTool);
        line_tool.set_direction(dir);
    }
}

// AFTER: Safe downcast pattern
fn set_line_direction(&mut self, direction: String) {
    if let Some(line_tool) = self.active_tool.as_any_mut().downcast_mut::<LineTool>() {
        line_tool.set_direction(LineDirection::from_str(&direction));
    }
}
```

Requires adding to Tool trait:
```rust
fn as_any_mut(&mut self) -> &mut dyn Any;
```

### Fix 3: Proper Option Handling

```rust
// BEFORE: Panic on None
let idx = (x - self.start_pos.unwrap().0) as usize;

// AFTER: Safe handling
if let (Some(cursor), Some(start)) = (self.cursor, self.start_pos) {
    let idx = (cursor.0 - start.0) as usize;
    // ...
}
```

### Fix 4: Handle Character Conversion None

```rust
// BEFORE: Panic on special chars
match ch.to_uppercase().next().unwrap() {

// AFTER: Safe handling
match ch.to_uppercase().next() {
    Some('R') => Some(Self::Rectangle),
    Some('L') => Some(Self::Line),
    // ...
    _ => None,
}
```

## Consequences

### Positive
- Eliminates undefined behavior risk
- Removes panic potential in production
- Improves code maintainability
- Aligns with Clippy correctness lints

### Negative
- Requires adding `as_any_mut()` to Tool trait
- May require minor refactoring of tool implementations

## Implementation

1. Add `as_any_mut()` to Tool trait
2. Implement in all tool structs
3. Fix unsafe code in bindings.rs
4. Fix duplicate methods in text.rs
5. Fix unwrap() calls
6. Run `cargo clippy -- -D warnings`
7. Run `cargo test`

## References

- [Rust Best Practices Chapter 4](../.agents/skills/rust-best-practices/references/chapter_04.md)
- [Clippy Correctness Lints](https://rust-lang.github.io/rust-clippy/master/index.html#correctness)
