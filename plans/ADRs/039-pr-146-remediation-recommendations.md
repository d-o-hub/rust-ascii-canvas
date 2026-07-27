# ADR-039: PR #146 Remediation — WASM Type Contract, Abstraction Hygiene & Script Separation

## Status
Proposed

## Date
2026-07-27

## Context

PR #146 ("Refactor WASM TypeScript types F-22 / ADR-033") replaced the hand-maintained `AsciiEditorInterface` in `web/types.ts` with the WASM-generated `AsciiEditor` class type from `web/pkg/ascii_canvas.js`. It also added `tsc --noEmit` to the lint script.

A swarm analysis (3 agents: WASM type contract, textPos/Int32Array leak, tsconfig/lint setup) identified 5 concrete issues requiring remediation.

## Findings

### Finding 1: Redundant dual import + alias in main.ts

`web/main.ts` imports `AsciiEditor` as a value from `./pkg/ascii_canvas.js` AND as a type from `./types.ts` aliased to `AsciiEditorType`. Since TypeScript treats class imports as both a value and a type, the `types.ts` re-export is unnecessary for `main.ts` — the direct import is sufficient for both instantiation and type annotations.

The `types.ts` re-export (`export type { AsciiEditor }`) remains useful as a single source of truth for ALL OTHER files that don't need the runtime import (`clipboard.ts`, `exportSvg.ts`, `persistence.ts`, `state.ts`, `render.ts`).

**Recommendation**: In `main.ts`, use the direct import for both value and type position. Remove the redundant `import type { AsciiEditor as AsciiEditorType } from './types.js'`.

### Finding 2: Type-inaccurate `textCursorPosition` return type — implicit Int32Array leak

- Rust returns `Option<Vec<i32>>` → wasm-bindgen generates `Int32Array | null`
- Frontend type declares `number[] | null` — type-inaccurate
- Zero occurrences of `Int32Array` in frontend source code; the leak is invisible
- Only operations used on `textPos`: null-check, truthiness, destructuring — all work identically on `Int32Array`
- Runtime correctness is fine; type accuracy is wrong

**Swarm considered 5 options**:

| Option | Type accuracy | Churn | Risk |
|--------|------|-------|------|
| A. Keep `number[] \| null` (status quo) | ❌ | None | Low — works at runtime |
| B. Change to `Int32Array \| null` | ✅ | Frontend type changes | Low — but leaks WASM type |
| C. Convert in Rust to `js_sys::Array` | ✅ | Rust + WASM rebuild | Low — allocation overhead |
| D. Use tuple `[number, number] \| null` | ⚠️ | Type change only | Low — still type-inaccurate |
| E. Split into two fns (`textCursorX`, `textCursorY`) | ✅ | Rust + all consumers | Moderate |

**Recommendation**: Option B — update the exported type to `Int32Array | null`. It's accurate, zero-churn on the Rust side, and consumers already handle it correctly. The `Int32Array` is a standard JS built-in (not a WASM-specific concept), and destructuring works on it. If desired, a type alias in `types.ts` can document the rationale:

```typescript
/** WASM returns Int32Array (wasm-bindgen translates Vec<i32>).
 *  Int32Array supports destructuring and iteration identically to number[]. */
type TextPosition = Int32Array;
```

### Finding 3: `textPos = pos ? pos : null` is redundant

In `web/render.ts:270`:
```typescript
const pos = state.editor.textCursorPosition();
textPos = pos ? pos : null;
```

This is equivalent to `textPos = pos ?? null`, which is itself equivalent to `textPos = pos` (since `textPos` is already typed as `T | null` and `pos` is `T | null`). The ternary is dead logic.

**Recommendation**: Simplify to `textPos = state.editor.textCursorPosition();`.

### Finding 4: Missing `typecheck` script in web/package.json

PR #146 added `tsc --noEmit` inline to the `lint` script:
```json
"lint": "eslint . && tsc --noEmit"
```

This is problematic because:
- ESLint uses TypeScript 6 (via `typescript-eslint`), `tsc` uses TypeScript 7 — different tools, different failure modes
- Mixing concerns makes it harder to distinguish which tool failed
- The project's own architecture treats them as distinct checks (AGENTS.md, CI, quality-gates.sh all separate them)
- No standalone `typecheck` script exists anywhere in the project, despite ADR-018 referencing one

**Recommendation**: Revert the `lint` script to `"lint": "eslint ."` and add a separate `typecheck` script:
```json
"typecheck": "tsc --noEmit",
"lint": "eslint ."
```

Update the root `lint:web` script to chain both:
```json
"lint:web": "cd web && pnpm run lint && pnpm run typecheck"
```

