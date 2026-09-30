import { expect, test, type Page } from '@playwright/test';
import { clickGridCell, requireAsciiContent } from './helpers';

const DRAFTS_KEY = 'ascii-canvas-drafts-v1';
const LEGACY_KEY = 'ascii-canvas-autosave';
const legacy = JSON.stringify({
    format: 'ascii-canvas',
    version: 1,
    canvas: { width: 80, height: 40 },
    layers: [{ name: 'Legacy', cells: [{ x: 0, y: 0, ch: 'L' }] }],
});

async function ready(page: Page): Promise<void> {
    await page.goto('/');
    await page.waitForSelector('#loading.hidden', { state: 'attached', timeout: 30000 });
    await page.waitForFunction(() => window.editor !== null, null, { timeout: 15000 });
}

async function text(page: Page, value: string): Promise<void> {
    await page.locator('[data-tool="text"]').click();
    await clickGridCell(page, 0, 0);
    await page.keyboard.type(value);
    await page.keyboard.press('Escape');
}

async function newDraft(page: Page, name: string): Promise<void> {
    page.once('dialog', dialog => { void dialog.accept(name); });
    await page.locator('#draft-new').click();
    await expect(page.locator('#draft-name')).toHaveValue(name);
}

async function failWrites(page: Page): Promise<void> {
    await page.evaluate(() => {
        const original = Storage.prototype.setItem;
        Storage.prototype.setItem = function (key, value) {
            if (key === 'ascii-canvas-drafts-v1') {
                throw new DOMException('Storage full', 'QuotaExceededError');
            }
            original.call(this, key, value);
        };
    });
}

async function seedStorage(page: Page, key: string, value: string): Promise<void> {
    await page.addInitScript(({ key: storageKey, value: storageValue }) => {
        localStorage.setItem(storageKey, storageValue);
    }, { key, value });
}

function fullShelf(): string {
    return JSON.stringify({
        version: 1,
        revision: 'seed-revision',
        selected: 'draft-19',
        drafts: Array.from({ length: 20 }, (_, index) => ({
            id: `draft-${index}`,
            name: `Draft ${index}`,
            document: legacy,
        })),
    });
}

