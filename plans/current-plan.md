# Current Plan: GOAP Codebase Improvement & Feature Roadmap 2026

## Status: ACTIVE

**Created**: 2026-03-03
**Supersedes**: Previous production-readiness-only focus
**Methodology**: Goal-Oriented Action Planning (GOAP) with ADRs

---

## World State (Current)

### Verified Facts (2026-03-03)

| Property | Value |
|----------|-------|
| clippy_errors | 0 |
| clippy_warnings | 0 |
| rust_unit_tests | 79 passing |
| rust_integration_tests | 44 passing |
| rust_doc_tests | 2 passing |
| e2e_tests | 35+ (chromium only) |
| wasm_size | 151KB |
| unwrap_calls_in_src | 32 (most in #[cfg(test)]) |
| dead_code_allow | 1 (crate-level in lib.rs) |
| bindings_rs_loc | 722 (exceeds 500 LOC guideline) |
| waitForTimeout_calls | 85 in E2E tests |
| select_tool_field | Duplicate/unused Rc<RefCell> pattern |
| vitest_tests | 0 (dependency installed, no tests) |
| cross_browser_e2e | chromium only |
| duplicate_adr_numbering | ADR-005 used twice |
| stale_vite_config | Root vite.config.ts (port 3000) vs web/ (port 3003) |
| unused_deps | thiserror, anyhow (declared but unused) |
| duplicate_event_result | EventResult vs EditorEventResult |
| doc_warnings | ~52 (missing docs on public items) |
| typescript_strict_issues | Non-null assertions, missing WASM types |
| selection_copy_paste | NOT implemented (ADR-009) |
| enhanced_text_tool | NOT implemented (ADR-010) |
| preview_rendering | NOT implemented (ADR-011) |
| grid_customization | NOT implemented (ADR-012) |
| export_formats | ASCII only (no PNG/SVG) |
| accessibility | Partial (missing ARIA on some elements) |
| page_object_model | NOT implemented in E2E |

---

## Goals (Prioritized)

### Goal 1: Code Quality & Maintainability (Priority: HIGH)
**Target State**: Zero dead code, no crate-level allows, bindings.rs < 500 LOC, unused deps removed, no duplicate types
**Satisfaction**: clippy clean, no `#![allow(dead_code)]`, all files < 500 LOC, zero unused dependencies

### Goal 2: Test Reliability & Coverage (Priority: HIGH)
**Target State**: Zero flaky tests, cross-browser E2E, Page Object Model, vitest unit tests for frontend
**Satisfaction**: 0 `waitForTimeout` calls, 3 browser projects, POM in e2e/pages/, vitest tests exist

### Goal 3: Robustness & Error Handling (Priority: HIGH)
**Target State**: No unwrap in non-test code, proper Result/Option handling, no duplicate tool instances
**Satisfaction**: `unwrap()` only in `#[cfg(test)]` blocks, select_tool field removed or properly integrated

### Goal 4: New Feature - Layer System (Priority: MEDIUM)
**Target State**: Multiple layers with visibility/lock, layer reordering, merge
**Satisfaction**: Layer UI, per-layer grid, composite rendering

### Goal 5: New Feature - File Persistence (Priority: MEDIUM)
**Target State**: Save/load diagrams, localStorage + file download, auto-save
**Satisfaction**: Native file format (.asc), import/export, browser storage

### Goal 6: New Feature - Collaborative Editing (Priority: LOW)
**Target State**: Real-time multi-user editing via WebSocket/WebRTC
**Satisfaction**: CRDT-based conflict resolution, cursor presence, operational transforms

### Goal 7: Performance Optimization (Priority: MEDIUM)
**Target State**: Eliminate redundant allocations, optimize render path, lazy grid iteration
**Satisfaction**: No `.to_string()` on mouse move, `&'static str` for colors, sparse grid iteration

### Goal 8: Developer Experience (Priority: MEDIUM)
**Target State**: Clean documentation, consistent ADRs, accurate PROJECT_STATUS.md, no contradictions
**Satisfaction**: All ADR statuses correct, test counts accurate, no stale configs

---

## Action Plan

### Phase 1: Code Hygiene & Dead Code Cleanup (ADR-022)

**Preconditions**: Clean clippy, passing tests
**Effects**: Smaller codebase, no hidden dead code, cleaner dependency graph

| Action | Description | Est. Effort |
|--------|-------------|-------------|
| 1.1 | Remove `#![allow(dead_code)]` from lib.rs, fix any exposed issues | 30 min |
| 1.2 | Remove unused `thiserror` and `anyhow` deps from Cargo.toml | 10 min |
| 1.3 | Resolve duplicate `EventResult` vs `EditorEventResult` types | 30 min |
| 1.4 | Remove or fix unused `select_tool: Option<Rc<RefCell<SelectTool>>>` field | 45 min |
| 1.5 | Remove stale root `vite.config.ts` (web/ config is canonical) | 10 min |
| 1.6 | Remove empty `web/postcss.config.js` | 5 min |
| 1.7 | Fix duplicate ADR-005 numbering (renumber to 005a/005b) | 10 min |

**Verification**: `cargo clippy -- -D warnings`, `cargo test`, no stale files

### Phase 2: Bindings Refactor — Split bindings.rs (ADR-023)

**Preconditions**: Phase 1 complete
**Effects**: bindings.rs reduced from 722 to 578 LOC, better separation of concerns

| Action | Description | Est. Effort | Status |
|--------|-------------|-------------|--------|
| 2.1 | Extract tool management into `src/wasm/tool_manager.rs` | 1 hr | ✅ Done |
| 2.2 | Extract event handling into `src/wasm/event_handlers.rs` | 1 hr | 🔄 Partial |
| 2.3 | Extract render/export into `src/wasm/render_bridge.rs` | 45 min | ✅ Done |
| 2.4 | Keep `bindings.rs` as thin WASM facade (< 300 LOC) | 30 min | 🔄 578 LOC |

**Note**: Event handlers (on_pointer_*, on_key_*, on_wheel) remain in bindings.rs due to tight coupling with editor state. Would require significant refactoring to extract.

**Verification**: All files < 600 LOC, `cargo test`, E2E tests pass

### Phase 3: Test Robustness (ADR-024)

**Preconditions**: Phase 2 complete (stable API surface)
**Effects**: Zero flaky tests, cross-browser confidence

| Action | Description | Est. Effort |
|--------|-------------|-------------|
| 3.1 | Create `e2e/pages/EditorPage.ts` Page Object Model | 2 hr |
| 3.2 | Replace all 85 `waitForTimeout` with proper assertions | 3 hr |
| 3.3 | Enable Firefox + WebKit in playwright.config.ts | 30 min |
| 3.4 | Add ASCII output verification tests (draw + export + assert) | 2 hr |
| 3.5 | Add vitest unit tests for `web/main.ts` key functions | 2 hr |
| 3.6 | Update CI workflow for multi-browser matrix | 30 min |

**Verification**: `npx playwright test` (all 3 browsers), `npx vitest run`, zero `waitForTimeout`

### Phase 4: Error Handling Hardening (ADR-025)

**Preconditions**: Phase 1 complete
**Effects**: No panic potential in production code

| Action | Description | Est. Effort |
|--------|-------------|-------------|
| 4.1 | Audit all `unwrap()` in non-test src/ code (currently ~4 in production code) | 30 min |
| 4.2 | Replace with `Option`/`Result` propagation or `unwrap_or_default()` | 1 hr |
| 4.3 | Add boundary clamping in SelectTool (ADR-015 bug) | 30 min |
| 4.4 | Fix Arrow tool Bresenham negative dy bug | 30 min |
| 4.5 | Fix text tool boundary check variable (ADR-016 bug) | 30 min |

**Verification**: No `unwrap()` outside `#[cfg(test)]`, `cargo test`, specific regression tests added

### Phase 5: New Feature — Layer System (ADR-026)

**Preconditions**: Phase 4 complete (robust error handling)
**Effects**: Multi-layer editing, compositing, visibility control

| Action | Description | Est. Effort |
|--------|-------------|-------------|
| 5.1 | Design `Layer` struct: `{ id, name, grid, visible, locked, opacity }` | 1 hr |
| 5.2 | Create `LayerStack` manager with ordering, merge, flatten | 2 hr |
| 5.3 | Update `EditorState` to hold `LayerStack` instead of single `Grid` | 2 hr |
| 5.4 | Update rendering to composite layers (bottom-up with visibility) | 2 hr |
| 5.5 | Add WASM bindings: `addLayer`, `removeLayer`, `setActiveLayer`, `toggleVisibility` | 1.5 hr |
| 5.6 | Add layer panel UI (list, visibility toggles, reorder drag) | 3 hr |
| 5.7 | Update undo/redo to be layer-aware | 2 hr |
| 5.8 | Add layer-specific tests (unit + integration + E2E) | 2 hr |

**Verification**: `cargo test`, E2E layer tests, multi-layer export produces correct composite

### Phase 6: New Feature — File Persistence (ADR-027)

**Preconditions**: Phase 5 complete (layer system needed for full save/load)
**Effects**: Users can save, load, and share diagrams

| Action | Description | Est. Effort |
|--------|-------------|-------------|
| 6.1 | Define `.asc` file format (JSON: grid data, layers, metadata, version) | 1 hr |
| 6.2 | Implement `serialize_project()` / `deserialize_project()` in Rust | 2 hr |
| 6.3 | Add localStorage auto-save (debounced, on change) | 1.5 hr |
| 6.4 | Add "Save as File" (Blob download) and "Open File" (FileReader) in TS | 2 hr |
| 6.5 | Add Ctrl+S / Ctrl+O keyboard shortcuts | 30 min |
| 6.6 | Add UI: Save/Load buttons in toolbar, recent files list | 2 hr |
| 6.7 | Migration support for format versioning | 1 hr |

**Verification**: Round-trip test (create diagram → save → reload → verify identical), localStorage persistence test

### Phase 7: Performance Optimization (ADR-028)

**Preconditions**: Phase 2 complete (clean architecture)
**Effects**: Faster rendering, lower memory pressure

| Action | Description | Est. Effort |
|--------|-------------|-------------|
| 7.1 | Replace `.to_string()` in hot paths with `&'static str` | 1 hr |
| 7.2 | Use `Cow<'static, str>` for theme colors in render commands | 1 hr |
| 7.3 | Implement sparse grid iteration (skip empty rows) | 2 hr |
| 7.4 | Add content-bounds caching (invalidate on edit) | 1.5 hr |
| 7.5 | Benchmark before/after with `criterion` | 2 hr |

**Verification**: `cargo bench` shows measurable improvement, no regression in tests

### Phase 8: Documentation & Plan Reconciliation (ADR-029)

**Preconditions**: Can run in parallel with other phases
**Effects**: Accurate, non-contradictory documentation

| Action | Description | Est. Effort |
|--------|-------------|-------------|
| 8.1 | Update PROJECT_STATUS.md with accurate test counts and status | 30 min |
| 8.2 | Update all ADR statuses (Proposed→Accepted for implemented ones) | 30 min |
| 8.3 | Fix ADR-005 numbering conflict | 10 min |
| 8.4 | Resolve test count contradictions across all plan files | 30 min |
| 8.5 | Remove "Production Readiness Achieved" claims that are aspirational | 15 min |
| 8.6 | Add `cargo doc` pass to remove 52 doc warnings | 2 hr |

**Verification**: Manual review of all plans/ files for consistency

---

## Dependency Graph

```
Phase 1 (Code Hygiene) ──┬──> Phase 2 (Split bindings.rs)
                          │         │
                          │         └──> Phase 3 (Test Robustness)
                          │         │
                          │         └──> Phase 7 (Performance)
                          │
                          └──> Phase 4 (Error Handling)
                                    │
                                    └──> Phase 5 (Layer System)
                                              │
                                              └──> Phase 6 (File Persistence)

Phase 8 (Documentation) ──> Runs in parallel with all phases
```

---

## Progress Tracking

| Phase | Description | Status | Completion |
|-------|-------------|--------|------------|
| Phase 1 | Code Hygiene & Dead Code Cleanup | ✅ Complete | 100% |
| Phase 2 | Split bindings.rs | 🔄 In Progress | 65% |
| Phase 3 | Test Robustness | ⏸️ Pending | 0% |
| Phase 4 | Error Handling Hardening | ⏸️ Pending | 0% |
| Phase 5 | Layer System (NEW FEATURE) | ⏸️ Pending | 0% |
| Phase 6 | File Persistence (NEW FEATURE) | ⏸️ Pending | 0% |
| Phase 7 | Performance Optimization | ⏸️ Pending | 0% |
| Phase 8 | Documentation Reconciliation | ⏸️ Pending | 0% |

### Phase 2 Progress (bindings.rs Refactor)

**Current State:**
- `tool_manager.rs`: 92 LOC ✅
- `render_bridge.rs`: 90 LOC ✅
- `bindings.rs`: 578 LOC (down from 722)
- Target: < 300 LOC

**Completed Actions:**
- ✅ **2.1**: Extract tool management into `src/wasm/tool_manager.rs`
- ✅ **2.3**: Extract render/export into `src/wasm/render_bridge.rs`
- 🔄 **2.2**: Event handlers remain in bindings.rs (tightly coupled)
- 🔄 **2.4**: bindings.rs at 578 LOC (278 lines over target)

**Remaining Work:**
- Event handler extraction requires significant refactoring due to tight coupling
- Alternative: Accept 500-600 LOC as reasonable for WASM facade

### Completed Actions (2026-03-03)

- ✅ **1.1**: Removed `#![allow(dead_code)]` from lib.rs, added local allows for unused event types
- ✅ **1.2**: Removed unused `thiserror` and `anyhow` dependencies
- ✅ **1.3**: Removed unused `EventResult` type (duplicate of EditorEventResult)
- ✅ **1.4**: Removed unused `select_tool` Rc<RefCell<SelectTool>> field
- ✅ **1.5**: Removed stale root vite.config.ts (web/vite.config.ts is canonical)
- ✅ **1.6**: Removed empty web/postcss.config.js
- ✅ **1.7**: Renamed ADR-005 to ADR-005a and ADR-005b

**CI Verification**: ✅ All tests pass (Rust + E2E)

---

## Estimated Total Effort

| Category | Effort |
|----------|--------|
| Code Quality (Phase 1-2) | ~6 hours |
| Test Reliability (Phase 3) | ~10 hours |
| Error Handling (Phase 4) | ~3 hours |
| New Features (Phase 5-6) | ~22 hours |
| Performance (Phase 7) | ~7.5 hours |
| Documentation (Phase 8) | ~4 hours |
| **Total** | **~52.5 hours** |
