import { test, expect } from '@playwright/test';

test.use({ viewport: { width: 1600, height: 1200 } });

test.beforeEach(async ({ page }) => {
    // Reproduce WebKit's positive-but-provisional flex height in every engine.
    // A collapsed-height-only guard misses this and freezes a 20-row document.
    await page.addInitScript(() => {
        const observer = new MutationObserver(() => {
            const container = document.getElementById('canvas-container');
            if (!container) return;
            observer.disconnect();
            const measure = container.getBoundingClientRect.bind(container);
            let provisional = true;
            container.getBoundingClientRect = () => {
                const rect = measure();
                if (!provisional) return rect;
                requestAnimationFrame(() => { provisional = false; });
                return new DOMRect(rect.x, rect.y, rect.width, 150);
            };
        });
        observer.observe(document, { childList: true, subtree: true });
    });
});

test('initial grid waits for layout even when provisional height is nonzero', async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('#loading.hidden', { state: 'attached' });
    const dimensions = await page.evaluate(() => ({
        width: window.editor?.width ?? 0,
        height: window.editor?.height ?? 0,
    }));
    expect(dimensions.width).toBeGreaterThan(120);
    expect(dimensions.width).toBeLessThanOrEqual(240);
    expect(dimensions.height).toBeGreaterThan(50);
    expect(dimensions.height).toBeLessThanOrEqual(80);
});

test('settling initial layout never replaces restored dimensions or edge content', async ({ page }) => {
    await page.goto('/');
    await page.waitForSelector('#loading.hidden', { state: 'attached' });
    const saved = await page.evaluate(() => {
        const editor = window.editor;
        if (!editor) throw new Error('Editor missing');
        editor.resize(190, 75);
        editor.onPointerMove(180 * 8 + 4, 74 * 20 + 10);
        if (!editor.pasteText('Saved edge')) throw new Error('Could not seed edge content');
        const document = editor.serializeDocument();
        localStorage.setItem('ascii-canvas-autosave', document);
        return document;
    });
    await page.reload();
    await page.waitForSelector('#loading.hidden', { state: 'attached' });
    expect(await page.evaluate(() => window.editor?.serializeDocument())).toBe(saved);
});
