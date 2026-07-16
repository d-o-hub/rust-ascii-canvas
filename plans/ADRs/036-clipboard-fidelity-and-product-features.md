# ADR-036: Clipboard Fidelity & Product Features Bundle

## Status
Accepted

## Date
2026-07-15

## Context

Issue #21 documented broken external copy/paste. Analysis also found missing persistence, PNG export, grid size UI, layers, and a monolithic `main.ts`.

## Decision

1. **Selection-aware copy** — `exportForCopy()` uses selection region or composite grid; Ctrl+C fills internal clipboard then exports.
2. **CRLF normalization** — TypeScript clipboard path normalizes `\n` → `\r\n` for Windows Notepad.
3. **Paste origin** — offset by selection top-left or last cursor cell; only visible cells stored on copy.
4. **`.asc` JSON document** — serialize/load multi-layer diagrams; localStorage auto-save.
5. **PNG export** — download from offscreen/main canvas.
6. **Layers** — named layer stack with active layer swap; composite export.
7. **Frontend modules** — `types`, `constants`, `utils`, `clipboard`, `persistence`, `exportPng`.

## Consequences

- External paste into Notepad/VS Code/web editors preserves box geometry when using monospace fonts.
- Users can recover work after refresh via auto-save.
- Multi-layer is basic (undo is per active layer; history clears on layer switch).

## Implementation notes (2026-07-16)

- **Verified**: clippy clean; 98 rust lib tests; 14 vitest; 71 chromium E2E.
- **Plans**: `full-recommendations-2026-07.md`, `FOLLOW_UPS.md`, `PROJECT_STATUS.md` updated.
- **Open process items**: PR + close #21 (F-01); manual Windows paste QA (F-02); Dependabot #98/#99 (F-03).
- **Deferred from this ADR**: SVG (F-10); composite on-screen layer render (F-12); full layer UX (F-11).
