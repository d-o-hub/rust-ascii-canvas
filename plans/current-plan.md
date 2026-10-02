# Current Plan: GOAP Codebase Improvement & Feature Roadmap 2026

## Status: active work is tracked in FOLLOW_UPS.md

Use [the active backlog](FOLLOW_UPS.md#active-backlog) and
[the 2026-09-30 implementation plan](recommendations-implementation-2026-09-30.md).
The 2026-09-30 audit shipped as #228–#233; the remaining truthful-sensors +
skills candidate is ported onto `refactor/fail-closed-sensors` (PR pending).
F-13, R-06 and the R-08 npm audit inventory already shipped; do not re-plan them.

**Created**: 2026-03-03  
**Last updated**: 2026-10-02 (audit shipped as #228–#233; sensors port recorded)
**Supersedes (partially)**: Production-readiness-only focus; March 2026 world-state snapshot  
**Methodology**: Goal-Oriented Action Planning (GOAP) with ADRs  
**Latest execution**: [full-recommendations-2026-07.md](full-recommendations-2026-07.md)  
**Backlog**: [FOLLOW_UPS.md](FOLLOW_UPS.md) · **Release**: [RELEASING.md](RELEASING.md)

---

## Historical: PR #146 Post-Merge Remediation (2026-07-27)

**Swarm-based review** identified 5 issues in merged PR #146 (F-22 WASM types):

| # | Finding | Severity | ADR |
|---|---------|----------|-----|
| 1 | Redundant dual import + alias in `main.ts` | Low | 039 |
| 2 | Type-inaccurate `textCursorPosition` (`number[]` vs `Int32Array`) | Medium | 039 |
| 3 | Dead ternary in `render.ts` textPos assignment | Low | 039 |
| 4 | `tsc --noEmit` mixed into `lint` script — should be separate `typecheck` | Medium | 039 |
| 5 | Lost interface contract documentation | Low | 039 |

**Historical execution plan**: ADR-039 recorded priority 4 → 2 → 1 → 3 → 5.
This is not the active queue; any residual requires fresh code verification.

---

## Historical world-state snapshot (2026-07–09; not re-verified today)

| Property | Value |
|----------|-------|
| clippy (`-D warnings`) | ✅ 0 errors |
| rust_lib_tests | **117** passing |
| rust_integration + doc | passing |
| e2e_chromium | **91** tests (4 files) |
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
| layers | ✅ editor (F-11) + **undoable layer ops (F-13, ADR-043)** |
| light_theme | ✅ (F-31) |
| main.ts modules | ✅ split; `main.ts` 230 LOC (F-20) |
| package_metadata | ✅ d-o-hub URLs |
| release | **v0.1.4** released 2026-09-24; R-01/R-02/R-04 done; R-07 Codacy backlog done and gated in CI (#219 + #220/L-016) ([RELEASING.md](RELEASING.md)) |
| open_issues / PRs | 1 (#199 deferred) / 0 |
| open_dependabot | 0 (alert #11 closed by R-05 prune) |

---

## Goals progress

| Goal | Priority | Status | Notes |
|------|----------|--------|-------|
| 1 Code quality & maintainability | HIGH | ✅ Done | Clippy/fmt clean; `main.ts` 230 LOC; architecture + LOC sensors in gate |
| 2 Test reliability & coverage | HIGH | ✅ Done | waitForTimeout gone; vitest 29; chromium 79 (local list) + firefox/webkit CI matrix; POM |
| 3 Robustness & error handling | HIGH | 🟡 Partial | Clipboard hardened; more unwrap audit optional |
| 4 Layer system | MEDIUM | ✅ Shipped | F-11 editor and F-13 history shipped (#212 + #216, ADR-043); no residual feature issue. |
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

### Next actions

Only [FOLLOW_UPS.md — Active backlog](FOLLOW_UPS.md#active-backlog) is maintained
as the priority queue. It separates shipped work, unmerged candidates, queued
decisions and credential-blocked work. Completion history and the Codacy L-017
correction remain in that document; old test counts are not fresh gate evidence.

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
