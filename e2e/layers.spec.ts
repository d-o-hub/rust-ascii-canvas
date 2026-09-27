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

/**
 * Drag a rectangle across the canvas, in fractions of its own box so the test
 * does not depend on the canvas being a particular size on screen.
 */
async function dragOnCanvas(
    page: Page,
    fromX: number,
    fromY: number,
    toX: number,
    toY: number
): Promise<void> {
    const canvas = page.locator('#canvas');
    const box = await canvas.boundingBox();
    if (!box) throw new Error('canvas has no box');
    // Resolved inline rather than through a local `at(fx, fy)` closure: each
    // point is used once, and a function-valued local in an async scope trips
    // Codacy's Qwik serialisability rule on what is plain Playwright test code.
    await page.mouse.move(box.x + box.width * fromX, box.y + box.height * fromY);
    await page.mouse.down();
    await page.mouse.move(box.x + box.width * toX, box.y + box.height * toY, { steps: 8 });
    await page.mouse.up();
}

/** The editor's ASCII export, as rendered in the browser. */
async function exportAscii(page: Page): Promise<string> {
    return page.evaluate(() => {
        const editor = (window as unknown as { editor?: { exportAscii: () => string } }).editor;
        return editor?.exportAscii() ?? '';
    });
}

/**
 * Activate a layer by clicking its row, then wait for the active class.
 *
 * The row is a div whose only children are the visibility/lock buttons, the name
 * input and the move/merge/delete buttons - all of which the app deliberately
 * excludes from activating the layer. So the click goes through the locator's
 * documented `position` option (keeping auto-waiting and actionability checks)
 * into the row's bottom-left padding, and the class check makes a mis-click fail
 * loudly instead of silently doing the wrong thing.
 */
async function activateLayer(page: Page, index: number) {
    const row = await layerRow(page, index);
    const box = await row.boundingBox();
    if (!box) throw new Error('layer row has no box');
    await row.click({ position: { x: 2, y: Math.max(0, box.height - 3) } });
    await expect(row).toHaveClass(/active/);
}

test.beforeEach(async ({ page }) => {
    await openEditor(page);
    page.on('dialog', (dialog) => void dialog.accept());
});

test('undo starts disabled and enables after a layer change', async ({ page }) => {
    await expect(page.locator('#undo-btn')).toBeDisabled();

    // A rename is recorded on the layer the user is on, so undo lights up here.
    await renameLayer(page, 0, 'Named');
    await expect(page.locator('#undo-btn')).toBeEnabled();
    await expect(page.locator('#undo-btn')).toHaveAttribute('title', /Rename layer/);
    await page.locator('#undo-btn').click();
    await expect(page.locator('#undo-btn')).toBeDisabled();
});