test.describe('Named local drafts', () => {
    test.beforeEach(async ({ isMobile }) => {
        test.skip(isMobile, 'Draft controls live in the desktop side panel');
    });

    test('draft A/B are isolated across reload; deleting selected B preserves A', async ({ page }) => {
        await ready(page);
        await page.locator('#draft-name').fill('A');
        await page.locator('#draft-name').press('Tab');
        await text(page, 'AAA');
        const a = await page.locator('#draft-select').inputValue();

        await newDraft(page, 'B');
        const b = await page.locator('#draft-select').inputValue();
        expect(await requireAsciiContent(page)).toBe('');
        await text(page, 'BBB');

        await page.locator('#draft-select').selectOption(a);
        await expect(page.locator('#draft-select')).toHaveValue(a);
        expect(await requireAsciiContent(page)).toBe('AAA');

        await page.reload();
        await page.waitForSelector('#loading.hidden', { state: 'attached', timeout: 30000 });
        await page.waitForFunction(() => window.editor !== null, null, { timeout: 15000 });
        await expect(page.locator('#draft-name')).toHaveValue('A');
        expect(await requireAsciiContent(page)).toBe('AAA');
        await page.locator('#draft-select').selectOption(b);
        await expect(page.locator('#draft-select')).toHaveValue(b);
        expect(await requireAsciiContent(page)).toBe('BBB');

        page.once('dialog', dialog => { void dialog.accept(); });
        await page.locator('#draft-delete').click();
        await expect(page.locator('#draft-select option')).toHaveCount(1);
        expect(await requireAsciiContent(page)).toBe('AAA');
        await expect(page.locator('#draft-delete')).toBeDisabled();
    });

    test('failed save/switch/import keep current in-memory work and offer a downloadable backup', async ({ page }) => {
        await ready(page);
        const a = await page.locator('#draft-select').inputValue();
        await newDraft(page, 'B');
        const b = await page.locator('#draft-select').inputValue();
        await failWrites(page);

        await text(page, 'UNSAVED');
        const before = await page.evaluate(() => window.editor?.serializeDocument());
        const storageBefore = await page.evaluate(key => localStorage.getItem(key), DRAFTS_KEY);

        await page.locator('#draft-save').click();
        await expect(page.locator('#save-status')).toContainText('Save failed');

        await page.locator('#draft-select').selectOption(a);
        await expect(page.locator('#draft-select')).toHaveValue(b);
        await expect(page.locator('#save-status')).toContainText('Save failed');

        const picker = page.waitForEvent('filechooser');
        await page.locator('#load-btn').click();
        await (await picker).setFiles({
            name: 'import.asc',
            mimeType: 'application/json',
            buffer: Buffer.from(legacy),
        });
        await expect(page.locator('#status-toast')).toContainText('Import failed');
        expect(await page.evaluate(() => window.editor?.serializeDocument())).toBe(before);
        expect(await page.evaluate(key => localStorage.getItem(key), DRAFTS_KEY)).toBe(storageBefore);

        const download = page.waitForEvent('download');
        await page.locator('#draft-download').click();
        expect((await download).suggestedFilename()).toMatch(/\.asc$/);
        expect(await requireAsciiContent(page)).toBe('UNSAVED');
    });

    test('import is a new draft, saves the current draft first, and starts with fresh history', async ({ page }) => {
        await ready(page);
        await text(page, 'ORIGINAL');
        const original = await page.locator('#draft-select').inputValue();
        await expect(page.locator('#undo-btn')).toBeEnabled();

        const picker = page.waitForEvent('filechooser');
        await page.locator('#load-btn').click();
        await (await picker).setFiles({
            name: 'imported.asc',
            mimeType: 'application/json',
            buffer: Buffer.from(legacy),
        });

        await expect(page.locator('#draft-name')).toHaveValue('imported');
        await expect(page.locator('#draft-select option')).toHaveCount(2);
        expect(await requireAsciiContent(page)).toBe('L');
        await expect(page.locator('#undo-btn')).toBeDisabled();

        await page.locator('#draft-select').selectOption(original);
        await expect(page.locator('#draft-select')).toHaveValue(original);
        expect(await requireAsciiContent(page)).toBe('ORIGINAL');
        await expect(page.locator('#undo-btn')).toBeDisabled();
    });

    test('failed migration leaves legacy bytes and legacy content visible, not a blank replacement', async ({ page }) => {
        await seedStorage(page, LEGACY_KEY, legacy);
        await page.addInitScript(() => {
            const original = Storage.prototype.setItem;
            Storage.prototype.setItem = function (name, value) {
                if (name === 'ascii-canvas-drafts-v1') {
                    throw new DOMException('Storage full', 'QuotaExceededError');
                }
                original.call(this, name, value);
            };
        });

        await ready(page);
        expect(await requireAsciiContent(page)).toBe('L');
        await expect(page.locator('#save-status')).toContainText('Save failed');
        expect(await page.evaluate(key => localStorage.getItem(key), LEGACY_KEY)).toBe(legacy);
        expect(await page.evaluate(key => localStorage.getItem(key), DRAFTS_KEY)).toBeNull();
        await expect(page.locator('#draft-download')).toBeVisible();
    });

    test('successful migration retains the legacy recovery bytes and selects the recovered draft', async ({ page }) => {
        await seedStorage(page, LEGACY_KEY, legacy);
        await ready(page);

        expect(await requireAsciiContent(page)).toBe('L');
        await expect(page.locator('#draft-name')).toHaveValue('Recovered diagram');
        await expect(page.locator('#save-status')).toHaveText('Saved locally');
        expect(await page.evaluate(key => localStorage.getItem(key), LEGACY_KEY)).toBe(legacy);
        expect(await page.evaluate(key => localStorage.getItem(key), DRAFTS_KEY)).not.toBeNull();
    });

    test('a valid selected draft loads before the legacy key and restores on the first visit', async ({ page }) => {
        const shelf = JSON.stringify({
            version: 1,
            revision: 'initial-revision',
            selected: 'selected',
            drafts: [{ id: 'selected', name: 'Initial draft', document: legacy }],
        });
        await seedStorage(page, DRAFTS_KEY, shelf);
        await seedStorage(page, LEGACY_KEY, JSON.stringify({
            format: 'ascii-canvas',
            version: 1,
            canvas: { width: 80, height: 40 },
            layers: [{ name: 'Other', cells: [{ x: 0, y: 0, ch: 'X' }] }],
        }));

        await ready(page);
        await expect(page.locator('#draft-name')).toHaveValue('Initial draft');
        expect(await requireAsciiContent(page)).toBe('L');
    });

    test('corrupt shelf is rejected without replacement or later overwrite', async ({ page }) => {
        const corrupt = '{not valid local drafts';
        await seedStorage(page, DRAFTS_KEY, corrupt);
        await ready(page);

        expect(await requireAsciiContent(page)).toBe('');
        await expect(page.locator('#save-status')).toContainText('Save failed');
        expect(await page.evaluate(key => localStorage.getItem(key), DRAFTS_KEY)).toBe(corrupt);

        await text(page, 'KEEP');
        await page.locator('#draft-save').click();
        expect(await requireAsciiContent(page)).toBe('KEEP');
        await expect(page.locator('#save-status')).toContainText('Save failed');
        expect(await page.evaluate(key => localStorage.getItem(key), DRAFTS_KEY)).toBe(corrupt);
    });

    test('a corrupt selected document is rejected without replacing the live editor or shelf', async ({ page }) => {
        const shelf = JSON.stringify({
            version: 1,
            revision: 'corrupt-document-revision',
            selected: 'corrupt',
            drafts: [{ id: 'corrupt', name: 'Corrupt', document: '{bad document' }],
        });
        await seedStorage(page, DRAFTS_KEY, shelf);
        await ready(page);

        expect(await requireAsciiContent(page)).toBe('');
        await expect(page.locator('#save-status')).toContainText('Save failed');
        expect(await page.evaluate(key => localStorage.getItem(key), DRAFTS_KEY)).toBe(shelf);
        await expect(page.locator('#draft-select option')).toHaveCount(0);

        await text(page, 'KEEP');
        await page.locator('#draft-save').click();
        expect(await requireAsciiContent(page)).toBe('KEEP');
        expect(await page.evaluate(key => localStorage.getItem(key), DRAFTS_KEY)).toBe(shelf);
    });

    test('the twenty-draft shelf boundary is visible and cannot be exceeded', async ({ page }) => {
        await seedStorage(page, DRAFTS_KEY, fullShelf());
        await ready(page);

        await expect(page.locator('#draft-select option')).toHaveCount(20);
        await expect(page.locator('#draft-new')).toBeDisabled();
        await expect(page.locator('#draft-name')).toHaveValue('Draft 19');
        expect(await requireAsciiContent(page)).toBe('L');
    });

    test('stale shelf conflicts block writes until reload explicitly recovers the session', async ({ page, context }) => {
        await ready(page);
        await text(page, 'FIRST');
        await page.locator('#draft-save').click();

        const stale = await context.newPage();
        await ready(stale);
        await text(page, 'NEWER');
        await page.locator('#draft-save').click();
        await expect(stale.locator('#save-status')).toContainText('Save conflict');

        const newerShelf = await page.evaluate(key => localStorage.getItem(key), DRAFTS_KEY);
        await text(stale, 'STALE');
        await stale.locator('#draft-save').click();
        expect(await stale.evaluate(key => localStorage.getItem(key), DRAFTS_KEY)).toBe(newerShelf);
        await expect(stale.locator('#save-status')).toContainText('Save conflict');

        page.once('dialog', dialog => { void dialog.accept('blocked'); });
        await stale.locator('#draft-new').click();
        await expect(stale.locator('#draft-select option')).toHaveCount(1);
        expect(await stale.evaluate(key => localStorage.getItem(key), DRAFTS_KEY)).toBe(newerShelf);

        await stale.reload();
        await expect(stale.locator('#save-status')).toHaveText('Saved locally');
        expect(await requireAsciiContent(stale)).toBe('NEWER');
        await text(stale, 'RECOVERED');
        await stale.locator('#draft-save').click();
        await expect(stale.locator('#save-status')).toHaveText('Saved locally');
        expect(await requireAsciiContent(stale)).toBe('RECOVERED');
        await stale.close();
    });

    test('draft name input keeps native editing and does not route keys into the canvas', async ({ page }) => {
        await ready(page);
        await text(page, 'ART');
        const before = await page.evaluate(() => window.editor?.serializeDocument());

        const name = page.locator('#draft-name');
        await name.fill('Native');
        await name.press('Control+A');
        await page.keyboard.type('Draft');
        await name.press('Backspace');
        await name.press('Tab');

        const nativePaste = await name.evaluate(input => {
            const data = new DataTransfer();
            data.setData('text/plain', ' pasted name');
            return input.dispatchEvent(new ClipboardEvent('paste', {
                bubbles: true,
                cancelable: true,
                clipboardData: data,
            }));
        });
        expect(nativePaste).toBe(true);
        await expect(name).toHaveValue('Draf');
        expect(await page.evaluate(() => window.editor?.serializeDocument())).toBe(before);
        expect(await requireAsciiContent(page)).toBe('ART');
    });

    test('new and deleted drafts do not carry document history across replacement', async ({ page }) => {
        await ready(page);
        await text(page, 'ORIGINAL');
        const original = await page.locator('#draft-select').inputValue();
        await expect(page.locator('#undo-btn')).toBeEnabled();

        await newDraft(page, 'Temporary');
        const temporary = await page.locator('#draft-select').inputValue();
        await expect(page.locator('#undo-btn')).toBeDisabled();
        await text(page, 'TEMPORARY');
        await expect(page.locator('#undo-btn')).toBeEnabled();

        page.once('dialog', dialog => { void dialog.accept(); });
        await page.locator('#draft-delete').click();
        await expect(page.locator('#draft-select')).toHaveValue(original);
        expect(await requireAsciiContent(page)).toBe('ORIGINAL');
        await expect(page.locator('#undo-btn')).toBeDisabled();
        expect(temporary).not.toBe(original);
    });
});
