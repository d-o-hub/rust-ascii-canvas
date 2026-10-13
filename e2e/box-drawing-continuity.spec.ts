/** Assert that box-drawing borders are continuous after rasterization. */
import { expect, test } from '@playwright/test';
import { openEditor, requireAsciiContent } from './helpers';
import {
    borderProfile,
    cellCentre,
    expectContinuous,
    RECT,
    waitForPaint,
} from './border-profile';

test.describe('Box-drawing border continuity', () => {
    test.beforeEach(async ({ page }) => {
        await openEditor(page);
        await page.click('[data-tool="rectangle"]');

        const from = await cellCentre(page, RECT.left, RECT.top);
        const to = await cellCentre(page, RECT.right, RECT.bottom);
        await page.mouse.move(from.x, from.y);
        await page.mouse.down();
        await page.mouse.move(to.x, to.y, { steps: 10 });
        await page.mouse.up();
        await waitForPaint(page);
    });

    test('the document really contains vertical border glyphs', async ({ page }) => {
        const ascii = await requireAsciiContent(page);
        const rows = ascii.split('\n').filter((row) => row.includes('│'));
        expect(rows.length).toBeGreaterThanOrEqual(RECT.bottom - RECT.top - 1);
    });

    test('renders the horizontal border as one continuous line', async ({ page }) => {
        expectContinuous(await borderProfile(page, 'h'), 'h');
    });

    test('renders the vertical border as one continuous line', async ({ page }) => {
        expectContinuous(await borderProfile(page, 'v'), 'v');
    });
});
