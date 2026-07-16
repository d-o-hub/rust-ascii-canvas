# Follow-ups Backlog

**Updated**: 2026-07-16  
**Source**: Full recommendations bundle (issue #21 + roadmap analysis)  
**Primary plan**: [full-recommendations-2026-07.md](full-recommendations-2026-07.md)

Use this list for prioritization. Mark items done in-place and mirror major completions into `PROJECT_STATUS.md`.

---

## P0 — Ship / close the loop

- [ ] **F-01** Open GitHub PR for the recommendations branch; close issue **#21** with test evidence
- [ ] **F-02** Manual QA: draw box + arrow → Copy → paste into Windows Notepad, macOS TextEdit, VS Code, web textarea (monospace)
- [ ] **F-03** Triage Dependabot PRs:
  - [#99](https://github.com/d-o-hub/rust-ascii-canvas/pull/99) wasm-bindgen 0.2.121 → 0.2.126 (sync `wasm-bindgen-cli` + Netlify install)
  - [#98](https://github.com/d-o-hub/rust-ascii-canvas/pull/98) dorny/paths-filter 3 → 4

---

## P1 — Product

- [ ] **F-10** SVG export (ADR-013 remainder)
- [ ] **F-11** Layers: lock, reorder, rename UI, delete layer, merge down
- [ ] **F-12** Render path composites all visible layers (today only active layer is drawn; export composites)
- [ ] **F-13** Per-layer undo history (or unified history with layer-tagged commands)
- [ ] **F-14** Enhanced text tool (ADR-010): visible caret, multi-line polish
- [ ] **F-15** Preview rendering with distinct style (ADR-011)
- [ ] **F-16** Adjustable eraser radius (1/3/5)
- [ ] **F-17** Import plain ASCII / paste external text into grid at cursor

---

## P2 — Quality & engineering

- [ ] **F-20** Split remaining `web/main.ts` (~1300 LOC) into `events.ts`, `render.ts`, `ui.ts`
- [ ] **F-21** Run firefox + webkit E2E in CI (projects already in `playwright.config.ts`)
- [ ] **F-22** ADR-033: WASM-generated TS types; remove remaining `any` / dual interfaces
- [ ] **F-23** ADR status audit: mark Implemented/Accepted/Superseded for 001–035 drift
- [ ] **F-24** Ensure `wasm-opt` (binaryen) available in CI and document in README/mise
- [ ] **F-25** Remove `#![allow(missing_docs)]` from wasm modules or document public API
- [ ] **F-26** E2E: use shared `e2e/helpers.ts` `openEditor()` everywhere (reduce duplicated beforeEach)
- [ ] **F-27** Clipboard E2E: assert `navigator.clipboard.readText()` geometry where permissions allow
- [ ] **F-28** Performance ADR-028 remaining items (sparse grid, allocation audit)

---

## P3 — Stretch / future

- [ ] **F-30** Collaborative editing (CRDT / WebRTC) — Goal 6 in goal-state
- [ ] **F-31** Light theme + theme switcher
- [ ] **F-32** Full mobile UX audit (toolbar crowding, layer/grid panels)
- [ ] **F-33** Publish crates.io / npm package metadata if distributing library

---

## Recently completed (do not re-open)

| When | Item |
|------|------|
| 2026-07-16 | #21 clipboard fidelity (selection export, paste origin, CRLF, visible cells) |
| 2026-07-16 | `.asc` serialize/load + localStorage auto-save |
| 2026-07-16 | PNG export |
| 2026-07-16 | Grid size UI + basic layers |
| 2026-07-16 | Frontend modules: clipboard, persistence, exportPng, types, constants, utils |
| 2026-07-16 | Package repo URLs → d-o-hub |
| 2026-07-16 | E2E: loading.hidden attached state; text-tool shortcut isolation; responsive viewport-before-goto |
| 2026-07-16 | Chromium E2E 71 passed; Rust lib 98; Vitest 14 |
