//! Box-drawing glyphs are drawn as rectangles, never rasterized from a font.
//!
//! A font's `│` inks only part of the em box. This renderer's cells are 8x20,
//! while `web/render.ts:uploadFontAtlas()` rasterizes at 13px with a 2px top
//! offset, so a `│` covered rows 2..17 of 20 and every row boundary left a ~5px
//! gap: rectangle sides rendered dashed while top and bottom stayed solid. The
//! exported SVG had the same defect from an independent cause — a `font-size`
//! smaller than the row pitch. Dogfood ISSUE-001, 2026-10-03.
//!
//! Drawing the glyphs geometrically removes the font from the loop entirely, so
//! continuity no longer depends on which typeface loaded. It also gives canvas
//! and SVG one shared source of truth: both consume [`rects_for`], which is the
//! only place the stem positions are written down. Two renderers with two
//! hand-written geometry tables is how this bug got fixed once and stayed
//! broken in the other path.
//!
//! Only the 22 Unicode box-drawing codepoints are synthesized — the lines and
//! corners the border styles are built from. `-`, `|`, `+` and `*`, the `ascii`
//! and `dotted` styles, deliberately keep their font glyphs: `dotted` is *meant*
//! to break, and `|` is an ordinary character a user can type in text, so making
//! it a full-height bar would change what they wrote. Junctions (`├ ┤ ┬ ┴ ┼` and
//! the mixed-weight tees) are not in the table either: no tool draws them, and
//! they are not in the [`crate::render::FontAtlas`] glyph set, so synthesizing
//! them here would fix the SVG export while the canvas still cannot display them.

/// Cell width in device pixels, matching `GLYPH_WIDTH` in `web/constants.ts`.
pub const CELL_W: i32 = 8;
/// Cell height in device pixels, matching `GLYPH_HEIGHT` in `web/constants.ts`.
pub const CELL_H: i32 = 20;

/// Column the vertical stems sit in, measured from the left edge of the cell.
///
/// 3 keeps the synthesized stem exactly where the font's `│` already landed
/// (column 27 of cell column 3 on the production build), so fixing continuity
/// does not also shift every existing diagram sideways.
const STEM_X: i32 = 3;
/// Row the horizontal stems sit in, measured from the top of the cell.
///
/// 7 likewise matches where the font's `─` already inked (row 47 of cell row 2).
/// It is above the cell centre because that is where monospace fonts put the
/// box-drawing crossbar; centring it would move every border down by 3px.
const STEM_Y: i32 = 7;
/// Radius of a rounded corner's chamfer, in pixels along each stem.
///
/// 8x20 px is too coarse for a real arc: one pixel of curvature in either
/// direction is all the resolution there is. A single chamfer pixel is what a
/// box-drawing font manages at this size too, and it is what distinguishes
/// `╭` from `┌` in both the atlas and the SVG.
const ROUND_R: i32 = 2;

const UP: u8 = 1;
const DOWN: u8 = 2;
const LEFT: u8 = 4;
const RIGHT: u8 = 8;

const LIGHT: u8 = 0;
const DOUBLE: u8 = 1;
const HEAVY: u8 = 2;
const ROUNDED: u8 = 4;

/// A filled axis-aligned rectangle in cell units, origin at the cell's top-left.
pub type Rect = (i32, i32, i32, i32);

/// Every box-drawing glyph and how it is built: which arms it has, its stroke
/// weight, and whether its corners are rounded.
///
/// Arms are named by the direction the stroke leaves the stem crossing, so
/// `┌` is `DOWN | RIGHT`: it connects to the cell below and the cell to the
/// right. An arm always runs to the cell edge, which is what makes neighbouring
/// cells join into one line.
const SPECS: &[(char, u8, u8)] = &[
    // Light single line.
    ('│', UP | DOWN, LIGHT),
    ('─', LEFT | RIGHT, LIGHT),
    ('┌', DOWN | RIGHT, LIGHT),
    ('┐', DOWN | LEFT, LIGHT),
    ('└', UP | RIGHT, LIGHT),
    ('┘', UP | LEFT, LIGHT),
    // Double line. Two parallel strokes either side of the same axis, so a
    // `║` still crosses a `╔` at the identical stem positions.
    ('║', UP | DOWN, DOUBLE),
    ('═', LEFT | RIGHT, DOUBLE),
    ('╔', DOWN | RIGHT, DOUBLE),
    ('╗', DOWN | LEFT, DOUBLE),
    ('╚', UP | RIGHT, DOUBLE),
    ('╝', UP | LEFT, DOUBLE),
    // Heavy line. One stroke wider than light, on the same axis.
    ('┃', UP | DOWN, HEAVY),
    ('━', LEFT | RIGHT, HEAVY),
    ('┏', DOWN | RIGHT, HEAVY),
    ('┓', DOWN | LEFT, HEAVY),
    ('┗', UP | RIGHT, HEAVY),
    ('┛', UP | LEFT, HEAVY),
    // Rounded corners on light stems.
    ('╭', DOWN | RIGHT, LIGHT | ROUNDED),
    ('╮', DOWN | LEFT, LIGHT | ROUNDED),
    ('╰', UP | RIGHT, LIGHT | ROUNDED),
    ('╯', UP | LEFT, LIGHT | ROUNDED),
];

