# ASCII Canvas Editor - Project Status

## Overview

A production-grade Rust/WASM ASCII diagram editor with a dark Figma-like UI.

## Current Status: **Feature Bundle Merged (2026-07-16)** ✅

**Focus completed**: Issue #21 copy-paste fidelity + product roadmap items (persistence, PNG, grid UI, basic layers, frontend modules).

**Shipped**: [PR #107](https://github.com/d-o-hub/rust-ascii-canvas/pull/107) merged to `main` (`fa8d0ee`). Issue **#21** closed.  
**Next**: follow-ups in [FOLLOW_UPS.md](FOLLOW_UPS.md) and linked GitHub issues.

---

### Build Status

| Item | Value |
|------|--------|
| Version | 0.1.1 |
| WASM toolchain | cargo + wasm-bindgen **0.2.121** |
| Target | `wasm32-unknown-unknown` / ES modules |
| Rust | stable |
| WASM size budget | ≤ 1.5MB (`npm run check-size`) |
| wasm-opt | Optional in local build if binaryen missing |

### Test Results (Verified 2026-07-16)

| Suite | Result |
|-------|--------|
| `cargo clippy --all-targets --all-features -- -D warnings` | ✅ clean |
| `cargo test --lib` | ✅ **98** passed (incl. `clipboard_tests`) |
| Integration + doc tests | ✅ |
| Vitest (`web/`) | ✅ **14** passed (clipboard CRLF, logger, UX) |
| ESLint (`web/`) | ✅ |
| Playwright **Chromium** | ✅ **71** passed |
| Playwright Firefox / WebKit | Configured; not required in last local gate |

### Features (current)

| Feature | Status |
|---------|--------|
| 8 drawing tools + 6 border styles | ✅ |
| Undo/redo, zoom/pan, select move/delete | ✅ |
| Selection-aware copy + OS clipboard (CRLF) | ✅ (2026-07) |
| Internal cut/copy/paste with paste origin | ✅ |
| File save/load (`.asc`) + localStorage auto-save | ✅ |
| PNG export | ✅ |
| Grid size UI + responsive defaults | ✅ |
| Basic layers (add/switch; composite export + composite pixel render) | ✅ basic |
| SVG export | ❌ deferred (F-10) |
| Full layer editor (lock/reorder/history) | ❌ (F-11, F-13) |

---

## Recent Completions (2026-07-15 → 2026-07-16)

### Issue #21 + recommendations bundle

**Plan**: [full-recommendations-2026-07.md](full-recommendations-2026-07.md)  
**ADR**: [036](ADRs/036-clipboard-fidelity-and-product-features.md)  
**Follow-ups**: [FOLLOW_UPS.md](FOLLOW_UPS.md)

1. **Clipboard fidelity**
   - `exportForCopy`, selection-scoped `export_region`, full-grid trim preserves right borders
   - Ctrl+C + Copy button + CRLF for external editors
   - Paste offset; visible-only internal clipboard
2. **Persistence**: `serializeDocument` / `loadDocument`, auto-save key `ascii-canvas-autosave`
3. **PNG export**, **grid size panel**, **layer select/add**
4. **Frontend modules**: `clipboard.ts`, `persistence.ts`, `exportPng.ts`, `types.ts`, `constants.ts`, `utils.ts`
5. **Repo metadata** → `github.com/d-o-hub/rust-ascii-canvas`
6. **E2E hardening**: loading wait state, text-tool vs shortcuts, responsive viewport order, autosave isolation

### Previous milestones (historical)

- **2026-03-20**: Tool malfunctions #18/#19 fixed; v0.1.1; 152 E2E era (counts later rebased)
- **2026-03-04**: Select move, tool switch, preview render path, freehand border style
- **Earlier**: WASM bindings split (ADR-023), waitForTimeout elimination (ADR-034), POM (ADR-035)

---

## Immediate next steps

1. **F-01** — Open PR; close #21  
2. **F-02** — Manual multi-OS paste QA  
3. **F-03** — Dependabot #98 / #99  
4. **F-10 / F-11** — SVG export and layer polish when product prioritizes  

Full backlog: [FOLLOW_UPS.md](FOLLOW_UPS.md)

---

## Architecture (summary)

```
ascii-canvas/
├── src/core/          # Grid, tools, commands, history, ascii_export
├── src/render/        # Canvas renderer, dirty rect, font atlas
├── src/wasm/          # AsciiEditor bindings (split modules)
├── web/               # Vite app
│   ├── main.ts        # Orchestration
│   ├── clipboard.ts / persistence.ts / exportPng.ts
│   └── pkg/           # wasm-bindgen output
├── e2e/               # Playwright (+ helpers.ts)
├── plans/             # Status, ADRs, follow-ups
└── tests/             # Rust integration tests
```

## Keyboard (high-signal)

| Key | Action |
|-----|--------|
| V R L A D T F E | Tools (disabled while Text tool selected — type freely, then Escape) |
| Ctrl+C / X / V | Copy / cut / paste (selection-aware) |
| Ctrl+Z / Y | Undo / redo |
| B | Cycle border style |
| Space+drag | Pan |

---

*Last updated: 2026-07-16*
