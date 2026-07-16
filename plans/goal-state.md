# Goal State: Codebase Improvement & Feature Roadmap 2026

## Status: ACTIVE
**Updated**: 2026-07-16

---

## GOAP World State Model

### Current State (Verified 2026-07-16)

```yaml
code_quality:
  clippy_errors: 0
  clippy_warnings: 0
  dead_code_allow: false          # no crate-level allow (wasm modules still allow missing_docs)
  unused_deps: []                 # thiserror/anyhow cleaned historically
  duplicate_types: false          # EditorEventResult path in use
  max_file_loc: ~1300             # web/main.ts still large (F-20)
  stale_configs: []               # package metadata fixed to d-o-hub

tests:
  rust_unit: 98/98
  rust_integration: 45+
  rust_doc: 2+
  e2e_chromium: 71
  e2e_firefox: configured         # not last-gate required
  e2e_webkit: configured
  vitest_frontend: 14
  flaky_patterns: 0               # waitForTimeout eliminated
  page_object_model: true

error_handling:
  clipboard_secure_context: true
  paste_origin: true
  boundary_bugs: 0                # known tool boundary issues from 2026-03 fixed

features:
  drawing_tools: 8
  border_styles: 6
  undo_redo: true
  zoom_pan: true
  clipboard_export: true          # selection-aware + CRLF
  layers: true                    # basic add/switch/export composite
  file_persistence: true          # .asc + localStorage
  png_export: true
  svg_export: false               # F-10
  grid_customization: true
  preview_rendering: false        # F-15 / ADR-011

documentation:
  adr_count: 36
  plans_refreshed: 2026-07-16
  follow_ups_file: true           # plans/FOLLOW_UPS.md
  open_issue_21: closed           # PR #107 merged
  pr_107_merged: true
  composite_pixel_render: true    # F-12 done
```

### Target State (next horizon)

```yaml
code_quality:
  max_file_loc: 500               # finish main.ts split (F-20)
  wasm_missing_docs_allow: false  # F-25

tests:
  e2e_firefox: in_ci              # F-21
  e2e_webkit: in_ci
  vitest_frontend: 25+

features:
  layers: full                    # lock, reorder, history (F-11, F-13)
  svg_export: true                # F-10
  preview_rendering: true         # F-15
  enhanced_text_tool: true        # F-14

process:
  issue_21: closed
  dependabot_99: triaged          # F-03 remainder
```

---

## Definition of Done

### Tier 1: Code Quality

- [x] `cargo clippy --all-targets --all-features -- -D warnings`
- [x] `cargo test` (lib + integration) passing
- [ ] All source files < 500 LOC (main.ts still over — F-20)
- [ ] No `#![allow(missing_docs)]` on wasm (F-25)

### Tier 2: Test Reliability

- [x] Zero `waitForTimeout` flaky patterns
- [x] Page Object Model present
- [x] Vitest frontend tests
- [x] Chromium E2E green (71)
- [ ] Firefox + WebKit in CI (F-21)

### Tier 3: Features

- [x] Selection copy/paste + external clipboard fidelity
- [x] File persistence
- [x] PNG export
- [x] Grid customization
- [x] Basic layers
- [ ] SVG export (F-10)
- [ ] Full layer system (F-11–F-13)
- [ ] Preview rendering (F-15)
- [ ] Enhanced text tool (F-14)

### Tier 4: Process

- [x] Plans + FOLLOW_UPS updated for 2026-07 bundle
- [x] PR #107 merged; #21 closed (F-01)
- [ ] Dependabot #99 resolved (F-03; #98 already merged)

---

## References

- [full-recommendations-2026-07.md](full-recommendations-2026-07.md)
- [FOLLOW_UPS.md](FOLLOW_UPS.md)
- [PROJECT_STATUS.md](PROJECT_STATUS.md)
- [ADR-036](ADRs/036-clipboard-fidelity-and-product-features.md)