/// Stroke offsets perpendicular to the arm, for each weight.
///
/// Light and heavy are contiguous; double is two single strokes with a gap at
/// the axis, which is what reads as "double" at this cell size.
fn offsets(weight: u8) -> &'static [i32] {
    match weight & 3 {
        DOUBLE => &[-1, 1],
        HEAVY => &[0, 1],
        _ => &[0],
    }
}

/// The rectangles that make up `ch`, in cell units.
///
/// Returns `None` for anything that is not a box-drawing glyph, so callers fall
/// back to the font atlas. `SVG` exporters translate these by the cell origin;
/// the rasterizer fills them directly.
pub fn rects_for(ch: char) -> Option<Vec<Rect>> {
    let (_, arms, style) = *SPECS.iter().find(|&&(c, _, _)| c == ch)?;
    let weight = style & 3;
    let rounded = style & ROUNDED != 0;

    // How far each arm extends. A plain arm runs to the cell edge, which is what
    // makes neighbouring cells join into one continuous line. A rounded corner
    // pulls *both* of its arms back by the chamfer radius so the diagonal pixel
    // can join them — shortening only the arms that leave the corner towards the
    // cell interior is what makes `╮` read as a corner rather than a nub.
    let (up_len, down_start, left_len, right_start) = if rounded {
        (
            STEM_Y + 1 - ROUND_R,
            STEM_Y + ROUND_R,
            STEM_X + 1 - ROUND_R,
            STEM_X + ROUND_R,
        )
    } else {
        (STEM_Y + 1, STEM_Y, STEM_X + 1, STEM_X)
    };

    let mut out = Vec::new();
    for &o in offsets(weight) {
        if arms & UP != 0 {
            out.push((STEM_X + o, 0, 1, up_len));
        }
        if arms & DOWN != 0 {
            out.push((STEM_X + o, down_start, 1, CELL_H - down_start));
        }
        if arms & LEFT != 0 {
            out.push((0, STEM_Y + o, left_len, 1));
        }
        if arms & RIGHT != 0 {
            out.push((right_start, STEM_Y + o, CELL_W - right_start, 1));
        }
    }

    if rounded {
        // The chamfer itself: one pixel on the diagonal between the two arm
        // ends, stepped out from the stem crossing towards the open quadrant.
        let sx = if arms & LEFT != 0 { -1 } else { 1 };
        let sy = if arms & UP != 0 { -1 } else { 1 };
        for i in 1..ROUND_R {
            out.push((STEM_X + sx * i, STEM_Y + sy * i, 1, 1));
        }
    }

    Some(out)
}

/// Fill the glyph into an 8x20 alpha buffer (one byte per pixel), which is the
/// size [`crate::render::FontAtlas`] indexes.
///
/// Returns `None` if `ch` is not a box-drawing glyph, so the caller keeps
/// whatever it had rather than drawing a blank cell.
pub fn rasterize(ch: char) -> Option<Vec<u8>> {
    let rects = rects_for(ch)?;
    let mut buf = vec![0u8; (CELL_W * CELL_H) as usize];
    for (x, y, w, h) in rects {
        for py in y..(y + h) {
            for px in x..(x + w) {
                if (0..CELL_W).contains(&px) && (0..CELL_H).contains(&py) {
                    buf[(py * CELL_W + px) as usize] = 255;
                }
            }
        }
    }
    Some(buf)
}

