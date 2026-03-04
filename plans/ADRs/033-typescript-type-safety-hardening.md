# ADR-033: TypeScript Type Safety Hardening

## Status
Proposed

## Date
2026-03-03

## Context

A deep audit of `web/main.ts` (787 LOC) and the E2E test files reveals **four categories of type safety violations** that undermine the TypeScript strict-mode guarantees the project aims for (see ADR-018).

### Category 1: `(editor as any)` Runtime Casts in main.ts

**Location**: `web/main.ts`, lines 314–315

```typescript
// Call WASM to set line direction (if supported)
if (editor && (editor as any).setLineDirection) {
    (editor as any).setLineDirection(direction);
}
```

**Problem**: `setLineDirection` IS declared in the `AsciiEditorInterface` at line 34:
```typescript
interface AsciiEditorInterface {
    // ...
    setLineDirection(direction: string): void;  // ← already typed!
    // ...
}
```

The `as any` cast is therefore **redundant and incorrect** — it bypasses the type system for a method that is already properly typed. This is a copy-paste artifact from before `setLineDirection` was added to the interface.

### Category 2: `void` Suppression Anti-Pattern

**Location**: `web/main.ts`, lines 60, 108, 110

```typescript
let isInitialized = false;
void isInitialized; // Suppress unused variable warning

const LINE_DIRECTIONS = ['auto', 'horizontal', 'vertical'];
void LINE_DIRECTIONS; // Suppress unused - reserved for future UI

let currentLineDirection = 'auto';
void currentLineDirection; // Suppress unused - state tracked via DOM
```

**Problem**: Using `void expr` to silence the TypeScript/ESLint "unused variable" warning is an anti-pattern. It:
- Keeps dead code in the bundle
- Signals that the variable was declared prematurely
- Masks the real issue: these variables should either be used or removed

`isInitialized` is set to `true` on line 183 but never read. `LINE_DIRECTIONS` and `currentLineDirection` are declared but the actual direction state is tracked via DOM class manipulation instead.

### Category 3: `@ts-ignore` in E2E Tests

**Location**: `e2e/canvas.spec.ts`, lines 237, 273, 306, 343, 372, 403, 798 (7 occurrences)

```typescript
const ascii = await page.evaluate(() => {
    // @ts-ignore
    return window.editor.exportAscii();
});
```

**Problem**: `window.editor` is explicitly declared in `main.ts`:
```typescript
declare global {
    interface Window {
        editor: AsciiEditorInterface | null;
    }
}
window.editor = null;
```

The `@ts-ignore` comments are unnecessary because `window.editor` is already typed. The E2E test files simply don't import the type declaration, so TypeScript can't see it. The fix is to share the type declaration, not suppress the error.

### Category 4: `(window as any)` in tools-drawing.spec.ts

**Location**: `e2e/tools-drawing.spec.ts`, lines 30, 183, 266, 293, 343 (5 occurrences)

```typescript
async function getAsciiContent(page: import('@playwright/test').Page) {
    return page.evaluate(() => (window as any).editor.exportAscii());
}
```

**Problem**: Same root cause as Category 3. `window.editor` is typed but the type is not imported into the test file. The `as any` cast silences the error instead of fixing it.

### Category 5: Untyped `RenderCommand` Field Access

**Location**: `web/main.ts`, `executeRenderCommand()` function (lines ~460–530)

```typescript
function executeRenderCommand(cmd: RenderCommand) {
    switch (cmd.type || Object.keys(cmd)[0]) {
        case 'Clear':
            ctx.fillStyle = cmd.color as string || '#1e1e1e';  // ← as string cast
            ctx.fillRect(0, 0, canvas.width, canvas.height);
            break;
        case 'DrawChar':
            ctx.fillStyle = '#d4d4d4';
            ctx.fillText(cmd.char as string, cmd.x as number, cmd.y as number);  // ← multiple casts
            break;
        case 'DrawRect':
            ctx.fillStyle = cmd.color as string || '#264f78';
            ctx.fillRect(cmd.x as number, cmd.y as number, cmd.width as number, cmd.height as number);
            break;
```

**Problem**: `RenderCommand` is typed as `{ type: string; [key: string]: unknown }` (line 14–16), so every field access requires a cast. This means TypeScript cannot catch typos like `cmd.colour` vs `cmd.color` or `cmd.Char` vs `cmd.char`.

The render commands are a tagged union from the Rust side — they should be typed as a discriminated union in TypeScript.

### Summary of Issues

| Issue | Location | Severity | Count |
|---|---|---|---|
| `(editor as any)` for typed method | `main.ts:314-315` | Medium | 2 |
| `void` suppression of dead variables | `main.ts:60,108,110` | Low | 3 |
| `@ts-ignore` on typed `window.editor` | `canvas.spec.ts` | Medium | 7 |
| `(window as any)` on typed `window.editor` | `tools-drawing.spec.ts` | Medium | 5 |
| Untyped `RenderCommand` field access | `main.ts:460-530` | High | 10+ |

