# ADR-048: Box-Drawing Glyphs Are Geometry, Not Font Ink

## Status

Accepted — chosen by the user on 2026-10-03 from the three routes put forward for
dogfood ISSUE-001 ([plans/FOLLOW_UPS.md](../FOLLOW_UPS.md), 2026-10-03 pass).
**Implementation:** #250 — `src/render/box_drawing.rs` as the single geometry
table, consumed by the canvas font atlas and the SVG exporter; the retained
rendered-border assertion is `e2e/box-drawing-continuity.spec.ts`. No document
format, WASM public surface, ruleset, release pin or required check changes.

## Context

A drawn rectangle exported correct ASCII — `│` on every border row — but *rendered*
with dashed left and right edges while top and bottom were solid, in the canvas,
in the PNG and in the SVG alike. Two independent mechanisms produced the same
defect:

- **Canvas / PNG.** `web/render.ts` rasterizes each font glyph at 13px (`RASTER_FONT_SIZE`)
  and places it in a 20px cell, so a `│` can only ever ink part of its own cell.
  Stacked down a column, the un-inked tail of each cell is a break in the border:
  the measured raster along the border column read `1x8, 0x5, 1x15, 0x5, 1x15…`
  (23 breaks) against a single unbroken `1x297` run across the top edge.
- **SVG.** `<text>` at `font-size=14px` on a 20px row pitch, so the glyph ink is
  ~18px tall and nine stacked `│` elements left eight ~2px gaps. Identical in
  Chromium, Firefox and WebKit, which rules out local font fallback: the problem
  is the relationship between glyph extent and cell pitch, not a missing font.

The two axes cannot both be satisfied by ink. A glyph tall enough to bridge a
20px row is a 20px-tall raster; the same glyph rotated to serve as a horizontal
stroke is 20px *wide*, so it cannot also be 8px wide and land on its own stem
column. No font size fixes a vertical dash without breaking the horizontal one,
and no font guarantees box-drawing glyphs whose strokes meet at cell boundaries.

## Decision

- **A single geometry table in Rust** — `src/render/box_drawing.rs` — defines the
  22 Unicode box-drawing glyphs the drawing tools can emit (light, double, heavy
  and rounded lines, corners and joins) as axis-aligned rectangles in an 8×20
  cell. Both renderers consume it. Two hand-written geometry tables is how this
  bug would get fixed in one path and stay broken in the other.
- **The canvas path synthesizes, and refuses to be overridden.** `FontAtlas`
  fills those codepoints from the table at construction *and* in `update_glyph`,
  which ignores the browser's raster for a box-drawing codepoint. The atlas is
  the last word, because the upload path is exactly where the 13px-in-20px raster
  used to re-enter. `render_glyph_placeholder` lost its box-drawing arms, so the
  pre-font-load placeholder and the final glyph are the same geometry.
- **The SVG path emits primitives.** `export_svg` replaces the `<text>` element
  for those codepoints with `<rect>` elements scaled by the *cell's* pitch rather
  than the font's, inheriting the group's existing `fill`. Non-box glyphs keep
  `<text>`.
- **The stem axes are the measured font axes** — column 3 of 8, row 7 of 20 — so
  fixing continuity does not slide existing diagrams sideways.
- **`- | + *` are excluded.** The ASCII and dotted border styles are the user's
  choice of a broken or literal line; `|` is also typed text. Only glyphs the
  tools use to *draw* a border are synthesized.
- **Rounded corners get a one-pixel chamfer.** 8×20 px cannot express an arc, and
  a faux curve reads as noise; the chamfer is what a box-drawing font manages at
  this size. `Rounded` must stay distinguishable from `Single`, which is asserted
  rather than assumed.

## Guard-rail boundaries

The `.asc` format, `exportAscii()`, clipboard semantics and the WASM public
surface are unchanged: documents still store characters, and only rendering
changes. **SVG output bytes do change** — box-drawing runs are `<rect>` elements
instead of `<text>` — so an SVG of the same document taken before this change is
not byte-equal to one taken after. Text in SVG remains text and stays searchable
and selectable; box-drawing runs become shapes. This is a user-visible artifact
change and is called out in the PR rather than buried.

## Alternatives

- **Rasterize the font at the full cell height.** Rejected: it changes metrics for
  every glyph in the atlas to fix 22 of them, and the source font's own
  box-drawing glyphs still do not meet at cell boundaries.
- **Match the line pitch to the font (e.g. a 15px cell).** Rejected: it relayouts
  the whole grid, changes PNG and SVG dimensions, and re-justifies the cell→column
  mapping existing documents depend on.
- **Draw borders in the TypeScript layer.** Rejected: a second geometry table, and
  it moves presentation maths away from the `core`/`render` boundary that already
  owns the cell model — plus it cannot reach the SVG exporter, which is Rust.

## Consequences

Stroke weight fidelity is bounded by the cell: a heavy line is two adjacent
pixels, not a true 2px-heavy stroke, and the rounded family is a chamfer. The
atlas now has a documented exception list, so a future "why doesn't my glyph
upload stick?" question has an answer in the module rather than in git history.
Both renderers inherit one table, so extending the covered set (e.g. the `┬ ┴
├ ┤ ┼` joins if a tool starts emitting them) is a single edit.

## Verification

- `src/render/box_drawing.rs` — 12 tests: full-cell coverage per axis, corners
  reach only the edges they join, one shared stem axis, double weight draws two
  strokes, rounded differs from square, nothing drawn outside the cell, and the
  SVG path scaling at an arbitrary pitch plus its fallbacks.
- `src/render/font_renderer.rs` — 4 new tests, including
  `uploading_a_font_raster_cannot_break_a_box_drawing_glyph`: feeding the atlas
  the exact rows-2..17 mask that caused ISSUE-001 leaves the atlas byte-equal to
  a fresh one.
- `src/wasm/helpers_tests.rs` — the old `test_export_svg`, which asserted a box
  character inside `<text>`, became two tests stating the new contract: the box
  is geometry with no `<text>` and no `dominant-baseline`, and an ordinary glyph
  still exports as `>A</text>`.
- `e2e/box-drawing-continuity.spec.ts` — asserts the user-visible invariant (a
  rendered border is one unbroken line) by sampling the canvas, with the
  horizontal edge as the control. 18/18 across Chromium, Firefox, WebKit,
  Pixel 5, iPhone 12 and iPad Air.
- Mutation evidence: with only the atlas's synthesis override removed, the
  vertical edge reads `gaps: 13, longestGap: 5` on Chromium while the horizontal
  control still passes — the original defect, reproduced from the test.
