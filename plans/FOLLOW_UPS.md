# Follow-ups Backlog

**Updated**: 2026-07-16  
**Source**: Full recommendations bundle (issue #21 + post-merge analysis)  
**Primary plan**: [full-recommendations-2026-07.md](full-recommendations-2026-07.md)

Use this list for prioritization. Mark items done in-place and mirror major completions into `PROJECT_STATUS.md`.  
**GitHub issues** track open work (numbers below).

---

## P0 — Ship / close the loop

| ID | Status | Issue | Notes |
|----|--------|-------|--------|
| **F-01** | ✅ | — | [PR #107](https://github.com/d-o-hub/rust-ascii-canvas/pull/107) merged; #21 closed |
| **F-02** | open | [#108](https://github.com/d-o-hub/rust-ascii-canvas/issues/108) | Manual clipboard QA across editors |
| **F-03** | partial | [#109](https://github.com/d-o-hub/rust-ascii-canvas/issues/109) | #98 merged; open Dependabot [#99](https://github.com/d-o-hub/rust-ascii-canvas/pull/99) wasm-bindgen |

---

## P1 — Product

| ID | Status | Issue | Notes |
|----|--------|-------|--------|
| **F-10** | open | [#110](https://github.com/d-o-hub/rust-ascii-canvas/issues/110) | SVG export (ADR-013) |
| **F-11** | open | [#111](https://github.com/d-o-hub/rust-ascii-canvas/issues/111) | Layers UI: lock, reorder, delete, merge |
| **F-12** | ✅ | — | Composite pixel render + export (#107) |
| **F-13** | open | [#111](https://github.com/d-o-hub/rust-ascii-canvas/issues/111) | Layer history (same issue as F-11) |
| **F-14** | open | [#112](https://github.com/d-o-hub/rust-ascii-canvas/issues/112) | Enhanced text tool (ADR-010) |
| **F-15** | open | [#113](https://github.com/d-o-hub/rust-ascii-canvas/issues/113) | Preview rendering style (ADR-011) |
| **F-16** | open | [#114](https://github.com/d-o-hub/rust-ascii-canvas/issues/114) | Eraser radius 1/3/5 |
| **F-17** | open | [#115](https://github.com/d-o-hub/rust-ascii-canvas/issues/115) | External paste / import plain ASCII |

---

## P2 — Quality & engineering

| ID | Status | Issue | Notes |
|----|--------|-------|--------|
| **F-20** | open | [#116](https://github.com/d-o-hub/rust-ascii-canvas/issues/116) | Split `web/main.ts` (~1350 LOC) |
| **F-21** | open | [#117](https://github.com/d-o-hub/rust-ascii-canvas/issues/117) | Firefox + WebKit E2E in CI |
| **F-22** | open | [#118](https://github.com/d-o-hub/rust-ascii-canvas/issues/118) | WASM-generated TS types (ADR-033) |
| **F-23** | open | [#119](https://github.com/d-o-hub/rust-ascii-canvas/issues/119) | ADR 001–036 status audit |
| **F-24** | open | [#120](https://github.com/d-o-hub/rust-ascii-canvas/issues/120) | wasm-opt / binaryen reliability |
| **F-25** | open | [#121](https://github.com/d-o-hub/rust-ascii-canvas/issues/121) | wasm `missing_docs` allows |
| **F-26** | ✅ | — | `openEditor()` migration largely done in #107 |
| **F-27** | open | [#122](https://github.com/d-o-hub/rust-ascii-canvas/issues/122) | Clipboard `readText` E2E geometry |
| **F-28** | open | [#123](https://github.com/d-o-hub/rust-ascii-canvas/issues/123) | Dirty-rect pixel buffer (ADR-028) |

---

## P3 — Stretch / future

| ID | Status | Issue | Notes |
|----|--------|-------|--------|
| **F-30** | open | [#124](https://github.com/d-o-hub/rust-ascii-canvas/issues/124) | Collab editing spike |
| **F-31** | open | [#125](https://github.com/d-o-hub/rust-ascii-canvas/issues/125) | Light theme + switcher |
| **F-32** | open | [#126](https://github.com/d-o-hub/rust-ascii-canvas/issues/126) | Mobile UX audit |
| **F-33** | open | [#127](https://github.com/d-o-hub/rust-ascii-canvas/issues/127) | crates.io / npm publish decision |

---

## Do not re-open

| When | Item |
|------|------|
| 2026-03-18 | Issue **#9** pixel buffer + putImageData — [PR #11](https://github.com/d-o-hub/rust-ascii-canvas/pull/11); keep closed |
| 2026-07-16 | Issue **#21** copy-paste fidelity — [PR #107](https://github.com/d-o-hub/rust-ascii-canvas/pull/107) merged |
| 2026-07-16 | Persistence, PNG, grid UI, basic layers, composite render, Codacy-clean CI |

---

## Related

- [PROJECT_STATUS.md](PROJECT_STATUS.md)
- [goal-state.md](goal-state.md)
- [ADR-036](ADRs/036-clipboard-fidelity-and-product-features.md)
- Issues: [#108](https://github.com/d-o-hub/rust-ascii-canvas/issues/108)–[#127](https://github.com/d-o-hub/rust-ascii-canvas/issues/127)