### Finding 5: Lost interface contract documentation

The old `AsciiEditorInterface` (50 lines) served as explicit documentation of every WASM method the frontend depends on. Its removal means:
- No single place to see the frontend's expected WASM API surface
- PR reviewers must cross-reference `.d.ts` files that don't exist until WASM is built
- New contributors can't easily understand the editor's capabilities

**Recommendation**: Either:
a. Add a README comment in `types.ts` documenting the pattern
b. Or use a `type AsciiEditor = import('./pkg/ascii_canvas.js').AsciiEditor` re-export with a JSDoc block listing exported methods

## Decision

### Fix 1: Clean up main.ts imports

```typescript
// main.ts — use direct import for both value and type
import init, { AsciiEditor } from './pkg/ascii_canvas.js';
// Remove: import type { AsciiEditor as AsciiEditorType } from './types.js';

// Use AsciiEditor as the type directly:
declare global {
    interface Window {
        editor: AsciiEditor | null;
    }
}
```

### Fix 2: Update textCursorPosition type to Int32Array | null

In `web/types.ts`:
```typescript
export type { AsciiEditor };
```

The WASM-generated type already declares `textCursorPosition(): Int32Array | null`. Since we re-export the type directly, this is already correct as long as consumers don't override it.

Remove the redundant ternary in `web/render.ts`:
```typescript
// Before
const pos = state.editor.textCursorPosition();
textPos = pos ? pos : null;

// After
textPos = state.editor.textCursorPosition();
```

### Fix 3: Add `typecheck` script and revert `lint`

`web/package.json`:
```json
"typecheck": "tsc --noEmit",
"lint": "eslint ."
```

Root `package.json` `lint:web`:
```json
"lint:web": "cd web && pnpm run lint && pnpm run typecheck"
```

### Fix 4: Document the WASM contract

In `web/types.ts`, add a JSDoc block above the re-export to document the expected contract:

```typescript
/**
 * WASM-generated AsciiEditor class type.
 *
 * Exported methods (from Rust wasm/bindings.rs):
 *   - tool, width, height, zoom, pan, can_undo, can_redo ...
 *   - setTool, setBorderStyle, setEraserSize, setLineDirection ...
 *   - onPointerDown/Move/Up, onKeyDown/Up, onWheel ...
 *   - undo, redo, clear, textCursorPosition, selectAll ...
 *   - exportAscii, exportForCopy, exportSvg, serializeDocument ...
 *   - copySelection, paste, pasteText ...
 *   - layerName, layerVisible, setLayerVisible, setActiveLayer ...
 *   - addLayer, renameLayer, layerLocked, setLayerLocked ...
 *   - deleteLayer, moveLayer, mergeLayerDown ...
 *   - getRenderCommands, getDirtyRenderCommands ...
 *   - renderToPixelBuffer, updateFontAtlasGlyph, resize ...
 */
export type { AsciiEditor };
```

## Consequences

### Positive
- `main.ts` no longer imports the same type from two places with an alias
- `textCursorPosition` type is accurate at the WASM boundary
- `web/package.json` has a clear, separate `typecheck` script — CI and local dev can run it independently
- `lint` script stays focused on code style; `typecheck` handles type safety
- Interface contract is documented for future maintainers

### Negative
- `types.ts` re-export now has a ~25-line doc comment (maintenance burden if the Rust API changes)
- One-time churn across 7 files (mainly `main.ts`)
- Contributors must remember to run both `pnpm lint` AND `pnpm typecheck` (mitigated by root `lint:web` and quality gates)

## Implementation Order

1. `web/types.ts` — add JSDoc block above `export type { AsciiEditor }`
2. `web/main.ts` — remove redundant `import type { AsciiEditor as AsciiEditorType }`, use direct import as type
3. `web/render.ts` — simplify `textPos` assignment, remove ternary
4. `web/package.json` — split `lint`/`typecheck` scripts
5. Root `package.json` — update `lint:web` to chain `pnpm run typecheck`
6. Verify: `pnpm lint`, `pnpm typecheck`, `pnpm test`, `npm run gate:fast`

## References
- [PR #146](https://github.com/d-o-hub/rust-ascii-canvas/pull/146)
- [ADR-033: TypeScript Type Safety Hardening](./033-typescript-type-safety-hardening.md)
- [ADR-038: TypeScript 7 Side-by-Side](./038-typescript-7-side-by-side.md)
- [ADR-018: TypeScript Production Standards](./018-typescript-production-standards.md)
- AGENTS.md: Web typecheck/lint conventions
