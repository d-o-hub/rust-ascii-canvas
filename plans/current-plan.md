# Current Plan: GOAP Codebase Improvement & Feature Roadmap 2026

## Status: ACTIVE — post recommendations bundle

**Created**: 2026-03-03  
**Last updated**: 2026-07-16  
**Supersedes (partially)**: Production-readiness-only focus; March 2026 world-state snapshot  
**Methodology**: Goal-Oriented Action Planning (GOAP) with ADRs  
**Latest execution**: [full-recommendations-2026-07.md](full-recommendations-2026-07.md)  
**Backlog**: [FOLLOW_UPS.md](FOLLOW_UPS.md)

---

## World State (Current — Verified 2026-07-16)

| Property | Value |
|----------|-------|
| clippy (`-D warnings`) | ✅ 0 errors |
| rust_lib_tests | **98** passing |
| rust_integration + doc | passing |
| e2e_chromium | **71** passing |
| e2e_firefox / webkit | projects configured; CI gate optional (F-21) |
| vitest_frontend | **14** passing |
| waitForTimeout in e2e | **0** |
| page_object_model | yes (`e2e/pages/EditorPage.ts`) |
| selection_copy_paste | ✅ implemented (ADR-009 / #21) |
| file_persistence | ✅ `.asc` + localStorage (ADR-027) |
| png_export | ✅ (ADR-013 partial) |
| svg_export | ❌ (F-10) |
| grid_customization | ✅ UI + responsive (ADR-012) |
| layers | ✅ basic (ADR-026); polish F-11–F-13 |
| main.ts modules | partial split; main still large (F-20) |
| package_metadata | ✅ d-o-hub URLs |
| open_issue_#21 | fixed in code; PR/close pending (F-01) |
| open_dependabot | #98, #99 (F-03) |

---

## Goals progress

| Goal | Priority | Status | Notes |
|------|----------|--------|-------|
| 1 Code quality & maintainability | HIGH | 🟡 Ongoing | Clippy clean; main.ts / docs still follow-ups |
| 2 Test reliability & coverage | HIGH | 🟢 Strong | waitForTimeout gone; vitest; chromium 71; multi-browser CI open |
| 3 Robustness & error handling | HIGH | 🟡 Partial | Clipboard hardened; more unwrap audit optional |
| 4 Layer system | MEDIUM | 🟢 Basic done | F-11–F-13 for full ADR-026 |
| 5 File persistence | MEDIUM | ✅ Done | ADR-027 |
| 6 Collaborative editing | LOW | ❌ Not started | F-30 |
| 7 Performance optimization | MEDIUM | 🟡 Partial | Dirty-rect exists; ADR-028 backlog F-28 |
| 8 Developer experience / docs | MEDIUM | 🟡 Improved | Plans refreshed 2026-07-16; ADR sweep F-23 |

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
| 1 | F-01 | PR + close #21 | release |
| 2 | F-02 | Manual paste QA (Windows Notepad) | QA |
| 3 | F-03 | Dependabot #98/#99 | deps |
| 4 | F-12 | Composite layer **rendering** | product |
| 5 | F-10 | SVG export | product |
| 6 | F-20 | Finish main.ts split | frontend |
| 7 | F-21 | Multi-browser CI E2E | CI |

---

## Phase checklist (historical roadmap — status)

### Phase 1: Code hygiene (ADR-022)
- [x] Major dead-code / bindings split work (historical)  
- [ ] Full ADR-022 checklist audit if still needed  

### Phase 2: Test reliability (ADR-024, 034, 035)
- [x] waitForTimeout elimination  
- [x] POM introduced  
- [x] Vitest tests present  
- [ ] Firefox/WebKit CI (F-21)  

### Phase 3: Features
- [x] Selection copy/paste (009)  
- [x] Grid customization (012)  
- [x] File persistence (027)  
- [x] PNG export (013 partial)  
- [x] Layers basic (026 partial)  
- [ ] SVG (F-10)  
- [ ] Enhanced text (010)  
- [ ] Preview tint (011)  

### Phase 4: Docs
- [x] 2026-07-16 status / follow-ups / recommendations plan  
- [ ] Full ADR status reconciliation (F-23)  

---

## Definition of done for “recommendations complete”

- [x] #21 root causes fixed in code  
- [x] Unit + chromium E2E green  
- [x] Plans document progress + follow-ups  
- [ ] PR merged; issue #21 closed (F-01)  
- [ ] Manual multi-OS paste confirmation (F-02)  
