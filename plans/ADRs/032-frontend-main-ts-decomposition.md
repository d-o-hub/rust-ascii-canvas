# ADR-032: Decompose main.ts Monolith into Focused Modules

## Status
Proposed

## Date
2026-03-03

## Context

`web/main.ts` is **787 lines of code**, exceeding the project-wide 500 LOC guideline by 57%. This was verified by:

```bash
$ wc -l web/main.ts
787 web/main.ts
```

The file currently handles **six distinct responsibilities** in a single flat module:

| Responsibility | Lines (approx.) | Functions |
|---|---|---|
| Type definitions (interfaces) | 1–50 | `EventResult`, `RenderCommand`, `AsciiEditorInterface` |
| Global state & constants | 51–115 | 20+ module-level `let`/`const` declarations |
| Initialization & lifecycle | 116–200 | `initialize()`, `measureFont()`, `resizeCanvas()` |
| Event handling | 201–400 | `setupEventListeners()`, `handlePointerDown/Move/Up/Leave()`, `handleWheel()`, `handleKeyDown/Up()` |
| Rendering pipeline | 401–530 | `requestRender()`, `render()`, `executeRenderCommand()` |
| UI utilities | 531–787 | `setTool()`, `updateToolButtons()`, `updateUI()`, `copyToClipboard()`, `showToast()`, `setZoom()`, `fitZoom()`, `cycleBorderStyle()`, `showShortcutsModal()`, `hideShortcutsModal()`, `capitalize()`, `debounce()` |

### Specific Code Evidence

**Global state sprawl** (lines 55–115): 20 module-level mutable variables with no encapsulation:
```typescript
// Global state
let editor: AsciiEditorInterface | null = null;
let canvas: HTMLCanvasElement | null = null;
let ctx: CanvasRenderingContext2D | null = null;
let animationFrameId: number | null = null;
let isInitialized = false;
void isInitialized; // Suppress unused variable warning  ← dead state

// DOM Elements - will be initialized in initialize()
let loadingOverlay: HTMLElement;
let canvasContainer: HTMLElement;
// ... 14 more module-level DOM element variables
```

**`void` suppression anti-pattern** (lines 60, 108, 110): Three variables are declared but never used, suppressed with `void`:
```typescript
let isInitialized = false;
void isInitialized; // Suppress unused variable warning

const LINE_DIRECTIONS = ['auto', 'horizontal', 'vertical'];
void LINE_DIRECTIONS; // Suppress unused - reserved for future UI

let currentLineDirection = 'auto';
void currentLineDirection; // Suppress unused - state tracked via DOM
```

**`executeRenderCommand` uses `const` inside `switch`** (lines 490–530): The `DrawGrid` case declares `const` variables inside a `switch` without a block scope, which is a lint warning in strict mode:
```typescript
case 'DrawGrid':
    // ...
    const panX = cmd.pan_x as number || 0;  // ← no block scope
    const panY = cmd.pan_y as number || 0;
```

**`setupEventListeners` is 120+ lines** (lines 230–360): A single function that wires every event in the application, making it impossible to test individual subsystems in isolation.

### Why This Matters

1. **Testability**: No individual function can be unit-tested with vitest because all state is module-global and intertwined.
2. **Maintainability**: Adding a new feature (e.g., layers, file persistence) requires editing a 787-line file.
3. **Guideline violation**: The project's 500 LOC guideline exists precisely to prevent this pattern (see `plans/TECHNICAL_ANALYSIS.md`).
4. **Cognitive load**: A new contributor must read 787 lines to understand any single feature.

## Decision

Split `web/main.ts` into **five focused modules**, each under 200 LOC:

```
web/
├── main.ts              # Entry point only (~50 LOC): imports, DOMContentLoaded
├── editor-state.ts      # Global state singleton + type definitions (~100 LOC)
├── event-handlers.ts    # All pointer/keyboard/wheel handlers (~200 LOC)
├── renderer.ts          # Render loop + executeRenderCommand (~150 LOC)
└── ui-controller.ts     # Tool switching, zoom, toast, modal, UI updates (~200 LOC)
```

### Module Responsibilities

**`editor-state.ts`** — Single source of truth:
```typescript
// editor-state.ts
export interface AsciiEditorInterface { ... }
export interface EventResult { ... }
export interface RenderCommand { ... }

export const state = {
    editor: null as AsciiEditorInterface | null,
    canvas: null as HTMLCanvasElement | null,
    ctx: null as CanvasRenderingContext2D | null,
    animationFrameId: null as number | null,
    charWidth: 8.4,
    lineHeight: 20,
    currentBorderStyleIndex: 0,
};
```

