# Goal State: Codebase Improvement & Feature Roadmap 2026

## Status: ACTIVE
**Updated**: 2026-04-24

---

## GOAP World State Model

### Current State (Verified 2026-04-24)

```yaml
code_quality:
  clippy_errors: 0
  clippy_warnings: 0
  dead_code_allow: true          # crate-level #![allow(dead_code)]
  unused_deps: []
  duplicate_types: false
  max_file_loc: 510              # bindings.rs improved from 722
  stale_configs: []

tests:
  rust_unit: 92/92
  rust_integration: 45/45
  rust_doc: 2/2
  e2e_chromium: 152+
  e2e_firefox: 0                 # not configured
  e2e_webkit: 0                  # not configured
  vitest_frontend: 2             # 2 tests passing
  flaky_patterns: 85             # waitForTimeout calls
  page_object_model: false

error_handling:
  unwrap_in_prod: ~4             # unwrap() calls outside test code
  boundary_bugs: 0               # SelectTool, ArrowTool, TextTool FIXED
  select_tool_duplicate: false   # RESOLVED

features:
  drawing_tools: 8
  border_styles: 6
  undo_redo: true
  zoom_pan: true
  clipboard_export: true
  selection_ops: true            # Copy/Cut/Paste/SelectAll
  layers: false
  file_persistence: false
  png_svg_export: false
  grid_customization: false
  preview_rendering: true
  ux_improvements: true          # Contextual cursors, Ctrl+A, Escape reset

documentation:
  adr_count: 35
  stale_adrs: 0                  # ALL statuses correct
  test_count_contradictions: 0   # ALL reconciled
  doc_warnings: ~52
```

### Target State

```yaml
code_quality:
  clippy_errors: 0
  clippy_warnings: 0
  dead_code_allow: false         # REMOVED
  unused_deps: []                # CLEANED
  duplicate_types: false         # RESOLVED
  max_file_loc: 500              # ALL files under limit
  stale_configs: []              # REMOVED

tests:
  rust_unit: 95+                 # new tests for layers, persistence
  rust_integration: 50+          # new integration tests
  rust_doc: 5+                   # more doc tests
  e2e_chromium: 160+
  e2e_firefox: 160+              # ENABLED
  e2e_webkit: 160+               # ENABLED
  vitest_frontend: 20+           # NEW
  flaky_patterns: 0              # ALL waitForTimeout REMOVED
  page_object_model: true        # IMPLEMENTED

error_handling:
  unwrap_in_prod: 0              # ALL replaced
  boundary_bugs: 0               # ALL fixed
  select_tool_duplicate: false   # RESOLVED

features:
  drawing_tools: 8
  border_styles: 6
  undo_redo: true
  zoom_pan: true
  clipboard_export: true
  layers: true                   # NEW
  file_persistence: true         # NEW
  png_svg_export: false          # DEFERRED (P2)
  grid_customization: false      # DEFERRED (P1)
  preview_rendering: true

documentation:
  adr_count: 35
  stale_adrs: 0                  # ALL statuses correct
  test_count_contradictions: 0   # ALL reconciled
  doc_warnings: 0                # ALL documented
```

---

## Definition of Done

### Tier 1: Code Quality (Must Have)

- [ ] `cargo clippy --all-targets --all-features -- -D warnings` — 0 errors, 0 warnings
- [ ] No `#![allow(dead_code)]` at crate level
- [ ] All source files < 500 LOC
- [x] No unused dependencies in Cargo.toml
- [x] No duplicate type definitions
- [x] No stale configuration files
- [x] `cargo test` — 100% passing
- [ ] `cargo doc` — 0 warnings

### Tier 2: Test Reliability (Must Have)

- [ ] 0 `waitForTimeout` calls in E2E tests
- [ ] Page Object Model in `e2e/pages/`
- [ ] Cross-browser: Chromium + Firefox + WebKit
- [x] Vitest unit tests for frontend TypeScript
- [ ] ASCII output verification in E2E (draw → export → assert)

### Tier 3: Robustness (Must Have)

- [ ] 0 `unwrap()` outside `#[cfg(test)]` blocks
- [x] All known bugs fixed (SelectTool boundary, ArrowTool Bresenham, TextTool position)
- [x] No duplicate tool instances

### Tier 4: New Features (Should Have)

- [ ] Layer system with visibility, locking, reordering
- [ ] File persistence (save/load with `.asc` format)
- [ ] localStorage auto-save

### Tier 5: Performance (Should Have)

- [ ] No `.to_string()` allocations in hot paths
- [ ] Sparse grid iteration
- [ ] Content-bounds caching
- [ ] Benchmark suite with `criterion`

### Tier 6: Documentation (Must Have)

- [ ] All ADR statuses accurate
- [ ] PROJECT_STATUS.md reflects actual state
- [ ] No contradictions across plan files
- [ ] All public APIs documented

---

## Success Metrics

| Metric | Current | Target | Priority |
|--------|---------|--------|----------|
| Clippy issues | 0 | 0 | Maintain |
| Source files > 500 LOC | 1 | 0 | High |
| Unused deps | 0 | 0 | High |
| `unwrap()` in prod | ~4 | 0 | High |
| `waitForTimeout` calls | 85 | 0 | High |
| Cross-browser E2E | 1/3 | 3/3 | High |
| Vitest tests | 2 | 20+ | Medium |
| Layers support | No | Yes | Medium |
| File persistence | No | Yes | Medium |
| Doc warnings | ~52 | 0 | Medium |
| Stale ADR statuses | 0 | 0 | Medium |
