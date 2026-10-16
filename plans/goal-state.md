# Goal State: Codebase Improvement & Feature Roadmap 2026

## Status: ROADMAP ACHIEVED — active decisions and blocked work are tracked in FOLLOW_UPS.md
**Updated**: 2026-10-15

---

## GOAP World State Model

### Current repository/release state (remote status verified 2026-10-15; dated test metrics are labeled below)

```yaml
code_quality:
  clippy_errors: 0
  clippy_warnings: 0
  dead_code_allow: false          # wasm missing_docs allows removed (F-25)
  unused_deps: []
  duplicate_types: false
  max_file_loc: 477               # web/events.ts; web source is covered by the LOC sensor
  loc_allowlist: empty             # no oversized source files remain
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
  adr_count: 49                     # 51 files; duplicate-number variants exist for ADR-005 and ADR-040
  plans_refreshed: 2026-10-15       # v0.1.5 + #265/#270/#273 reconciliation
  release_runbook: true            # plans/RELEASING.md (rollback section added, ADR-044)
  open_issues: 0                    # #199 and #263 are closed
  open_prs: 0                       # #273 merged
  release: v0.1.5 (2026-10-15)      # R-01/R-02/R-04 done; versioned WASM asset shipped
  dependabot_alerts: 0              # R-05 pruned esbuild; #11 was the last open alert

harness:                           # ADR-044
  merge_contract_enforced: true     # ruleset: required checks + thread resolution
  required_status_checks: 3         # Codacy, CI Success, PR Readiness (merge gate)
  required_approvals: 0             # gates decide, not a human click
  merge_gate_sensor: scripts/pr-merge-gate.sh
  ruleset_drift_sensor: scripts/ruleset-check.sh   # the only guard on repo state
  delivery_loop: documented         # agents-docs/delivery.md
  adversarial_review: pr-roast      # every finding cited to official docs
  e2e_sensors_gated_in_ci: true     # L-016 — ESLint (root) + TypeScript (e2e) in web job (#220)
  codacy_backlog_readable: true      # L-017 — npm run codacy:check (agent procedure, not a gate)
  local_rule_family_parity: true     # L-017 — eslint-plugin-security in both ESLint configs
  workflow_token_permissions: least_privilege # #273 — job-scoped release/issues access
  workflow_dependency_lockfile: deferred      # ADR-050 — preview tool conflicts with full-SHA policy
  codacy_backlog: 0                  # 11 High findings fixed in #222; verified repo-level
```

### Target State (next horizon — post-roadmap)

```yaml
features:
  layer_history: true              # F-13 — shipped in #213, hardened in #216
  collaborative_prototype: decision_deferred # F-30 spike complete; new scope/issue/ADR required

process:
  release_0_1_5: published         # R-01
  changelog_backfill: curated      # R-01
  esbuild_advisory: closed         # R-05 prune landed 2026-09-28 (alert #11)
  codacy_backlog_gated: done       # R-07 — #219 sensors + #220 L-016 CI wiring

harness:
  loc_sensor_covers_web: true      # shipped via #227/#233/#241; no oversized source entries remain
  merge_contract: done             # ADR-044 (#213): ruleset + pr-merge-gate.sh
  ruleset_drift_sensor: done       # scripts/ruleset-check.sh + .github/ruleset-main.json
  gate_script_is_not_a_gate: done  # L-016 (#220): e2e sensors wired into ci.yml web job
  required_check_rule_family_parity: done  # L-017 (#223): security plugin runs locally
  repo_level_intake: done           # L-017 (#223): codacy:check, agent procedure by design
  codacy_coverage_upload: blocked   # needs a project-scoped CODACY_PROJECT_TOKEN secret
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