**`event-handlers.ts`** — Pure event logic, imports from `editor-state`:
```typescript
// event-handlers.ts
import { state } from './editor-state.js';
import { requestRender } from './renderer.js';
import { updateUI, showToast } from './ui-controller.js';

export function handlePointerDown(e: PointerEvent): void { ... }
export function handlePointerMove(e: PointerEvent): void { ... }
export function handlePointerUp(e: PointerEvent): void { ... }
export function handleKeyDown(e: KeyboardEvent): void { ... }
export function handleWheel(e: WheelEvent): void { ... }
```

**`renderer.ts`** — Isolated render pipeline:
```typescript
// renderer.ts
import { state } from './editor-state.js';

export function requestRender(): void { ... }
export function render(): void { ... }
function executeRenderCommand(cmd: RenderCommand): void { ... }
```

**`ui-controller.ts`** — UI state management:
```typescript
// ui-controller.ts
export function setTool(toolName: string): void { ... }
export function updateToolButtons(activeTool: string): void { ... }
export function updateUI(): void { ... }
export function showToast(message: string, isError?: boolean): void { ... }
export function setZoom(zoom: number): void { ... }
export function fitZoom(): void { ... }
```

**`main.ts`** — Thin entry point:
```typescript
// main.ts (~50 LOC)
import init from './pkg/ascii_canvas.js';
import { state } from './editor-state.js';
import { setupEventListeners } from './event-handlers.js';
import { requestRender } from './renderer.js';
import { updateUI } from './ui-controller.js';

async function initialize(): Promise<void> { ... }

if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initialize);
} else {
    initialize();
}
```

## Consequences

### Positive
- All files under 200 LOC (well within 500 LOC guideline)
- Each module is independently unit-testable with vitest
- New features (layers, file persistence) have clear homes
- Eliminates `void` suppression anti-pattern by removing dead state
- `editor-state.ts` provides a single place to audit all mutable state
- Enables tree-shaking of unused modules in production builds

### Negative
- Requires updating all import paths in E2E tests (none currently import from `main.ts` directly — only `window.editor` is used)
- Circular dependency risk: `event-handlers` → `renderer` → `editor-state` must be a DAG
- One-time refactor effort (~4 hours)
- Vite HMR may need verification after module split

## Implementation Plan

1. **Create `web/editor-state.ts`**: Extract all interfaces, global `let`/`const` declarations, and the `FONT_SIZE`, `GRID_WIDTH`, `GRID_HEIGHT`, `BORDER_STYLES` constants. Remove the three `void` suppressions by deleting `isInitialized`, `LINE_DIRECTIONS`, and `currentLineDirection` (or properly using them).
2. **Create `web/renderer.ts`**: Extract `requestRender()`, `render()`, `executeRenderCommand()`. Fix the `const` inside `switch` by adding block scopes `{ }` around each `case`.
3. **Create `web/event-handlers.ts`**: Extract `handlePointerDown/Move/Up/Leave()`, `handleWheel()`, `handleKeyDown/Up()`, `handleEventResult()`, and `setupEventListeners()`.
4. **Create `web/ui-controller.ts`**: Extract `setTool()`, `updateToolButtons()`, `updateUI()`, `copyToClipboard()`, `showToast()`, `setZoom()`, `fitZoom()`, `cycleBorderStyle()`, `showShortcutsModal()`, `hideShortcutsModal()`, `capitalize()`, `debounce()`, `updateCursorIndicator()`.
5. **Slim `web/main.ts`**: Keep only `initialize()`, `measureFont()`, `resizeCanvas()`, and the `DOMContentLoaded` bootstrap. Target: < 100 LOC.
6. **Verify**: `npm run build` passes, all E2E tests pass, `wc -l web/*.ts` shows all files < 300 LOC.

## Alternatives Considered

### A. Keep single file, add JSDoc sections
- **Rejected**: Does not fix testability or the 500 LOC violation. Cosmetic only.

### B. Use a class-based `Editor` wrapper
- **Rejected**: Introduces `this` binding complexity with DOM event listeners. Functional modules with a shared state object are simpler and more tree-shakeable.

### C. Use a framework (React, Svelte, Vue)
- **Rejected**: Overkill for this application. Adds build complexity and bundle size. The current vanilla TS approach is appropriate; it just needs better organization.

## References
- [ADR-023: Split bindings.rs](./023-split-bindings-rs.md) — Same pattern applied to Rust side
- [plans/TECHNICAL_ANALYSIS.md](../TECHNICAL_ANALYSIS.md) — 500 LOC guideline
- [ADR-018: TypeScript Production Standards](./018-typescript-production-standards.md)
