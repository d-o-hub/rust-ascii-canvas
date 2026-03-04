# ADR-019: E2E Test Enhancement Strategy

## Status
**Proposed** - 2026-03-01

## Context

Current E2E test coverage is 5.4/10 with critical gaps:
- No pan functionality tests
- No drawing output verification
- Flaky test patterns (hardcoded waits)
- Single browser testing (Chromium only)

## Decision

Implement comprehensive E2E testing strategy following Playwright 2026 best practices:

### 1. Add Missing Critical Tests

```typescript
// e2e/canvas.spec.ts - Pan functionality
test.describe('Pan Functionality', () => {
    test('should pan canvas with Space+drag', async ({ page }) => {
        await page.click('#canvas');
        
        // Get initial pan state
        const initialPan = await page.evaluate(() => window.editor.pan);
        
        // Pan with Space+drag
        await page.keyboard.down('Space');
        await page.mouse.move(100, 100);
        await page.mouse.down();
        await page.mouse.move(200, 200, { steps: 10 });
        await page.mouse.up();
        await page.keyboard.up('Space');
        
        // Verify pan changed
        const newPan = await page.evaluate(() => window.editor.pan);
        expect(newPan).not.toEqual(initialPan);
    });
});
```

### 2. Drawing Output Verification

```typescript
test('should draw rectangle with correct ASCII characters', async ({ page }) => {
    await page.click('[data-tool="rectangle"]');
    await drawRect(page, 10, 10, 20, 15);
    
    const ascii = await page.evaluate(() => window.editor.export_ascii());
    
    // Verify border characters
    expect(ascii).toContain('┌'); // Top-left corner
    expect(ascii).toContain('┐'); // Top-right corner
    expect(ascii).toContain('└'); // Bottom-left corner
    expect(ascii).toContain('┘'); // Bottom-right corner
    expect(ascii).toContain('─'); // Horizontal
    expect(ascii).toContain('│'); // Vertical
});
```

### 3. Replace Flaky Patterns

```typescript
// BEFORE: Hardcoded wait
await page.waitForTimeout(500);
await expect(element).toBeVisible();

// AFTER: Proper assertion
await expect(element).toBeVisible({ timeout: 5000 });
```

### 4. Cross-Browser Testing

```typescript
// playwright.config.ts
export default defineConfig({
    projects: [
        {
            name: 'chromium',
            use: { ...devices['Desktop Chrome'] },
        },
        {
            name: 'firefox',
            use: { ...devices['Desktop Firefox'] },
        },
        {
            name: 'webkit',
            use: { ...devices['Desktop Safari'] },
        },
    ],
    retries: process.env.CI ? 1 : 0,
    workers: process.env.CI ? 1 : '50%',
});
```

### 5. Page Object Model

```typescript
// e2e/pages/EditorPage.ts
export class EditorPage {
    constructor(private page: Page) {}
    
    async selectTool(tool: string) {
        await this.page.click(`[data-tool="${tool}"]`);
    }
    
    async drawRect(x1: number, y1: number, x2: number, y2: number) {
        const canvas = this.page.locator('#canvas');
        await canvas.hover();
        await this.page.mouse.move(x1, y1);
        await this.page.mouse.down();
        await this.page.mouse.move(x2, y2, { steps: 20 });
        await this.page.mouse.up();
    }
    
    async getAsciiOutput(): Promise<string> {
        return this.page.evaluate(() => window.editor.export_ascii());
    }
}
```

## Test Coverage Targets

| Feature | Current | Target |
|---------|---------|--------|
| Pan functionality | 0% | 100% |
| Drawing output verification | 10% | 100% |
| Border style rendering | 30% | 100% |
| Select tool operations | 20% | 100% |
| Cross-browser | Chromium only | Chrome, Firefox, Safari |

## Consequences

### Positive
- Reliable, maintainable test suite
- Cross-browser confidence
- Proper coverage of all features
- Easier debugging with traces

### Negative
- Increased test runtime
- Additional maintenance overhead
- CI resource requirements

## Implementation

1. Update `playwright.config.ts` with cross-browser projects
2. Create `e2e/pages/EditorPage.ts` Page Object
3. Add pan functionality tests
4. Add drawing output verification tests
5. Replace all `waitForTimeout` with proper assertions
6. Run `npx playwright test`

## References

- [Playwright Best Practices](https://playwright.dev/docs/best-practices)
- [Playwright CI Guide](https://playwright.dev/docs/ci)
- [Page Object Model](https://playwright.dev/docs/pom)
