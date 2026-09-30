import { expect, test } from '@playwright/test';
import { clickGridCell, openEditor, requireAsciiContent } from './helpers';

test('Text owns literal Space while named browser keys leave artwork unchanged', async ({ page }) => {
    await openEditor(page);
    await page.locator('[data-tool="text"]').click();
    await clickGridCell(page, 0, 0);
    await page.keyboard.type('A B');
    expect(await requireAsciiContent(page)).toBe('A B');
    const before = await page.evaluate(() => window.editor?.serializeDocument());
    await page.locator('#canvas').evaluate(canvas => {
        for (const key of ['Shift', 'Control', 'Dead', 'F1', 'Unidentified', 'ArrowRight']) {
            canvas.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true, cancelable: true }));
        }
    });
    expect(await page.evaluate(() => window.editor?.serializeDocument())).toBe(before);
    await page.keyboard.type('C');
    expect(await requireAsciiContent(page)).toBe('A BC');
});

test('native field typing and undo shortcuts do not mutate the canvas', async ({ page }) => {
    await openEditor(page);
    await page.locator('[data-tool="text"]').click();
    await clickGridCell(page, 0, 0);
    await page.keyboard.type('safe');
    const before = await page.evaluate(() => window.editor?.serializeDocument());
    const name = page.locator('.layer-name-input');
    await name.focus();
    await name.press('Control+A');
    await page.keyboard.type('draft text');
    await name.press('Backspace');
    await name.press('Control+Z');
    expect(await page.evaluate(() => window.editor?.serializeDocument())).toBe(before);
    expect(await requireAsciiContent(page)).toBe('safe');
});