test('a new layer starts with no history of its own', async ({ page }) => {
    // Adding is recorded on the layer the user was on, so the new empty layer has
    // nothing to undo: the step lives where the work was, and undo brings focus
    // back there.
    await page.locator('#add-layer-btn').click();
    await expect(page.locator(LAYER_ITEM)).toHaveCount(2);
    await expect(await layerRow(page, 1)).toHaveClass(/active/);
    await expect(page.locator('#undo-btn')).toBeDisabled();

    await activateLayer(page, 0);
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

test('adding a layer is undoable from the layer the add was made on', async ({
    page,
}) => {
    await page.locator('#add-layer-btn').click();
    await expect(page.locator(LAYER_ITEM)).toHaveCount(2);
    await expect(await layerRow(page, 1)).toHaveClass(/active/);

    // The add is recorded on the layer the user was on (layer 0), so the new
    // layer starts with no history of its own. Undo is reachable only from
    // layer 0 — ADR-043 puts layer commands on the active layer's history, so
    // undo returns you to the layer you acted from.
    await activateLayer(page, 1);
    await expect(page.locator('#undo-btn')).toBeDisabled();

    await activateLayer(page, 0);
    await expect(page.locator('#undo-btn')).toBeEnabled();
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

    // Layer 1 was created empty, so navigating onto it must not conjure history.
    await activateLayer(page, 1);
    await expect(page.locator('#undo-btn')).toBeDisabled();
    await expect(page.locator('#redo-btn')).toBeDisabled();

    // Going back and forth is navigation, not an edit: the counts must not move.
    await activateLayer(page, 0);
    await expect(page.locator('#undo-btn')).toBeEnabled();
    await activateLayer(page, 1);
    await expect(page.locator('#undo-btn')).toBeDisabled();
    await activateLayer(page, 0);
    await expect(page.locator('#undo-btn')).toBeEnabled();
});

test('adding a layer can be redone', async ({ page }) => {
    await page.locator('#add-layer-btn').click();
    await expect(page.locator(LAYER_ITEM)).toHaveCount(2);

    // The add is recorded on the layer the user was on, so both steps live there.
    await activateLayer(page, 0);
    await expect(page.locator('#undo-btn')).toHaveAttribute('title', /Add layer/);
    await page.locator('#undo-btn').click();
    await expect(page.locator(LAYER_ITEM)).toHaveCount(1);

    await expect(page.locator('#redo-btn')).toBeEnabled();
    await page.locator('#redo-btn').click();
    await expect(page.locator(LAYER_ITEM)).toHaveCount(2);
    expect(await layerName(page, 1)).toBe('Layer 2');
});

test('merging drawn content is undone cell for cell', async ({ page }) => {
    // Draw on the lower layer, snapshot it, then add a layer and draw over the
    // same region so the merge has real overlap to overwrite.
    await dragOnCanvas(page, 0.15, 0.2, 0.35, 0.4);

    await page.locator('#add-layer-btn').click();
    await dragOnCanvas(page, 0.15, 0.2, 0.5, 0.5);
    // The composite with both layers, before the merge folds them together.
    const beforeMergeComposite = await exportAscii(page);

    await (await layerRow(page, 1)).getByRole('button', { name: 'Merge down' }).click();
    await expect(page.locator(LAYER_ITEM)).toHaveCount(1);
    // Folding the upper layer in leaves the same picture, which is the point of a
    // merge: the layer structure changed, not the drawing.
    const merged = await exportAscii(page);
    expect(merged).toBe(beforeMergeComposite);

    // One undo restores both layers, so the composite is exactly what it was
    // with two layers: the upper drawing is a layer again and the lower layer's
    // overwritten cells are back.
    await page.locator('#undo-btn').click();
    await expect(page.locator(LAYER_ITEM)).toHaveCount(2);
    expect(await exportAscii(page)).toBe(beforeMergeComposite);
});

test('a blocked undo says the layer is locked', async ({ page }) => {
    // Order matters. The add is recorded on the layer the user was on (layer 0),
    // so a drawing made AFTER the add is the top entry there. Drawing first would
    // leave the add on top, and an add is not blocked by a content lock - the
    // undo would simply succeed and never explain itself.
    await page.locator('#add-layer-btn').click();
    await expect(page.locator(LAYER_ITEM)).toHaveCount(2);
    await activateLayer(page, 0);

    // Draw on layer 0 so its newest entry is a drawing...
    await dragOnCanvas(page, 0.15, 0.2, 0.35, 0.4);
    // ...then lock it from layer 1, so the lock lands on layer 1's history and
    // the drawing stays on top of layer 0's.
    await activateLayer(page, 1);
    await (await layerRow(page, 0)).getByRole('button', { name: 'Lock layer' }).click();

    await activateLayer(page, 0);
    await expect(page.locator('#undo-btn')).toBeEnabled();
    await page.locator('#undo-btn').click();
    // The editor refuses and keeps the entry, so the user is told why instead of
    // watching the button do nothing.
    await expect(page.locator('#status-toast')).toHaveText(
        'Cannot undo on a locked layer - unlock it first'
    );
    await expect(page.locator('#undo-btn')).toBeEnabled();

    // Unlocking from the other layer frees the same step.
    await activateLayer(page, 1);
    await (await layerRow(page, 0)).getByRole('button', { name: 'Unlock layer' }).click();
    await activateLayer(page, 0);
    await page.locator('#undo-btn').click();
    expect(await exportAscii(page)).not.toContain('\u250c');
});

test('merge down is undoable', async ({ page }) => {
    await page.locator('#add-layer-btn').click();
    await (await layerRow(page, 1)).getByRole('button', { name: 'Merge down' }).click();
    await expect(page.locator(LAYER_ITEM)).toHaveCount(1);
    await expect(page.locator('#undo-btn')).toHaveAttribute('title', /Merge layer down/);

    await page.locator('#undo-btn').click();
    await expect(page.locator(LAYER_ITEM)).toHaveCount(2);
});
