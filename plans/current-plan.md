# Current Plan: GOAP Codebase Improvement & Feature Roadmap 2026

## Status: ACTIVE — Codacy parity landed (#222 + #223/L-017) and R-06 shipped; next is R-08 (L-013 npm audit)

**Created**: 2026-03-03  
**Last updated**: 2026-09-29 (plans sync: #222 fixed the 11 Codacy findings; #223 closes L-017)  
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
| 1 | F-13 | ✅ Done — layer-command history shipped (issue #207, ADR-043; #212 + #216) | core/product |
| 2 | R-05 | ✅ Done — dependency refresh pruned the unused optional peers (esbuild, jsdom) → alert #11 closed, L-006 trigger gone | deps |
| 3 | R-07 | ✅ Done — 11 residual `security_detect-*` findings fixed in **#222** (no suppressions; one of them exposed a vacuous assertion in `canvas.spec.ts`); rule-family parity, repo-level intake and the Biome/Qwik misconfiguration closed in **#223** under **L-017** / ADR-045 | harness/ci | — #219/#220 fixed the `e2e/` *scope* blind spot (L-014 → L-016), but "41 → 0 actionable" was wrong: Codacy's PR analysis is **diff-scoped** and neither local ESLint config enables `eslint-plugin-security`, so **11 High `security_detect-*` findings** survived on the repo-level backlog. Findings fixed 2026-09-29; the rule-family parity + repo-level intake gap is **L-017** (open) | harness/ci |
| 4 | R-06 | ✅ Done — dev server binds `127.0.0.1` with `strictPort`; `pnpm run dev:lan` is the documented LAN opt-in. Justified by three 2026 Vite advisories that list dev-server network exposure as a precondition, and by a stale server found on 3004 | frontend/security |
| 5 | R-08 | ✅ Done — **L-013** closed in #225: `pnpm audit` for both lockfiles as a web-gated `Security Audit (npm)` CI job, plus **L-018** (the merge-gate coherence check is now bidirectional). Current state: both lockfiles clean at `high` | harness/security |
| 6 | R-09 | Pay the LOC debt the new ratchet pins: `web/events.ts` (850 — `setupEventListeners` is 430 of them) and `web/ui.ts` (540), then drop their `.loc-allowlist` entries. Sensor itself shipped in #227 (L-020) | frontend |
: the `Security audit` sensor is `cargo audit` only — no npm advisory sensor exists for either lockfile, and the 2026 Vite dev-server advisories (GHSA-v2wj-q39q-566r, GHSA-p9ff-h696-f583, CVE-2026-53571) all list *"exposes the dev server to the network"* as their precondition, which is what R-06 removes | harness/security |
| 6 | #199 | do-harness adoption — deferred until upstream #235–#237 ship pinned | harness |

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
