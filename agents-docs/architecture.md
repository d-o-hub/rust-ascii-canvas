# Architecture Guide (Feedforward)

Agents **must** read this before changing module boundaries, adding dependencies, or growing large files.

## Layered design

```
┌─────────────────────────────────────────┐
│  web/          TypeScript Vite app      │  UI events, DOM, persistence, export
├─────────────────────────────────────────┤
│  src/wasm/     wasm-bindgen bindings    │  JS interop only
├─────────────────────────────────────────┤
│  src/ui/       shortcuts, theme, toolbar│  Editor chrome (Rust-side config)
│  src/render/   canvas, dirty rect, font │  May use core; not wasm
├─────────────────────────────────────────┤
│  src/core/     grid, tools, commands    │  Pure Rust — no wasm/web-sys
│  src/utils/    shared math/helpers      │  No layer above
└─────────────────────────────────────────┘
```

## Hard rules (enforced by `scripts/check-architecture.sh`)

| From | Must not import |
|------|-----------------|
| `src/core/**` | `crate::wasm`, `crate::render`, `crate::ui` |
| `src/utils/**` | `crate::wasm`, `crate::render`, `crate::ui`, `crate::core` (keep utils leaf-level; prefer no core) |
| `src/render/**` | `crate::wasm`, `crate::ui` |
| `src/ui/**` | `crate::wasm`, `crate::render` |

**Allowed:** `wasm` → `core`, `render`, `ui` as needed. `render`/`ui` → `core`. `web/` only talks to WASM public API (`AsciiEditor`), not Rust internals.

## File size

- Soft/hard limit: **500 lines** per source file (`.rs`, and prefer for `.ts` too).
- Known debt is listed in `.loc-allowlist` (one path per line). **Do not grow allowlisted files**; prefer extracting modules.
- New files over 500 lines fail the quality gate.

## WASM / web boundary

- Business logic and drawing algorithms live in **core**, not in `web/` or `wasm/helpers.rs`.
- `web/` modules stay focused: `clipboard.ts`, `persistence.ts`, `exportPng.ts`, `main.ts` orchestration.
- Public WASM surface should stay stable; prefer additive APIs.

## Testing map

| Layer | Prefer |
|-------|--------|
| `core` tools/commands/grid | Unit tests in same file + `tests/core/` |
| Export / clipboard fidelity | Unit tests + focused E2E |
| DOM / UX wiring | Vitest where pure; Playwright for integration |
| All 8 tools | E2E + tool-validation skill |

## Performance / size fitness

- Release WASM size budget: **≤ 1.5MB** (`npm run check-size`).
- Prefer dirty-rect and sparse structures already in render/core; do not introduce full-grid O(n) work on every mouse move without measuring.

## When you must write an ADR

- New cross-layer dependency or splitting a crate
- Changing document format (`.asc`), clipboard semantics, or history model
- New CI quality gate or relaxing an existing one
- See `plans/ADRs/` and skill `goap-adr-planner`
