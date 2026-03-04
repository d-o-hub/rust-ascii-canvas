# ADR-018: TypeScript Production Standards

## Status
**Proposed** - 2026-03-01

## Context

The TypeScript frontend has 4 critical issues and 6 medium issues that prevent production readiness. Key problems include unsafe type assertions, missing tests, and poor error handling.

## Decision

Implement production standards for TypeScript following 2026 best practices:

### 1. Defensive DOM Element Access

```typescript
// BEFORE: Unsafe non-null assertions
const canvas = document.getElementById('canvas')!;

// AFTER: Defensive access with error handling
function getRequiredElement<T extends HTMLElement>(id: string): T {
    const el = document.getElementById(id);
    if (!el) {
        throw new Error(`Required element #${id} not found`);
    }
    return el as T;
}

// Usage
const canvas = getRequiredElement<HTMLCanvasElement>('canvas');
```

### 2. Proper WASM Type Definitions

```typescript
// BEFORE: All methods return 'any'
interface AsciiEditor {
    getRenderCommands(): any;
    onKeyDown(key: string, ctrl: boolean, shift: boolean): any;
}

// AFTER: Proper typed interface
interface EventResult {
    needs_redraw: boolean;
    tool: string;
    can_undo: boolean;
    can_redo: boolean;
    should_copy: boolean;
    ascii: string | null;
}

interface AsciiEditor {
    getRenderCommands(): RenderCommand[];
    onKeyDown(key: string, ctrl: boolean, shift: boolean): EventResult;
    onPointerDown(screen_x: number, screen_y: number): EventResult;
    // ... etc
}
```

### 3. Centralized Configuration

```typescript
// BEFORE: Scattered magic numbers
const FONT_SIZE = 14;
let charWidth = 8.4;
const GRID_WIDTH = 80;

// AFTER: Centralized config
export const CONFIG = {
    FONT: {
        SIZE: 14,
        FAMILY: 'JetBrains Mono',
    },
    GRID: {
        WIDTH: 80,
        HEIGHT: 40,
    },
    ZOOM: {
        MIN: 0.3,
        MAX: 4.0,
        DEFAULT: 1.0,
    },
    TOAST_DURATION_MS: 2000,
} as const;
```

### 4. Add Unit Tests with Vitest

```typescript
// web/tests/main.test.ts
import { describe, it, expect, vi } from 'vitest';

describe('Tool Selection', () => {
    it('should select rectangle tool', () => {
        const { setTool } = await import('../main');
        setTool('rectangle');
        expect(editor?.tool).toBe('rectangle');
    });
});

describe('Keyboard Shortcuts', () => {
    it('should switch tools on key press', () => {
        // Test keyboard handling
    });
});
```

### 5. Modal Accessibility

```html
<!-- BEFORE: Missing ARIA -->
<div id="shortcuts-modal" class="modal-overlay hidden">

<!-- AFTER: Proper ARIA -->
<div id="shortcuts-modal" 
     class="modal-overlay hidden"
     role="dialog" 
     aria-modal="true" 
     aria-labelledby="modal-title">
    <div class="modal-content">
        <h2 id="modal-title">Keyboard Shortcuts</h2>
```

### 6. Error Handling Pattern

```typescript
// BEFORE: Console only
} catch (error) {
    console.error('Failed:', error);
}

// AFTER: Proper error handling
} catch (error) {
    const message = error instanceof Error ? error.message : 'Unknown error';
    reportError(error); // Send to tracking service
    showErrorToast(message); // User-friendly display
}
```

## Consequences

### Positive
- Type safety throughout frontend
- Proper error handling and recovery
- Accessibility compliance
- Testable code structure

### Negative
- Requires refactoring main.ts
- Need to maintain type definitions for WASM
- Additional test files to maintain

## Implementation

1. Create `web/src/config.ts` with centralized constants
2. Create `web/src/types/wasm.ts` with proper interfaces
3. Add `web/tests/` directory with Vitest tests
4. Update `web/index.html` with ARIA attributes
5. Refactor `web/main.ts` with defensive patterns
6. Run `npm run type-check` and `npm test`

## References

- [TypeScript Strict Mode](https://www.typescriptlang.org/tsconfig#strict)
- [Vitest Documentation](https://vitest.dev/)
- [WAI-ARIA Dialog Pattern](https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/)
