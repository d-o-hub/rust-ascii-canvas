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
  e2e_chromium: 91                 # grew with F-13 layer-history specs
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
  layer_history: true              # F-13 — ops recorded in History (ADR-043)
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
  adr_count: 44                     # 46 files, but 005 and 040 are each duplicated
  plans_refreshed: 2026-09-27
  release_runbook: true            # plans/RELEASING.md (rollback section added, ADR-044)
  open_issues: 1                    # #199 (deferred)
  open_prs: 0
  release: v0.1.4 (2026-09-24)      # R-01/R-02/R-04 done
  dependabot_alerts: 0              # R-05 pruned esbuild; #11 was the last open alert

harness:                           # ADR-044
  merge_contract_enforced: true     # ruleset: required checks + thread resolution
  required_status_checks: 3         # Codacy, CI Success, PR Readiness (merge gate)
  required_approvals: 0             # gates decide, not a human click
  merge_gate_sensor: scripts/pr-merge-gate.sh
  ruleset_drift_sensor: scripts/ruleset-check.sh   # the only guard on repo state
  delivery_loop: documented         # agents-docs/delivery.md
  adversarial_review: pr-roast      # every finding cited to official docs
```

### Target State (next horizon — post-roadmap)

```yaml
features:
  layer_history: true              # F-13 — shipped in #212, hardened in #216
  collaborative_prototype: decided # F-30 spike -> ADR

process:
  release_0_1_4: published         # R-01
  changelog_backfill: curated      # R-01
  esbuild_advisory: closed         # R-05 prune landed 2026-09-28 (alert #11)

harness:
  loc_sensor_covers_web: true      # candidate — see FOLLOW_UPS
  merge_contract: done             # ADR-044 (#213): ruleset + pr-merge-gate.sh
  ruleset_drift_sensor: done       # scripts/ruleset-check.sh + .github/ruleset-main.json
```

### Definition of Done — Tier 5: Harness merge contract (ADR-044, #213)

- [x] Ruleset requires `CI Success` + `PR Readiness (merge gate)`, not just Codacy
- [x] Review-thread resolution required; `required_approving_review_count: 0`
- [x] `scripts/pr-merge-gate.sh` mirrors the contract locally (read-only, never merges)
- [x] `CI Success` fails on `cancelled` and depends on `changes`
- [x] `quality-gates.sh` §2d fails if `ci-success.needs` drifts
- [x] Ruleset made reviewable + restorable (`ruleset-check.sh`, snapshot in git)
- [x] Delivery loop documented with a machine-checkable exit criterion per stage
- [x] Adversarial review (`pr-roast`) found 3 blockers + 4 majors in the first
      commit; all fixed — green gates caught none of them

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
- [x] Chromium E2E green (91)
- [x] Firefox + WebKit in CI (F-21)

### Tier 3: Features

- [x] Selection copy/paste + external clipboard fidelity
- [x] File persistence
- [x] PNG export
- [x] Grid customization
- [x] Layer editor (lock/reorder/delete/merge; F-11)
- [x] Layer operation history (F-13 — ops recorded in History, ADR-043)
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
