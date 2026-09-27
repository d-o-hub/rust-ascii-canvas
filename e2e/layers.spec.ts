import { expect, test, type Locator, type Page } from '@playwright/test';
import { openEditor } from './helpers';

/**
 * Layer operations are undoable (ADR-043 / issue #207).
 *
 * Every structural layer change must be one undo step with an exact redo, must
 * never trap the user behind a lock, and must not leave the undo button
 * claiming there is nothing to do when there is.
 */

const LAYER_ITEM = '#layer-list .layer-item';

/**
 * The panel lists the top layer first (`for (let i = count - 1; i >= 0; i--)` in
 * `web/ui.ts`), so a layer index has to be mapped to its row before clicking it.
 */
async function layerRow(page: Page, index: number): Promise<Locator> {
    const count = await page.locator(LAYER_ITEM).count();
    return page.locator(LAYER_ITEM).nth(count - 1 - index);
}

async function layerName(page: Page, index = 0): Promise<string> {
    return (await layerRow(page, index)).locator('.layer-name-input').inputValue();
}

/**
 * Rename through the panel input. The panel re-renders on every UI update and
 * replaces its inputs, so a fill can land on an element that is about to be
 * detached; retry until the name sticks instead of assuming one shot is enough.
 */
async function renameLayer(page: Page, index: number, name: string) {
    for (let attempt = 0; attempt < 3; attempt++) {
        const input = (await layerRow(page, index)).locator('.layer-name-input');
        await input.fill(name);
        await input.press('Tab'); // blur commits the input's change event
        // The input shows what was typed whether or not it was committed, so use
        // the undo title as the signal that the rename really reached the editor.
        try {
            await expect(page.locator('#undo-btn')).toHaveAttribute('title', /Rename layer/, {
                timeout: 2000,
            });
            return;
        } catch {
            /* the panel re-rendered under us; try again */
        }
    }
    await expect(page.locator('#undo-btn')).toHaveAttribute('title', /Rename layer/);
    expect(await layerName(page, index)).toBe(name);
}

/** Activate a layer by clicking its row, then wait for the active class. */
async function activateLayer(page: Page, index: number) {
    const row = await layerRow(page, index);
    await row.click({ position: { x: 4, y: 4 } });
    await expect(row).toHaveClass(/active/);
}

test.beforeEach(async ({ page }) => {
    await openEditor(page);
    page.on('dialog', (dialog) => void dialog.accept());
});

test('undo starts disabled and enables after a layer change', async ({ page }) => {
    await expect(page.locator('#undo-btn')).toBeDisabled();

    await page.locator('#add-layer-btn').click();
    await expect(page.locator('#undo-btn')).toBeEnabled();
    await expect(page.locator('#undo-btn')).toHaveAttribute('title', /Add layer/);
});

test('rename is one undo step and redoes exactly', async ({ page }) => {
    expect(await layerName(page)).toBe('Layer 1');

    await renameLayer(page, 0, 'Renamed');
    expect(await layerName(page)).toBe('Renamed');

    // Assert before clicking: a disabled button would make the click wait for its
    // full action timeout instead of reporting the real problem.
    await expect(page.locator('#undo-btn')).toBeEnabled();
    await page.locator('#undo-btn').click();
    expect(await layerName(page)).toBe('Layer 1');
    await expect(page.locator('#undo-btn')).toBeDisabled();

    await expect(page.locator('#redo-btn')).toBeEnabled();
    await page.locator('#redo-btn').click();
    expect(await layerName(page)).toBe('Renamed');
});

test('adding a layer is undoable straight away', async ({ page }) => {
    await page.locator('#add-layer-btn').click();
    await expect(page.locator(LAYER_ITEM)).toHaveCount(2);
    await expect(await layerRow(page, 1)).toHaveClass(/active/);

    await page.locator('#undo-btn').click();
    await expect(page.locator(LAYER_ITEM)).toHaveCount(1);
    await expect(await layerRow(page, 0)).toHaveClass(/active/);

    // A second undo must not remove another layer.
    await expect(page.locator('#undo-btn')).toBeDisabled();
});

test('deleting a layer is undoable and restores it in place', async ({ page }) => {
    await page.locator('#add-layer-btn').click();
    await activateLayer(page, 0);

    await (await layerRow(page, 0)).getByRole('button', { name: 'Delete layer' }).click();
    await expect(page.locator(LAYER_ITEM)).toHaveCount(1);

    await page.locator('#undo-btn').click();
    await expect(page.locator(LAYER_ITEM)).toHaveCount(2);
    expect(await layerName(page, 0)).toBe('Layer 1');
});

test('hiding a layer is undoable and the button names the step', async ({ page }) => {
    await (await layerRow(page, 0)).getByRole('button', { name: 'Hide layer' }).click();
    await expect(
        (await layerRow(page, 0)).getByRole('button', { name: 'Show layer' })
    ).toBeVisible();
    await expect(page.locator('#undo-btn')).toHaveAttribute('title', /Hide layer/);

    await page.locator('#undo-btn').click();
    await expect(
        (await layerRow(page, 0)).getByRole('button', { name: 'Hide layer' })
    ).toBeVisible();
});

test('a lock can be undone while the layer is still locked', async ({ page }) => {
    await (await layerRow(page, 0)).getByRole('button', { name: 'Lock layer' }).click();
    await expect(
        (await layerRow(page, 0)).getByRole('button', { name: 'Unlock layer' })
    ).toBeVisible();

    // Locking must not dead-end undo: the lock change itself is undoable.
    await page.locator('#undo-btn').click();
    await expect(
        (await layerRow(page, 0)).getByRole('button', { name: 'Lock layer' })
    ).toBeVisible();
});

test('switching layers does not create history', async ({ page }) => {
    await page.locator('#add-layer-btn').click();
    await activateLayer(page, 0);

    // Layer 0 has no history of its own, so undo is honestly unavailable.
    await expect(page.locator('#undo-btn')).toBeDisabled();
});

test('merge down is undoable', async ({ page }) => {
    await page.locator('#add-layer-btn').click();
    await (await layerRow(page, 1)).getByRole('button', { name: 'Merge down' }).click();
    await expect(page.locator(LAYER_ITEM)).toHaveCount(1);
    await expect(page.locator('#undo-btn')).toHaveAttribute('title', /Merge layer down/);

    await page.locator('#undo-btn').click();
    await expect(page.locator(LAYER_ITEM)).toHaveCount(2);
});
