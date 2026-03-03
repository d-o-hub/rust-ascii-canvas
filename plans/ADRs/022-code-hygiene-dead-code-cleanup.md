# ADR-022: Code Hygiene & Dead Code Cleanup

## Status
Proposed

## Date
2026-03-03

## Context

The codebase has accumulated several hygiene issues that mask potential problems:

1. **Crate-level `#![allow(dead_code)]`** in `src/lib.rs:37` suppresses warnings across the entire crate, hiding genuinely unused code.
2. **`thiserror` and `anyhow`** are declared in `Cargo.toml` but never used — no `#[derive(Error)]` or `anyhow::Result` anywhere.
3. **Duplicate `EventResult` types**: `src/wasm/events.rs` exports `EventResult` (with cursor field) while `src/wasm/bindings.rs` defines a separate `EditorEventResult`. The public `EventResult` appears unused.
4. **Unused `select_tool` field**: `bindings.rs:33` holds `Option<Rc<RefCell<SelectTool>>>` that is never properly synchronized with the `active_tool: Box<dyn Tool>` field — two independent SelectTool instances.
5. **Stale configuration files**: Root `vite.config.ts` (port 3000) conflicts with canonical `web/vite.config.ts` (port 3003). Empty `web/postcss.config.js` serves no purpose.
6. **Duplicate ADR numbering**: Two files share the `005-` prefix.

## Decision

### 1. Remove `#![allow(dead_code)]`
Remove the crate-level allow and address each dead code warning individually:
- Genuinely dead code → delete
- Code needed for future features → annotate with `#[allow(dead_code)]` locally with a comment explaining why

### 2. Remove unused dependencies
Remove `thiserror` and `anyhow` from `Cargo.toml`. If error handling is needed later, re-add with actual usage.

### 3. Unify EventResult types
- Delete the unused `EventResult` from `events.rs`
- Rename `EditorEventResult` in `bindings.rs` to `EventResult` and make it the single source of truth
- Or, if both are needed, clearly differentiate their purposes

### 4. Fix select_tool dual-instance problem
Option A (Preferred): Remove the `select_tool: Option<Rc<RefCell<SelectTool>>>` field entirely. Use `active_tool.as_any_mut().downcast_mut::<SelectTool>()` when needed.
Option B: Make `active_tool` wrap the same `Rc<RefCell<SelectTool>>` instance. More complex, less benefit.

### 5. Remove stale configs
- Delete root `vite.config.ts` (CI and Playwright both use port 3003 from web/)
- Delete `web/postcss.config.js` (empty, no plugins)

### 6. Fix ADR numbering
Rename `005-package-metadata-consistency.md` to `005b-package-metadata-consistency.md`.

## Consequences

### Positive
- Compiler catches genuinely dead code going forward
- Smaller dependency tree, faster builds
- No type confusion between EventResult variants
- No silent state divergence in selection tool
- Cleaner project root

### Negative
- May expose dead code that requires decisions (delete vs keep)
- Removing `select_tool` field requires verifying no code path depends on it
- Any external references to root vite.config.ts will break (unlikely)

### Risks
- Dead code removal might accidentally remove code used only through WASM (which `cargo check` may not see). Mitigation: Test with `wasm-pack build` after changes.
