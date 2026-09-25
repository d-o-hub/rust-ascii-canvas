# Goal State: Codebase Improvement & Feature Roadmap 2026

## Status: ACHIEVED (2026-07 target met) — next horizon = release + new product work
**Updated**: 2026-09-23

---

## GOAP World State Model

### Current State (Verified 2026-09-23)

```yaml
code_quality:
  clippy_errors: 0
  clippy_warnings: 0
  dead_code_allow: false          # wasm missing_docs allows removed (F-25)
  unused_deps: []
  duplicate_types: false
  max_file_loc: 853               # web/events.ts (web/ not covered by LOC sensor — see FOLLOW_UPS)
  stale_configs: []

tests:
  rust_unit: 117/117
  rust_integration: 45+
  rust_doc: 2+
  e2e_chromium: 79
  e2e_firefox: in_ci
  e2e_webkit: in_ci
  vitest_frontend: 29
  flaky_patterns: 0
  page_object_model: true

error_handling:
  clipboard_secure_context: true
  paste_origin: true
  boundary_bugs: 0

features:
  drawing_tools: 8
  border_styles: 6
  undo_redo: true
  zoom_pan: true
  zoom_shortcuts: true             # #194
  clipboard_export: true           # ADR-041 fidelity modes
  external_paste: true             # F-17
  layers: editor                   # lock/reorder/delete/merge (F-11)
  layer_history: false             # F-13 residual — ops bypass History
  file_persistence: true
  png_export: true
  svg_export: true                 # F-10
  grid_customization: true
  preview_rendering: true          # F-15
  enhanced_text_tool: true         # F-14
  eraser_radius: true              # F-16
  light_theme: true                # F-31
  mobile_drawer_escape: true       # #195

documentation:
  adr_count: 42
  plans_refreshed: 2026-09-23
  release_runbook: true            # plans/RELEASING.md
  open_issues: 2                  # #207 (next), #199 (deferred)
  open_prs: 0
  release: v0.1.4 (2026-09-24)    # R-01/R-02/R-04 done
```

### Target State (next horizon — post-roadmap)

```yaml
features:
  layer_history: true              # F-13 residual — needs issue
  collaborative_prototype: decided # F-30 spike -> ADR

process:
  release_0_1_4: published         # R-01
  changelog_backfill: curated      # R-01
  esbuild_advisory: resolved       # R-03

harness:
  loc_sensor_covers_web: true      # candidate — see FOLLOW_UPS
```

---

## Definition of Done

### Tier 1: Code Quality

- [x] `cargo clippy --all-targets --all-features -- -D warnings`
- [x] `cargo test` (lib + integration) passing
- [x] `web/main.ts` split well under 500 LOC (230; F-20) — residual: `web/events.ts` 853 LOC, LOC sensor does not scan `web/` (see FOLLOW_UPS)
- [x] No `#![allow(missing_docs)]` on wasm (F-25)

### Tier 2: Test Reliability

- [x] Zero `waitForTimeout` flaky patterns
- [x] Page Object Model present
- [x] Vitest frontend tests (29)
- [x] Chromium E2E green (79)
- [x] Firefox + WebKit in CI (F-21)

### Tier 3: Features

- [x] Selection copy/paste + external clipboard fidelity
- [x] File persistence
- [x] PNG export
- [x] Grid customization
- [x] Layer editor (lock/reorder/delete/merge; F-11)
- [ ] Layer operation history (F-13 residual — ops bypass History)
- [x] SVG export (F-10)
- [x] Preview rendering (F-15)
- [x] Enhanced text tool (F-14)

### Tier 4: Process

- [x] Plans + FOLLOW_UPS updated for 2026-07 bundle
- [x] PR #107 merged; #21 closed (F-01)
- [x] Dependabot #99 resolved (F-03; coordinated wasm-bindgen 0.2.128, ADR-042)
- [x] Docs reconciled 2026-09-23 + release runbook ([RELEASING.md](RELEASING.md))

---

## References

- [full-recommendations-2026-07.md](full-recommendations-2026-07.md)
- [FOLLOW_UPS.md](FOLLOW_UPS.md)
- [PROJECT_STATUS.md](PROJECT_STATUS.md)
- [RELEASING.md](RELEASING.md)
- [ADR-036](ADRs/036-clipboard-fidelity-and-product-features.md)