## Decision

### Fix 1: Remove redundant `(editor as any)` casts

```typescript
// BEFORE (main.ts:314-315)
if (editor && (editor as any).setLineDirection) {
    (editor as any).setLineDirection(direction);
}

// AFTER
if (editor) {
    editor.setLineDirection(direction);
}
```

### Fix 2: Remove dead variables, use them or delete them

```typescript
// REMOVE these three declarations entirely:
// - isInitialized (set but never read)
// - LINE_DIRECTIONS (array never iterated)
// - currentLineDirection (state tracked via DOM, not this variable)

// If LINE_DIRECTIONS is needed for future cycling logic, keep it but use it:
// const LINE_DIRECTIONS = ['auto', 'horizontal', 'vertical'] as const;
// type LineDirection = typeof LINE_DIRECTIONS[number];
```

### Fix 3: Create a shared type declaration file for E2E tests

Create `e2e/types.d.ts`:
```typescript
// e2e/types.d.ts
import type { AsciiEditorInterface } from '../web/editor-state.js';

declare global {
    interface Window {
        editor: AsciiEditorInterface | null;
    }
}
```

Then in test files, replace `@ts-ignore` and `(window as any)`:
```typescript
// BEFORE
const ascii = await page.evaluate(() => {
    // @ts-ignore
    return window.editor.exportAscii();
});

// AFTER
const ascii = await page.evaluate(() => {
    return window.editor?.exportAscii() ?? '';
});
```

### Fix 4: Type `RenderCommand` as a discriminated union

```typescript
// In editor-state.ts (after ADR-032 decomposition)
type RenderCommand =
    | { type: 'Clear'; color: string }
    | { type: 'SetFont'; font: string; scale: number }
    | { type: 'DrawChar'; x: number; y: number; char: string; scale: number }
    | { type: 'DrawRect'; x: number; y: number; width: number; height: number; color: string }
    | { type: 'DrawGrid'; pan_x: number; pan_y: number; cell_width: number; cell_height: number; color: string };
```

This eliminates all `as string` / `as number` casts in `executeRenderCommand` and makes the switch exhaustive.

## Consequences

### Positive
- TypeScript strict mode catches field name typos in render commands at compile time
- Eliminates 14+ type suppression comments across the codebase
- `window.editor` type is shared between `main.ts` and all E2E tests — single source of truth
- Dead variables removed, reducing bundle size marginally
- `executeRenderCommand` becomes exhaustively type-checked

### Negative
- `RenderCommand` discriminated union must stay in sync with Rust's `RenderCommand` enum — a manual contract
- Requires updating `tsconfig.json` to include `e2e/types.d.ts` in the compilation scope
- One-time effort (~2 hours)

## Implementation Plan

1. **Remove dead variables** from `main.ts`: delete `isInitialized`, `LINE_DIRECTIONS`, `currentLineDirection` and their `void` suppressions (lines 60, 108, 110).
2. **Fix `(editor as any)` casts** at lines 314–315: use `editor.setLineDirection(direction)` directly.
3. **Define `RenderCommand` discriminated union** in `editor-state.ts` (after ADR-032 split).
4. **Update `executeRenderCommand`** to use the discriminated union — remove all `as string`/`as number` casts.
5. **Create `e2e/types.d.ts`** with the `Window` augmentation.
6. **Remove all `@ts-ignore`** from `canvas.spec.ts` (7 occurrences).
7. **Remove all `(window as any)`** from `tools-drawing.spec.ts` (5 occurrences).
8. **Verify**: `npx tsc --noEmit` passes with zero errors across `web/` and `e2e/`.

## Alternatives Considered

### A. Add `// eslint-disable` instead of fixing
- **Rejected**: Suppresses the symptom, not the cause. Increases technical debt.

### B. Use `unknown` instead of discriminated union for `RenderCommand`
- **Rejected**: `unknown` still requires type narrowing at every access point. The discriminated union is the correct TypeScript pattern for tagged enums.

### C. Generate TypeScript types from Rust via `wasm-bindgen` type generation
- **Considered**: `wasm-bindgen` can generate `.d.ts` files for WASM exports. However, `RenderCommand` is serialized as JSON (not a direct WASM type), so it won't appear in generated types. Manual typing is required.

## References
- [ADR-018: TypeScript Production Standards](./018-typescript-production-standards.md)
- [ADR-032: Frontend main.ts Decomposition](./032-frontend-main-ts-decomposition.md)
- TypeScript Handbook: [Discriminated Unions](https://www.typescriptlang.org/docs/handbook/2/narrowing.html#discriminated-unions)