/// The `<rect>` elements for `ch`, in an SVG cell whose pitch is
/// `cell_w` x `cell_h`, with its top-left at `origin_x`,`origin_y`.
///
/// `None` means the glyph is not synthesized and the caller should emit its
/// usual `<text>` element. The geometry is authored against an 8x20 cell and
/// scaled here, because the exporter reads its pitch from the renderer's metrics
/// rather than from these constants — an unscaled rect would collapse to a
/// speck at any other zoom. A degenerate pitch returns `None` for the same
/// reason the rasterizer does: keep the old output rather than draw nothing.
pub fn svg_rects_for(
    ch: char,
    origin_x: f64,
    origin_y: f64,
    cell_w: f64,
    cell_h: f64,
) -> Option<String> {
    let rects = rects_for(ch)?;
    if !(cell_w.is_finite() && cell_h.is_finite()) || cell_w <= 0.0 || cell_h <= 0.0 {
        return None;
    }
    let sx = cell_w / CELL_W as f64;
    let sy = cell_h / CELL_H as f64;

    let mut out = String::new();
    for (x, y, w, h) in rects {
        out.push_str(&format!(
            r##"<rect x="{x}" y="{y}" width="{w}" height="{h}" />"##,
            x = origin_x + x as f64 * sx,
            y = origin_y + y as f64 * sy,
            w = w as f64 * sx,
            h = h as f64 * sy,
        ));
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ink(buf: &[u8], x: i32, y: i32) -> bool {
        buf[(y * CELL_W + x) as usize] > 0
    }

    #[test]
    fn every_specified_glyph_has_rects() {
        for &(ch, _, _) in SPECS {
            assert!(
                !rects_for(ch).unwrap_or_default().is_empty(),
                "{ch} produced no rectangles"
            );
        }
    }

    /// The ISSUE-001 regression: a vertical border is one unbroken line.
    #[test]
    fn vertical_glyph_inks_every_row_of_its_cell() {
        for (ch, weight) in [('│', LIGHT), ('║', DOUBLE), ('┃', HEAVY)] {
            let buf = rasterize(ch).unwrap();
            for &o in offsets(weight) {
                for y in 0..CELL_H {
                    assert!(
                        ink(&buf, STEM_X + o, y),
                        "{ch} leaves row {y} un-inked, so stacked cells would dash"
                    );
                }
            }
        }
    }

    #[test]
    fn horizontal_glyph_inks_every_column_of_its_cell() {
        for (ch, weight) in [('─', LIGHT), ('═', DOUBLE), ('━', HEAVY)] {
            let buf = rasterize(ch).unwrap();
            for &o in offsets(weight) {
                for x in 0..CELL_W {
                    assert!(ink(&buf, x, STEM_Y + o), "{ch} leaves column {x} un-inked");
                }
            }
        }
    }

    /// Each corner must reach the two edges it connects and touch neither of
    /// the two it does not, or borders would sprout nubs or stop short.
    #[test]
    fn corners_reach_only_the_edges_they_connect() {
        // (glyph, left edge, right edge, top edge, bottom edge)
        for (ch, l, r, u, d) in [
            ('┌', false, true, false, true),
            ('┐', true, false, false, true),
            ('└', false, true, true, false),
            ('┘', true, false, true, false),
            ('╭', false, true, false, true),
            ('╮', true, false, false, true),
            ('╰', false, true, true, false),
            ('╯', true, false, true, false),
        ] {
            let buf = rasterize(ch).unwrap();
            let right_col = (CELL_W - 1, STEM_Y);
            let left_col = (0, STEM_Y);
            let top_row = (STEM_X, 0);
            let bottom_row = (STEM_X, CELL_H - 1);
            assert_eq!(ink(&buf, left_col.0, left_col.1), l, "{ch} left edge");
            assert_eq!(ink(&buf, right_col.0, right_col.1), r, "{ch} right edge");
            assert_eq!(ink(&buf, top_row.0, top_row.1), u, "{ch} top edge");
            assert_eq!(ink(&buf, bottom_row.0, bottom_row.1), d, "{ch} bottom edge");
        }
    }

    /// A rectangle's four corners and its edges must share one stem axis,
    /// otherwise the join is visible even though every glyph is continuous.
    #[test]
    fn all_glyphs_share_the_same_stem_axis() {
        for &(ch, _, _) in SPECS {
            let rects = rects_for(ch).unwrap();
            for (x, y, w, h) in rects {
                if h > 1 {
                    // Vertical stroke: it must be centred on STEM_X.
                    assert!(
                        x == STEM_X - 1 || x == STEM_X || x == STEM_X + 1,
                        "{ch} draws a vertical stroke at column {x}, off the stem axis"
                    );
                }
                if w > 1 {
                    assert!(
                        y == STEM_Y - 1 || y == STEM_Y || y == STEM_Y + 1,
                        "{ch} draws a horizontal stroke at row {y}, off the stem axis"
                    );
                }
            }
        }
    }

    /// `rounded` is a distinct style, not an alias for `single`. If the chamfer
    /// ever collapses, `BorderStyle::Rounded` silently becomes `Single` and no
    /// other test here notices.
    #[test]
    fn rounded_corners_differ_from_their_square_siblings() {
        for (round, square) in [('╭', '┌'), ('╮', '┐'), ('╰', '└'), ('╯', '┘')] {
            assert_ne!(
                rasterize(round).unwrap(),
                rasterize(square).unwrap(),
                "{round} rasterizes identically to {square}, so the rounded style is gone"
            );
        }
    }

    #[test]
    fn ascii_and_dotted_members_are_not_synthesized() {
        // `|` is text a user typed; `*` and `.` are how the dotted style breaks.
        for ch in ['|', '-', '+', '*', '.', '#', '╱', '╲', 'A'] {
            assert!(rects_for(ch).is_none(), "{ch} must keep its font glyph");
        }
    }

    #[test]
    fn double_weight_draws_two_parallel_strokes() {
        let buf = rasterize('║').unwrap();
        assert!(ink(&buf, STEM_X - 1, 10) && ink(&buf, STEM_X + 1, 10));
        assert!(!ink(&buf, STEM_X, 10), "double should leave the axis clear");
    }

    #[test]
    fn nothing_is_drawn_outside_the_cell() {
        for &(ch, _, _) in SPECS {
            for (x, y, w, h) in rects_for(ch).unwrap() {
                assert!(x >= 0 && y >= 0, "{ch} starts at {x},{y}");
                assert!(
                    x + w <= CELL_W && y + h <= CELL_H,
                    "{ch} rect {x},{y} {w}x{h} overflows the {CELL_W}x{CELL_H} cell"
                );
            }
        }
    }

    /// The attribute list of each `<rect>` in `svg`, in document order.
    fn rect_tags(svg: &str) -> Vec<&str> {
        svg.split("<rect ")
            .skip(1)
            .map(|tag| tag.split_once("/>").map_or(tag, |(attrs, _)| attrs))
            .collect()
    }

    /// Read one numeric attribute out of a `<rect>` attribute list.
    fn attr(rect: &str, name: &str) -> f64 {
        let head = format!("{name}=\"");
        let (_, rest) = rect.split_once(&head).expect("attribute missing");
        let value = &rest[..rest.find('"').expect("unterminated attribute")];
        value.parse().expect("non-numeric attribute")
    }

    /// The SVG half of ISSUE-001: the exporter put a `font-size` smaller than
    /// the row pitch on the glyph, so a stacked `│` dashed there too. Rects
    /// scaled to the cell pitch cannot.
    ///
    /// Asserted as the *union* of the glyph's rects, since a stem is split at
    /// the crossing (the `│` glyph is a top and a bottom half). What must never
    /// move is the outer bound: row 0 and the next cell's first row.
    #[test]
    fn svg_rects_reach_both_edges_of_the_cell_at_any_pitch() {
        // Exact binary scale factors, so the equality below is not a flake.
        for (cw, chh) in [(8.0, 20.0), (16.0, 40.0)] {
            let svg = svg_rects_for('│', 0.0, 0.0, cw, chh).unwrap();
            let rects = rect_tags(&svg);
            assert!(rects.len() >= 2, "{svg} lost its split at the crossing");
            assert!(
                rects.iter().all(|r| attr(r, "y") >= 0.0),
                "{svg} starts above the cell"
            );
            assert!(
                rects.iter().any(|r| attr(r, "y") == 0.0),
                "{svg} leaves the cell top uncovered"
            );
            assert!(
                rects
                    .iter()
                    .any(|r| attr(r, "y") + attr(r, "height") == chh),
                "{svg} stops short of the next row at pitch {chh}"
            );
        }
    }

    #[test]
    fn svg_falls_back_for_glyphs_and_pitches_it_does_not_own() {
        assert!(svg_rects_for('|', 0.0, 0.0, 8.0, 20.0).is_none());
        assert!(svg_rects_for('A', 0.0, 0.0, 8.0, 20.0).is_none());
        for (cw, chh) in [(0.0, 20.0), (8.0, -1.0), (f64::NAN, 20.0)] {
            assert!(
                svg_rects_for('│', 0.0, 0.0, cw, chh).is_none(),
                "a degenerate cell pitch must not emit invisible rects"
            );
        }
    }

    /// A cell at (3, 2) is 24 px across and 40 px down, and the horizontal stem
    /// sits at row 7 of that cell — the same axis the canvas atlas inks.
    #[test]
    fn svg_rects_are_placed_at_the_cell_origin() {
        let svg = svg_rects_for('─', 24.0, 40.0, 8.0, 20.0).unwrap();
        let rects = rect_tags(&svg);
        assert_eq!(rects.len(), 2, "{svg}");
        for rect in &rects {
            assert_eq!(attr(rect, "y"), 47.0, "{rect} is off the stem axis");
            assert_eq!(attr(rect, "height"), 1.0, "{rect}");
        }
        assert_eq!(attr(rects[0], "x"), 24.0, "{svg}");
        assert_eq!(attr(rects[0], "width"), 4.0, "{svg}");
        assert_eq!(
            attr(rects[1], "x") + attr(rects[1], "width"),
            32.0,
            "{svg} stops short of the next column"
        );
    }
}
