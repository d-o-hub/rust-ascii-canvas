/**
 * Box-drawing continuity — rendered borders must connect across cells.
 *
 * Repro for dogfood ISSUE-001 (plans/FOLLOW_UPS.md, 2026-10-03). The glyph atlas
 * in `web/render.ts:uploadFontAtlas()` rasterizes every glyph at 13px into a
 * 20px-tall cell with a 2px top offset, so a `│` can only ever ink rows 2..17.
 * Adjacent rows therefore leave a 5px gap and a vertical border renders dashed,
 * while the horizontal one is continuous (its ink spans the full 8px advance).
 *
 * The ASCII export is correct — only the rendering is wrong, in canvas, PNG and
 * SVG alike. These tests assert the user-visible invariant (a border is one
 * unbroken line), not a pixel layout, so any of the candidate fixes can turn
 * them green: draw box-drawing glyphs geometrically, size the raster to the
 * cell, or match the cell pitch to the font.
 *
 * The horizontal case is asserted alongside the vertical one on purpose: it is
 * the control. If the sampling geometry were wrong, both would fail and the
 * failure would be honest rather than a false green.
 */

import { test, expect, type Page } from '@playwright/test';
import { openEditor, requireAsciiContent } from './helpers';

/** Corner cells of the rectangle every test in this file draws. */
const RECT = { left: 3, top: 2, right: 22, bottom: 16 } as const;

type Profile = {
    /** Device pixels sampled along the border, excluding the corner cells. */
    samples: number;
    /** Runs of un-inked device pixels strictly inside the span. */
    gaps: number;
    /** Longest un-inked run, in device pixels. */
    longestGap: number;
    /** Device pixels per CSS pixel of the canvas. */
    scale: number;
    devicePixelRatio: number;
    zoom: number;
    /** Viewport offset in CSS px the grid is blitted at. */
    pan: [number, number];
    /** Canvas backing store size, and its CSS layout box. */
    canvasSize: [number, number];
    cssSize: [number, number];
    /** Top-left device pixel of the probed span, and the border line's index. */
    probe: [number, number];
    /**
     * Where the brightest ink on the whole canvas actually is.
     *
     * `gaps: 1` with `longestGap == samples` is ambiguous on its own: it reads
     * the same whether the border is genuinely one long break or the probe is
     * aimed at empty canvas. These two fields settle that — if the brightest
     * pixel anywhere is below the ink threshold the frame has no ink to find,
     * and if it is elsewhere they say how far off the aim was.
     */
    maxBrightness: number;
    brightest: [number, number];
};

/**
 * Walk one border edge of the drawn rectangle and count the breaks in its ink.
 *
 * `axis: 'v'` samples down the left border's cell column, `axis: 'h'` across the
 * top border's cell row. Each step takes the brightest pixel in a small strip
 * centred on the border, because sub-pixel drift would otherwise read as a gap.
 *
 * Every coordinate is derived, never assumed: the pitch comes from the metrics
 * the app publishes (`window.charWidth` / `window.lineHeight`), the CSS→device
 * ratio from the canvas' backing-store-to-layout width, and the origin from the
 * viewport transform the pixel buffer is blitted through
 * (`web/render.ts:142-148`, `drawImage(offscreen, panX, panY, w * zoom, ...)`).
 * The profile also reports the transform it measured under, so a failure says
 * whether the sampler was looking in the right place.
 */
