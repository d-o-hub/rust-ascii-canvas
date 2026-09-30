import { expect, test } from '@playwright/test';
import { clickGridCell, openEditor, requireAsciiContent } from './helpers';

test('viewport changes preserve dimensions, edge content and undo/redo history', async ({ page }) => {
    await openEditor(page);
    await page.locator('[data-tool="text"]').click();
    await clickGridCell(page, 70, 20);
    await page.keyboard.type('edge');
    await page.keyboard.press('Escape');
    const before = await page.evaluate(() => ({ document: window.editor?.serializeDocument(), undo: window.editor?.can_undo, redo: window.editor?.can_redo }));
    await page.setViewportSize({ width: 375, height: 667 });
    await expect(page.locator('#canvas')).toHaveCSS('width', '375px');
    // Inline canvas width changes only after the debounced resize ran.
    expect(await page.evaluate(() => ({ document: window.editor?.serializeDocument(), undo: window.editor?.can_undo, redo: window.editor?.can_redo }))).toEqual(before);
    expect(await requireAsciiContent(page)).toBe('edge');
});

test('manual shrink is confirmed and cancellation leaves the document and history unchanged', async ({ page }) => {
    await openEditor(page);
    await page.locator('[data-tool="text"]').click();
    await clickGridCell(page, 70, 20);
    await page.keyboard.type('edge');
    await page.keyboard.press('Escape');
    const before = await page.evaluate(() => window.editor?.serializeDocument());
    await page.locator('#grid-width').fill('40');
    await page.locator('#grid-height').fill('20');
    const dialog = page.waitForEvent('dialog');
    const apply = page.locator('#apply-grid-btn').click();
    const prompt = await dialog;
    expect(prompt.message()).toContain('permanently cropped');
    await prompt.dismiss();
    await apply;
    expect(await page.evaluate(() => window.editor?.serializeDocument())).toBe(before);
    await expect(page.locator('#undo-btn')).toBeEnabled();
    await page.locator('#grid-width').fill('40');
    await page.locator('#grid-height').fill('20');
    page.once('dialog', next => { void next.accept(); });
    await page.locator('#apply-grid-btn').click();
    await expect(page.locator('#grid-size')).toHaveText('40 × 20');
    expect(await requireAsciiContent(page)).toBe('');
});

test('pasting into a layer name stays native and never pastes into the canvas', async ({ page, context, browserName }) => {
    await openEditor(page);
    const native = await page.locator('.layer-name-input').evaluate(input => {
        const data = new DataTransfer();
        data.setData('text/plain', 'Pasted layer name');
        return input.dispatchEvent(new ClipboardEvent('paste', { bubbles: true, cancelable: true, clipboardData: data }));
    });
    expect(native).toBe(true); // All engines: the global handler must not prevent native paste.
    expect(await requireAsciiContent(page)).toBe('');
    // Chromium additionally supports automated permission-granted OS clipboard
    // paste; Firefox/WebKit exercise event ownership above, not a fake native paste.
    if (browserName !== 'chromium') return;
    await context.grantPermissions(['clipboard-read', 'clipboard-write']);
    await page.evaluate(() => navigator.clipboard.writeText('Pasted layer name'));
    const name = page.locator('.layer-name-input');
    await name.fill('');
    await name.press('Control+V');
    await expect(name).toHaveValue('Pasted layer name');
    expect(await requireAsciiContent(page)).toBe('');
    await name.press('Tab');
    await expect(name).toHaveValue('Pasted layer name');
});

test('desktop and mobile PNG export committed pixels without clearing selection', async ({ page }) => {
    await openEditor(page);
    await page.locator('[data-tool="text"]').click();
    await clickGridCell(page, 0, 0);
    await page.keyboard.type('PNG');
    await page.keyboard.press('Escape');
    const expected = await page.evaluate(() => {
        const editor = window.editor;
        if (!editor) throw new Error('Editor missing');
        const canvas = document.createElement('canvas');
        canvas.width = editor.width * 8;
        canvas.height = editor.height * 20;
        const context = canvas.getContext('2d');
        if (!context) throw new Error('Context missing');
        context.putImageData(new ImageData(new Uint8ClampedArray(editor.exportPixelBuffer()), canvas.width, canvas.height), 0, 0);
        const result = { png: canvas.toDataURL('image/png'), document: editor.serializeDocument(), undo: editor.can_undo };
        editor.selectAll();
        HTMLAnchorElement.prototype.click = function () { document.documentElement.dataset.exportedPng = this.href; };
        return result;
    });
    await page.locator('#png-btn').click();
    expect(await page.evaluate(() => document.documentElement.dataset.exportedPng)).toBe(expected.png);
    expect(await page.evaluate(() => window.editor?.has_selection)).toBe(true);
    expect(await page.evaluate(() => window.editor?.serializeDocument())).toBe(expected.document);
    expect(await page.evaluate(() => window.editor?.can_undo)).toBe(expected.undo);
    await page.setViewportSize({ width: 375, height: 667 });
    await page.locator('#mobile-menu-btn').click();
    await page.evaluate(() => { delete document.documentElement.dataset.exportedPng; });
    await page.locator('#mobile-png-btn').click();
    expect(await page.evaluate(() => document.documentElement.dataset.exportedPng)).toBe(expected.png);
    expect(await page.evaluate(() => window.editor?.has_selection)).toBe(true);
});
