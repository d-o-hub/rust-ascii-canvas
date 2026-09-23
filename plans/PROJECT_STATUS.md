# ASCII Canvas Editor - Project Status

## Overview

A production-grade Rust/WASM ASCII diagram editor with a dark/light Figma-like UI.

## Current Status: **Roadmap complete — release v0.1.4 pending (2026-09-23)** ✅

**Shipped since v0.1.3 (2026-08-05)**: clipboard export fidelity (#176/#177), coordinated wasm-bindgen 0.2.128 + pin-parity sensor (ADR-042, #192), zoom keyboard shortcuts (#194), mobile drawer Escape dismissal + focus restore (#195).

**Reality check (2026-09-23)**: every roadmap issue **#110–#127 is closed** and its feature verified in code (see [FOLLOW_UPS.md](FOLLOW_UPS.md)); 0 open PRs; docs reconciled in this pass.  
**Next**: cut **v0.1.4** via the [release runbook](RELEASING.md) — the 2026-08-08 and 2026-09-22 Release dispatches failed `Determine version` because `VERSION` was never bumped (harness L-007) — then curate the changelog backfill (R-01).

---

### Build Status

| Item | Value |
|------|--------|
| Version | 0.1.3 on `main`; **0.1.4 pending** (release-prep PR not yet opened) |
| WASM toolchain | cargo + wasm-bindgen **0.2.128** (dep + CLI pins enforced by `quality-gates.sh` §2b, ADR-042) |
| Target | `wasm32-unknown-unknown` / ES modules |
| Rust | stable (`rust-toolchain.toml`) |
| WASM size budget | ≤ 1.5MB (`npm run check-size`) |
| wasm-opt | Required in CI; local build fails without binaryen unless `SKIP_WASM_OPT=1` |

### Test Results (Verified 2026-09-23)

| Suite | Result |
|-------|--------|
| `cargo clippy --all-targets --all-features -- -D warnings` | ✅ clean (`npm run gate:fast`) |
| `cargo test --lib` | ✅ **117** passed (incl. `clipboard_tests`) |
| Integration + doc tests | ✅ |
| Vitest (`web/`) | ✅ **28** passed (3 files; clipboard CRLF, logger, UX) |
| ESLint + `tsc --noEmit` (`web/`) | ✅ (`npm run gate:fast`) |
| Playwright **Chromium** | ✅ **79** tests in 4 files (`--list`; CI runs the full matrix) |
| Playwright Firefox / WebKit | ✅ CI matrix (#117, `ci.yml:319`) |

### Features (current)

| Feature | Status |
|---------|--------|
| 8 drawing tools + 6 border styles | ✅ |
| Undo/redo, zoom/pan, select move/delete | ✅ |
| Zoom keyboard shortcuts (`+` / `-`) | ✅ #194 |
| Selection-aware copy + OS clipboard (CRLF) | ✅ #176 / ADR-041 |
| External paste / plain-ASCII import at cursor | ✅ e2e `clipboard.spec.ts` |
| File save/load (`.asc`) + localStorage auto-save | ✅ |
| PNG export | ✅ |
| SVG export | ✅ `web/exportSvg.ts` (F-10) |
| Grid size UI + responsive defaults | ✅ |
| Full layer editor (add/switch/rename/visible/lock/reorder/delete/merge) | ✅ (F-11) — layer ops are **not undoable** (F-13 residual, verified 2026-09-23) |
| Light theme + switcher | ✅ (F-31) |
| Preview rendering style | ✅ ADR-011 (F-15) |
| Enhanced text tool (caret, multi-line) | ✅ ADR-010 (F-14) |
| Eraser radius 1/3/5 | ✅ (F-16) |
| Dirty-rect pixel buffer | ✅ ADR-028 (F-28) |
| Mobile UX audit + drawer Escape/focus restore | ✅ #195 (F-32) |

---

## Recent Completions (2026-08 → 2026-09)

### Accessibility + infrastructure wave (2026-09-23 reconciliation) ✅
- **#195 drawer**: Escape dismisses the mobile side panel and restores focus to the menu button (guarded while the Text tool owns Escape); Codacy-clean.
- **#194 zoom**: `+` / `-` keyboard shortcuts for canvas zoom (rebased PR #185).
- **#192 / ADR-042**: coordinated wasm-bindgen 0.2.128 bump + pin-parity sensor; L-005 resolved.
- **#176 / #177 + ADR-041**: clipboard export fidelity controls + pure-ASCII fallback.
- **Harness**: L-006 (pnpm `approve-builds`), **L-007** + [release runbook](RELEASING.md); `scripts/release.sh` converted to a read-only preflight checker.

## Recent Completions (2026-07-15 → 2026-07-28)

### F-33: crates.io / npm package metadata / publishing decision (2026-07-28) ✅
- Decided on **app-only** distribution (no public library releases on crates.io or npm). Documented in [ADR-040](ADRs/040-app-only-distribution.md).

### F-02: Manual Clipboard Fidelity QA across Editors (2026-07-16) ✅
- **Draw box + arrow** verified: drew on canvas, hit Copy, confirmed clipboard contains right borders (┐, │, ┘, etc.), uniform line widths, and CRLF (`\r\n`) endings.
- **Cross-editor pasting compatibility** verified successfully for:
  - **Windows Notepad** (passes: lines preserve box shape & borders via CRLF)
  - **macOS TextEdit** (passes: borders perfectly preserved in plain text/monospace)
  - **VS Code** (passes: uniform column width, proper box drawing symbols alignment)
  - **Browser monospace textarea** (passes: uniform rendering and exact characters preserved)

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

## Immediate next steps (2026-09-23)

1. **R-01 — cut v0.1.4**: release-prep PR (4 pins) → gates → merge → `dry_run=true` → dispatch; curated changelog backfill ([RELEASING.md](RELEASING.md)).
2. **R-02 — changelog range fix** (candidate): derive release notes from the latest GitHub Release tag instead of `git describe`.
3. **R-03 — dev-only Dependabot advisory** esbuild GHSA-g7r4-m6w7-qqqr; pick up with the next web-deps bump.
4. **New product work** — no open issues remain; candidates in [FOLLOW_UPS.md](FOLLOW_UPS.md#new-work-candidates-need-issues) (incl. the layer-op history F-13 residual).
5. **Harness** — Adopted ADR-037 (2026-07-16): tiered gates, architecture fitness, web CI, verify/code-review skills

Full backlog: [FOLLOW_UPS.md](FOLLOW_UPS.md)

### Agent harness (2026-07-16)

| Piece | Location |
|-------|----------|
| ADR | [037-harness-engineering](ADRs/037-harness-engineering.md) |
| Map | [agents-docs/harness.md](../agents-docs/harness.md) |
| Fast/full sensors | `npm run gate:fast` / `gate:full` |
| Architecture sensor | `scripts/check-architecture.sh` |
| Skills | `verify`, `code-review` |

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
| + / - | Zoom in / out (#194) |
| Escape | Close shortcuts modal → close mobile drawer (Text tool keeps Escape) |
| Space+drag | Pan |

---

## PR Queue Triage (2026-08-07, historical) — goap orchestrator run

Reviewed all 3 open PRs with an agent swarm (a11y deep-review + local Codacy rule repro):

| PR | Title | Verdict | Outcome |
|----|-------|---------|---------|
| #168 | 🎨 Palette: Keyboard nav parity & mobile ARIA | **Impact** (a11y) — 2 Codacy high alerts, 2 real test bugs, 1 handler hardening | Fixed code (not Codacy config), pushed, CI re-run → merge |
| #169 | deps-dev bump (root: playwright, eslint, ts-eslint) | **Impact** (dev deps) | Merged ✅ |
| #170 | deps-dev bump (web: playwright, ts-eslint, vite) | **Impact** (dev deps) | Merged ✅ |

### #168 fix details (addressing all bot comments + failing CI)
- **Codacy** `xss/no-mixed-html` (2 high): trigger is html-named **variable identifiers**, not casts; fixed with generic `querySelector<HTMLInputElement>` + guard clause. Reproduced/verified locally via ADR-040 recipe.
- **Test bugs** (swarm findings): mobile-menu test was order-dependent on module-singleton `state.canvas` (`setupEventListeners` early-returns without it); layer-rename test never exercised commit semantics. Both fixed; Enter-commits / Escape-discards now asserted.
- **Hardening**: `change` handler guards against no-op renames so Escape-cancel can't re-commit restored names.
- Learnings documented in ADR-040 (Follow-up 3).

*Last updated: 2026-09-23 — docs reconciled with code, issue states, and CI (Track B).*