async function borderProfile(page: Page, axis: 'v' | 'h'): Promise<Profile> {
    return page.evaluate(
        ({ axis, rect }) => {
            const canvas = document.querySelector<HTMLCanvasElement>('#canvas');
            if (!canvas) throw new Error('no #canvas to sample');
            const ctx = canvas.getContext('2d');
            if (!ctx) throw new Error('no 2d context on #canvas');

            const box = canvas.getBoundingClientRect();
            const scale = canvas.width / box.width;
            const cw = window.charWidth ?? 8;
            const lh = window.lineHeight ?? 20;
            const zoom = window.editor?.zoom ?? 1;
            const pan = window.editor?.pan ?? [0, 0];

            // A position in the grid, in CSS px, is (pan + gridCss) and then
            // `scale` device px each.
            const toDevice = (panAxis: number, gridCss: number) =>
                Math.round((panAxis + gridCss * zoom) * scale);

            const vertical = axis === 'v';
            const panAlong = vertical ? pan[1] : pan[0];
            const panAcross = vertical ? pan[0] : pan[1];

            // The span walked *along* the border, exclusive of the corner cells.
            // Walking uses that axis' own pitch: columns for a horizontal edge,
            // rows for a vertical one.
            const fromCell = (vertical ? rect.top : rect.left) + 1;
            const toCell = (vertical ? rect.bottom : rect.right) - 1;
            const alongPitch = vertical ? lh : cw;
            const startPx = toDevice(panAlong, fromCell * alongPitch);
            const endPx = toDevice(panAlong, (toCell + 1) * alongPitch);

            // The border's own cell, measured *across* — so it takes the other
            // axis' pitch. A row index must be scaled by lh, a column index by cw;
            // using one pitch for both puts the probe in the wrong place entirely
            // and makes both edges read as broken.
            const fixedCell = vertical ? rect.left : rect.top;
            const acrossPitch = vertical ? cw : lh;
            const fixedPx = toDevice(
                panAcross,
                fixedCell * acrossPitch + acrossPitch / 2
            );

            // One readback for the whole probe, so every number below describes
            // the *same* frame. Sampling pixel by pixel instead would let a
            // repaint land mid-walk.
            const full = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
            const luminanceAt = (x: number, y: number): number => {
                if (x < 0 || y < 0 || x >= canvas.width || y >= canvas.height) return 0;
                // `.at()` rather than a computed subscript: `security/detect-object-injection`
                // is locked to Codacy's Default coding standard, and the suite's answer is a
                // method call (as in the `lines.at()` assertions in `e2e/canvas.spec.ts`).
                const i = (y * canvas.width + x) * 4;
                return (
                    0.2126 * (full.at(i) ?? 0) +
                    0.7152 * (full.at(i + 1) ?? 0) +
                    0.0722 * (full.at(i + 2) ?? 0)
                );
            };

            // A +-3 *CSS* pixel band perpendicular to the border, expressed in the
            // device pixels it actually covers. The stroke is ~1px wide and lands a
            // few px off the cell centre (the `─` ink sits at y=47 in a row centred
            // on 50), so a tight band reads real ink as a gap — and a band written
            // in device pixels shrinks to nothing at a devicePixelRatio above 1.
            // The band stays inside the border's own cell, so it cannot borrow ink
            // from a neighbouring column.
            const band = [-3, -2, -1, 0, 1, 2, 3].map((o) => Math.round(o * scale));

            // 1 = un-inked device pixel along the border.
            const breaks: number[] = [];
            let longestGap = 0;
            let run = 0;

            for (let p = startPx; p < endPx; p++) {
                let best = 0;
                for (const o of band) {
                    const x = vertical ? fixedPx + o : p;
                    const y = vertical ? p : fixedPx + o;
                    best = Math.max(best, luminanceAt(x, y));
                }
                const isBreak = best > 120 ? 0 : 1;
                breaks.push(isBreak);
                if (isBreak === 0) {
                    if (run > longestGap) longestGap = run;
                    run = 0;
                } else {
                    run++;
                }
            }
            if (run > longestGap) longestGap = run;

            // Count distinct break runs rather than total broken length.
            let gaps = 0;
            run = 0;
            for (const b of breaks) {
                if (b === 1) {
                    run++;
                } else if (run > 0) {
                    gaps++;
                    run = 0;
                }
            }
            if (run > 0) gaps++;

            // Where the ink is, if anywhere: the brightest pixel on the whole
            // canvas. A border with `gaps: 1` is either really broken or being
            // probed at empty canvas, and these two look identical without it.
            let maxBrightness = 0;
            let brightest: [number, number] = [0, 0];
            for (let y = 0; y < canvas.height; y++) {
                for (let x = 0; x < canvas.width; x++) {
                    const lum = luminanceAt(x, y);
                    if (lum > maxBrightness) {
                        maxBrightness = lum;
                        brightest = [x, y];
                    }
                }
            }

            return {
                samples: breaks.length,
                gaps,
                longestGap,
                scale,
                devicePixelRatio: window.devicePixelRatio,
                zoom,
                pan: [pan[0], pan[1]],
                canvasSize: [canvas.width, canvas.height] as [number, number],
                cssSize: [box.width, box.height] as [number, number],
                probe: [vertical ? fixedPx : startPx, vertical ? startPx : fixedPx] as [
                    number,
                    number,
                ],
                maxBrightness: Math.round(maxBrightness),
                brightest,
            };
        },
        { axis, rect: RECT }
    );
}

