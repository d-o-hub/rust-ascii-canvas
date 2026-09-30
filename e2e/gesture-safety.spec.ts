import { expect, test } from '@playwright/test';
import { openEditor, requireAsciiContent } from './helpers';

test('paste during a freehand gesture survives cancellation and exact undo/redo', async ({ page }) => {
    await openEditor(page);
    await page.locator('[data-tool="freehand"]').click();
    const canvas = page.locator('#canvas');
    const box = await canvas.boundingBox();
    if (!box) throw new Error('Canvas missing');
    await page.mouse.move(box.x + 4, box.y + 10);
    await page.mouse.down();
    await page.mouse.move(box.x + 44, box.y + 10, { steps: 5 });
    const payload = await canvas.evaluate(element => {
        const paste = new ClipboardEvent('paste', { clipboardData: new DataTransfer(), bubbles: true, cancelable: true });
        // Firefox creates a separate data store; populate the event, not its input.
        paste.clipboardData?.setData('text/plain', 'P');
        element.dispatchEvent(paste);
        element.dispatchEvent(new PointerEvent('pointercancel', { pointerId: 1, pointerType: 'mouse' }));
        return paste.clipboardData?.getData('text/plain');
    });
    expect(payload).toBe('P');
    await page.mouse.up();
    expect(await requireAsciiContent(page)).toBe('P');
    await page.locator('#undo-btn').click();
    expect(await requireAsciiContent(page)).toBe('');
    await expect(page.locator('#undo-btn')).toBeDisabled();
    await page.locator('#redo-btn').click();
    expect(await requireAsciiContent(page)).toBe('P');
});

for (const interruption of ['pointercancel', 'lostpointercapture', 'blur']) {
    test(`${interruption} rolls back unfinished freehand and does not poison the next stroke`, async ({ page }) => {
        await openEditor(page);
        await page.locator('[data-tool="freehand"]').click();
        const canvas = page.locator('#canvas');
        const box = await canvas.boundingBox();
        if (!box) throw new Error('Canvas missing');
        await page.mouse.move(box.x + 4, box.y + 10);
        await page.mouse.down();
        await page.mouse.move(box.x + 84, box.y + 10, { steps: 10 });
        await page.evaluate(event => {
            if (event === 'blur') window.dispatchEvent(new Event('blur'));
            else document.getElementById('canvas')?.dispatchEvent(new PointerEvent(event, { pointerId: 1, pointerType: 'mouse' }));
        }, interruption);
        await page.mouse.up();
        expect(await requireAsciiContent(page)).toBe('');
        await expect(page.locator('#undo-btn')).toBeDisabled();
        await page.mouse.move(box.x + 4, box.y + 10);
        await page.mouse.down();
        await page.mouse.move(box.x + 84, box.y + 10, { steps: 10 });
        await page.mouse.up();
        expect(await requireAsciiContent(page)).not.toBe('');
        await page.locator('#undo-btn').click();
        expect(await requireAsciiContent(page)).toBe('');
    });
}
