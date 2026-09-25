# Current Plan: GOAP Codebase Improvement & Feature Roadmap 2026

## Status: ACTIVE — next cycle planned 2026-09-25 (F-13/#207 → R-05; #199 deferred)

**Created**: 2026-03-03  
**Last updated**: 2026-09-25 (GOAP cycle planned by agent swarm)  
**Supersedes (partially)**: Production-readiness-only focus; March 2026 world-state snapshot  
**Methodology**: Goal-Oriented Action Planning (GOAP) with ADRs  
**Latest execution**: [full-recommendations-2026-07.md](full-recommendations-2026-07.md)  
**Backlog**: [FOLLOW_UPS.md](FOLLOW_UPS.md) · **Release**: [RELEASING.md](RELEASING.md)

---

## Latest: PR #146 Post-Merge Remediation (2026-07-27)

**Swarm-based review** identified 5 issues in merged PR #146 (F-22 WASM types):

| # | Finding | Severity | ADR |
|---|---------|----------|-----|
| 1 | Redundant dual import + alias in `main.ts` | Low | 039 |
| 2 | Type-inaccurate `textCursorPosition` (`number[]` vs `Int32Array`) | Medium | 039 |
| 3 | Dead ternary in `render.ts` textPos assignment | Low | 039 |
| 4 | `tsc --noEmit` mixed into `lint` script — should be separate `typecheck` | Medium | 039 |
| 5 | Lost interface contract documentation | Low | 039 |

**Execution plan**: See ADR-039 for detailed fix steps. Priority: 4 → 2 → 1 → 3 → 5.

---

## World State (Current — Verified 2026-07-16)

| Property | Value |
|----------|-------|
| clippy (`-D warnings`) | ✅ 0 errors |
| rust_lib_tests | **117** passing |
| rust_integration + doc | passing |
| e2e_chromium | **79** tests (4 files) |
| e2e_firefox / webkit | ✅ CI matrix (F-21) |
| vitest_frontend | **29** passing |
| waitForTimeout in e2e | **0** |
| page_object_model | yes (`e2e/pages/EditorPage.ts`) |
| selection_copy_paste | ✅ implemented (ADR-009 / #21) |
| clipboard_export_fidelity | ✅ ADR-041 (#176/#177) |
| file_persistence | ✅ `.asc` + localStorage (ADR-027) |
| png_export | ✅ (ADR-013) |
| svg_export | ✅ (F-10) |
| grid_customization | ✅ UI + responsive (ADR-012) |
| layers | ✅ editor (F-11); **history F-13 not implemented** |
| light_theme | ✅ (F-31) |
| main.ts modules | ✅ split; `main.ts` 230 LOC (F-20) |
| package_metadata | ✅ d-o-hub URLs |
| release | **v0.1.4** released 2026-09-24; R-01/R-02/R-04 done ([RELEASING.md](RELEASING.md)) |
| open_issues / PRs | 2 (#207 next, #199 deferred) / 0 |
| open_dependabot | R-03 (low, dev-only esbuild advisory) |

---

## Goals progress

| Goal | Priority | Status | Notes |
|------|----------|--------|-------|
| 1 Code quality & maintainability | HIGH | ✅ Done | Clippy/fmt clean; `main.ts` 230 LOC; architecture + LOC sensors in gate |
| 2 Test reliability & coverage | HIGH | ✅ Done | waitForTimeout gone; vitest 29; chromium 79 (local list) + firefox/webkit CI matrix; POM |
| 3 Robustness & error handling | HIGH | 🟡 Partial | Clipboard hardened; more unwrap audit optional |
| 4 Layer system | MEDIUM | ✅ Editor done | F-11 editor shipped; **F-13 history residual** (fresh issue needed) |
| 5 File persistence | MEDIUM | ✅ Done | ADR-027 |
| 6 Collaborative editing | LOW | 🔵 Spike done | Prototype decision pending = new work |
| 7 Performance optimization | MEDIUM | ✅ Done | Dirty-rect pixel buffer (ADR-028 / F-28) |
| 8 Developer experience / docs | MEDIUM | ✅ Done | ADR audit F-23 done; docs reconciled 2026-09-23; release runbook added |

---

## Active workstreams

### Just completed (do not re-plan unless regression)

1. Issue #21 clipboard fidelity  
2. Persistence + PNG + grid UI + basic layers  
3. Frontend feature modules + metadata  
4. E2E stability fixes (loading wait, text shortcuts, responsive)

### Next actions (priority order)

| Order | ID | Action | Owner hint |
|-------|-----|--------|------------|
| 1 | F-13 | Layer-command history: core model → wasm/UI → E2E (issue #207, ADR-043) | core/product |
| 2 | R-05 | Dependency refresh: prune unused optional peers (esbuild, jsdom), review the 28 bumps → closes alert #11 | deps |
| 3 | R-06 | Decide the vite dev-server host default (currently 0.0.0.0) | frontend/security |
| 4 | #199 | do-harness adoption — deferred until upstream #235–#237 ship pinned | harness |

---

## Phase checklist (historical roadmap — status)

### Phase 1: Code hygiene (ADR-022)
- [x] Major dead-code / bindings split work (historical)  
- [ ] Full ADR-022 checklist audit if still needed  

### Phase 2: Test reliability (ADR-024, 034, 035)
- [x] waitForTimeout elimination  
- [x] POM introduced  
- [x] Vitest tests present  
- [x] Firefox/WebKit CI (F-21)  

### Phase 3: Features
- [x] Selection copy/paste (009)  
- [x] Grid customization (012)  
- [x] File persistence (027)  
- [x] PNG export (013 partial)  
- [x] Layers basic (026 partial)  
- [x] SVG (F-10)  
- [x] Enhanced text (010)  
- [x] Preview tint (011)  

### Phase 4: Docs
- [x] 2026-07-16 status / follow-ups / recommendations plan  
- [x] Full ADR status reconciliation (F-23)  
- [x] 2026-09-23 reconciliation: release runbook + L-007; docs match code/issue state

---

## Definition of done for “recommendations complete”

- [x] #21 root causes fixed in code  
- [x] Unit + chromium E2E green  
- [x] Plans document progress + follow-ups  
- [x] PR merged; issue #21 closed (F-01)  
- [x] Manual multi-OS paste confirmation (F-02)  