/** Fail with the transform the sampler measured under, not just the count. */
function expectContinuous(profile: Profile, axis: 'v' | 'h'): void {
    expect(profile.samples, `${axis} edge sampled too few pixels`).toBeGreaterThan(50);
    expect(profile.gaps, `${axis} edge: ${JSON.stringify(profile)}`).toBe(0);
}

/**
 * Viewport coordinates of a cell centre, through the same transform the sampler
 * uses.
 *
 * `borderProfile` reads `pan` and `zoom` because the grid is not necessarily at
 * the canvas origin; a drag that hardcoded `cell * 8` would land on different
 * cells than the ones being probed, and the profile would then read one long
 * un-inked run — a `gaps: 1` that looks like a broken border and is really a
 * probe pointing at nothing.
 */
async function cellCentre(page: Page, cellX: number, cellY: number): Promise<{ x: number; y: number }> {
    return page.evaluate(
        ({ cellX, cellY }) => {
            const canvas = document.querySelector<HTMLCanvasElement>('#canvas');
            if (!canvas) throw new Error('no #canvas to locate');
            const box = canvas.getBoundingClientRect();
            const cw = window.charWidth ?? 8;
            const lh = window.lineHeight ?? 20;
            const zoom = window.editor?.zoom ?? 1;
            const pan = window.editor?.pan ?? [0, 0];
            return {
                x: box.x + pan[0] + (cellX * cw + cw / 2) * zoom,
                y: box.y + pan[1] + (cellY * lh + lh / 2) * zoom,
            };
        },
        { cellX, cellY }
    );
}

/**
 * Wait for the app to present a frame that includes the released drag.
 *
 * The pixel buffer is blitted from `render()`, which is scheduled on the *next*
 * animation frame (`web/render.ts:95-99`), so immediately after `mouse.up()` the
 * canvas still holds the pre-release frame. Reading it then returns a border
 * that does not exist yet — and a single frame is not enough, because an
 * evaluate callback queued before the app's own still samples the old composite.
 */
async function waitForPaint(page: Page): Promise<void> {
    await page.evaluate(
        () =>
            new Promise<void>((resolve) => {
                requestAnimationFrame(() => requestAnimationFrame(() => { resolve(); }));
            })
    );
}

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
        // Guards the other two tests: if the export lost its `│`, a rendering
        // pass would be meaningless.
        const ascii = await requireAsciiContent(page);
        const rows = ascii.split('\n').filter((r) => r.includes('│'));
        expect(rows.length).toBeGreaterThanOrEqual(RECT.bottom - RECT.top - 1);
    });

    test('renders the horizontal border as one continuous line', async ({ page }) => {
        // Control case — continuous before the fix, so it catches a fix that
        // trades one axis for the other.
        expectContinuous(await borderProfile(page, 'h'), 'h');
    });

    test('renders the vertical border as one continuous line', async ({ page }) => {
        // ISSUE-001: 13 gaps on Chromium at the time this repro was written.
        expectContinuous(await borderProfile(page, 'v'), 'v');
    });
});
