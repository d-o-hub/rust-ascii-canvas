# Full Recommendations Implementation Plan

**Date**: 2026-07-15  
**Completed**: 2026-07-16  
**Status**: **COMPLETED** (core scope)  
**Source**: Open issue #21 + codebase improvement analysis  
**ADR**: [036-clipboard-fidelity-and-product-features.md](ADRs/036-clipboard-fidelity-and-product-features.md)

---

## Goals (original)

| # | Goal | Status |
|---|------|--------|
| 1 | Fix copy-paste (#21) end-to-end | ✅ Done |
| 2 | File persistence (localStorage + `.asc`) | ✅ Done |
| 3 | PNG export | ✅ Done |
| 4 | Decompose `web/main.ts` | ✅ Partial (modules extracted; main still orchestrator) |
| 5 | Package metadata + docs/ADR refresh | ✅ Done |
| 6 | Grid size UI | ✅ Done |
| 7 | Basic layer system | ✅ Done (basic stack) |
| 8 | Accessibility improvements | ✅ Partial (labels/ARIA on new controls) |
| 9 | Tests for clipboard fidelity | ✅ Done |

---

## What Shipped

### Rust / WASM (`src/wasm/`, `src/core/ascii_export.rs`)

- `export_for_copy()` — selection region or composite of visible layers
- Ctrl+C calls `copy_selection_impl()` then selection-aware export
- Paste origin = selection top-left or `last_cursor` (pointer position)
- Internal clipboard stores **visible cells only** (paste does not clobber with spaces)
- `is_clipboard_available()` requires secure context
- Document format: `serializeDocument` / `loadDocument` (`.asc` JSON, multi-layer)
- Layers: add / set active / rename / visibility; composite export
- Unit tests: `wasm::helpers::clipboard_tests` (5) + export right-border tests

### TypeScript (`web/`)

| Module | Role |
|--------|------|
| `clipboard.ts` | CRLF normalize + selection-aware OS copy |
| `persistence.ts` | Auto-save, download/upload `.asc` |
| `exportPng.ts` | PNG download from canvas |
| `types.ts` / `constants.ts` / `utils.ts` | Shared types and helpers |
| `main.ts` | Orchestration (events, render, UI wiring) |

### UI (`web/index.html`, `style.css`)

- Save / Load / PNG toolbar buttons
- Grid size inputs + Apply
- Layer select + Add layer
- Shortcuts modal: Ctrl+V / Ctrl+X

### Package / repo hygiene

- `Cargo.toml` + root `package.json` → `d-o-hub/rust-ascii-canvas`
- `build:wasm` tolerates missing `wasm-opt`
- TypeScript pinned to 5.x for eslint compatibility
- Vitest `happy-dom` environment

### E2E reliability (discovered while verifying)

| Bug | Fix |
|-----|-----|
| `#loading.hidden` wait used default `visible`, but CSS is `display:none` | `state: 'attached'` everywhere |
| Text tool typed `E`/`F`/`T` switched tools | Skip tool shortcuts while Text tool selected |
| Desktop auto-focus of mobile keyboard proxy stole focus | Coarse-pointer only |
| Responsive tests set viewport after load | Open with viewport **before** `goto` |
| Autosave polluted deterministic tests | `localStorage` clear in E2E init |

### Verification (2026-07-16)

| Suite | Result |
|-------|--------|
| `cargo clippy --all-targets --all-features -- -D warnings` | ✅ |
| `cargo test --lib` | ✅ 98 passed |
| Vitest (`web/`) | ✅ 14 passed |
| ESLint (`web/`) | ✅ |
| Playwright Chromium | ✅ **71 passed** |

---

## Follow-ups (backlog)

See also [FOLLOW_UPS.md](FOLLOW_UPS.md) for the living backlog.

### P0 — Ship / close the loop

| ID | Item | Notes |
|----|------|-------|
| F-01 | Open PR for this branch and close **#21** | Link ADR-036 + this plan |
| F-02 | Manual QA on Windows Notepad | Confirm CRLF + box borders 1:1 |
| F-03 | Review/merge Dependabot **#98**, **#99** | wasm-bindgen pin must stay in sync with CLI |

### P1 — Product completeness

| ID | Item | ADR / notes |
|----|------|-------------|
| F-10 | **SVG export** | ADR-013 (PNG done; SVG deferred) |
| F-11 | Layer polish | Lock, reorder, rename UI, merge; per-layer undo |
| F-12 | Composite **render** (not only export) | Draw all visible layers under active |
| F-13 | Layer history | Today history **clears** on layer switch |
| F-14 | Enhanced text tool | ADR-010 (cursor, multi-line polish) |
| F-15 | Preview rendering (blue tint) | ADR-011 |
| F-16 | Eraser size options | Gap analysis 2.3 |

### P2 — Quality & maintainability

| ID | Item | Notes |
|----|------|-------|
| F-20 | Finish `main.ts` decomposition | Still ~1300 LOC; split events/render/ui |
| F-21 | Cross-browser E2E in CI | firefox/webkit projects exist; CI often chromium-only |
| F-22 | TypeScript type safety | ADR-033; generated WASM types |
| F-23 | Remaining ADR status sweep | Many still “Proposed” though shipped earlier |
| F-24 | `wasm-opt` in local/CI path | Build skips if missing; pin binaryen via mise |
| F-25 | Doc warnings (`missing_docs` on wasm) | `#![allow(missing_docs)]` still on 4 modules |

### P3 — Stretch

| ID | Item | Notes |
|----|------|-------|
| F-30 | Collaborative editing | Goal-state Goal 6 |
| F-31 | Theme customization / light mode | Gap analysis 3.3 |
| F-32 | Touch/mobile polish beyond proxy | Pinch, larger hit targets audit |

---

## Related ADRs (status after this work)

| ADR | Topic | Status after bundle |
|-----|--------|---------------------|
| 009 | Selection copy/paste | Accepted (implemented) |
| 012 | Grid size customization | Accepted |
| 013 | PNG/SVG export | Accepted (PNG only; SVG deferred) |
| 026 | Layer system | Accepted (basic) |
| 027 | File persistence | Accepted |
| 032 | main.ts decomposition | Accepted (partial) |
| 036 | Clipboard fidelity & product features | Accepted |
