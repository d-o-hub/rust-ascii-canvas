# ASCII Canvas Editor - End-to-End (E2E) Testing Architecture

This directory contains Playwright-driven end-to-end tests designed to verify the correct functionality, performance, and cross-browser consistency of the ASCII Canvas Editor.

## Directory Structure

- `canvas.spec.ts` — Verifies core canvas load states, basic tool selections, border styles, status bar info, and simple drawing operations.
- `clipboard.spec.ts` — Focuses on clipboard operations, selection-aware ASCII export fidelity, CRLF line endings, and document save/load round-tripping.
- `responsive.spec.ts` — Checks responsive grid resizing across mobile, tablet, and desktop viewports, ensuring edge-drawing works correctly.
- `tools-drawing.spec.ts` — Validates drawing with each of the 8 tools (Rectangle, Line, Arrow, Diamond, Text, Freehand, Select, Eraser), shape moving, select deletes, and edge cases.
- `helpers.ts` — Common utility functions like `openEditor` and `clearAutosave`.

---

## Browser Testing Matrix

We support and test the following browser configurations across multiple viewports:

| Project Name | Browser Engine | Device / Viewport Profile | Screen Width | Purpose |
|---|---|---|---|---|
| **chromium** | Chromium | Desktop Chrome | 1280x720 | Main production desktop target |
| **firefox** | Firefox | Desktop Firefox | 1280x720 | Cross-browser compatibility |
| **webkit** | WebKit (Safari) | Desktop Safari | 1280x720 | macOS/iOS layout compatibility |
| **mobile-chrome** | Chromium | Pixel 5 | 393x851 | Responsive layout & touch interactions |
| **mobile-safari** | WebKit | iPhone 12 | 390x844 | Mobile Apple ecosystem rendering |
| **tablet-safari** | WebKit | iPad Air | 820x1180 | Medium form-factor touch interactions |

---

## Flake Budget and Retry Strategy

End-to-End tests in browser environments are inherently vulnerable to transient flakes due to asynchronous rendering, network latency, or container resource constraints. To maintain high reliability and a strict "flake budget":

1. **Automatic Retries**:
   - **Continuous Integration (CI)**: Configured with `retries: 1` inside `playwright.config.ts`. This absorbs single-occurrence rendering delays or dev server warmup flakes.
   - **Local Development**: Configured with `retries: 0` so that any functional issues are highlighted immediately.

2. **Timeout Boundaries**:
   - **Test Timeout**: 60 seconds (`timeout: 60000`).
   - **Assertion Timeout**: 10 seconds (`expect: { timeout: 10000 }`).
   - **Action Timeout**: 30 seconds (`actionTimeout: 30000`).
   - **CI Web Dev Server Warmup**: Up to 60 seconds to guarantee Vite is fully compiled and ready on port 3003.

3. **Flake Mitigation Practices**:
   - Avoid `waitForTimeout()`. Instead, use state-based assertions like `expect().toBeVisible()` or `page.waitForFunction()`.
   - Wait for the WASM module to initialize fully via `window.editor !== null` before performing user actions.
   - Reset the storage state before every test via `clearAutosave()` to guarantee tests start from a pristine canvas state.

---

## Mobile Skip Policy

The ASCII Canvas Editor uses a highly responsive CSS grid design. On viewports with width $\le$ 768px (which includes the `mobile-chrome` and `mobile-safari` profiles), the following visual sections are hidden to optimize smaller screens:
- **Side Panel** (`.side-panel`) — Zoom indicators, Grid size inputs, Layers, and the Shortcuts list.
- **Toolbar Left/Right** (`.toolbar-left`, `.toolbar-right`) — Logo, Undo/Redo, Copy, Save, Load, PNG export, Clear Canvas, and Help buttons.
- **Status Bar** (`.status-bar`) — Active tool description and dynamic status toasts.

### The Policy:
1. **No Silent Failures**: We must never let tests quietly pass if they cannot find an element that should be visible on normal screens.
2. **Explicit Skipping**: Any test that explicitly interacts with, or asserts the visibility of, elements inside the hidden panels must be conditionally skipped on mobile viewports using Playwright's `isMobile` fixture:
   ```ts
   test('should have zoom controls', async ({ page, isMobile }) => {
       test.skip(isMobile, 'Zoom controls are hidden on mobile viewports');
       // ...
   });
   ```
3. **Core Verification Enforced**: Drawing tools, canvas interactions (pointer down, drag, pointer up), grid limits, and tool-switching via the center toolbar or keyboard shortcuts are fully verified on mobile.
