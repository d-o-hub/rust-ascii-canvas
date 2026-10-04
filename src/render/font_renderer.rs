//! Bitmap font renderer for WASM pixel buffer.
//!
//! Provides a precomputed font atlas for common ASCII characters
//! and methods to render them into a raw RGBA buffer.
//!
//! Box-drawing glyphs are the exception: they are synthesized from the geometry
//! in [`crate::render::box_drawing`] rather than taken from a font, at build
//! time *and* whenever the browser re-uploads a raster. See that module for why.

use super::box_drawing;
use std::collections::HashMap;

/// An 8x20 monospace bitmap font.
pub struct FontAtlas {
    /// Glyph width in pixels
    pub glyph_width: usize,
    /// Glyph height in pixels
    pub glyph_height: usize,
    /// Map from character to index in data
    glyph_indices: HashMap<char, usize>,
    /// Raw alpha mask data (1 byte per pixel)
    data: Vec<u8>,
}

impl FontAtlas {
    /// Update the entire font atlas data.
    pub fn update_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }

    /// Update a specific glyph's bitmap data.
    ///
    /// A box-drawing codepoint ignores the bytes it is given and re-synthesizes
    /// from geometry: the browser rasterizes its font at 13px into a 20px cell,
    /// so accepting that raster is how `│` came to render as a dashed line
    /// (dogfood ISSUE-001).
    pub fn update_glyph(&mut self, ch: char, glyph_data: &[u8]) {
        if let Some(&idx) = self.glyph_indices.get(&ch) {
            let synthesized = box_drawing::rasterize(ch);
            let bytes = synthesized.as_deref().unwrap_or(glyph_data);
            let offset = idx * self.glyph_width * self.glyph_height;
            if offset + bytes.len() <= self.data.len() {
                self.data[offset..offset + bytes.len()].copy_from_slice(bytes);
            }
        }
    }
}

impl Default for FontAtlas {
    fn default() -> Self {
        Self::new()
    }
}

impl FontAtlas {
    /// Create a new font atlas with a simple embedded font.
    pub fn new() -> Self {
        let glyph_width = 8;
        let glyph_height = 20;
        let mut glyph_indices = HashMap::new();
        let mut data = Vec::new();

        let add_glyph = |ch: char, data: &mut Vec<u8>, indices: &mut HashMap<char, usize>| {
            let idx = indices.len();
            indices.insert(ch, idx);
            let mut glyph_data = vec![0u8; glyph_width * glyph_height];
            match box_drawing::rasterize(ch) {
                Some(synthesized) => glyph_data.copy_from_slice(&synthesized),
                None => Self::render_glyph_placeholder(&mut glyph_data, ch),
            }
            data.extend(glyph_data);
        };

        // Basic ASCII
        for c in 32..127u8 {
            add_glyph(c as char, &mut data, &mut glyph_indices);
        }

        // Box drawing characters and symbols
        let box_chars = [
            '┌', '┐', '└', '┘', '─', '│', '╔', '╗', '╚', '╝', '═', '║', '┏', '┓', '┗', '┛', '━',
            '┃', '╭', '╮', '╰', '╯', '+', '-', '|', '*', '·', '•', '●', '▲', '▼', '◄', '►', '╱',
            '╲', '◆',
        ];
        for &ch in &box_chars {
            if !glyph_indices.contains_key(&ch) {
                add_glyph(ch, &mut data, &mut glyph_indices);
            }
        }

        Self {
            glyph_width,
            glyph_height,
            glyph_indices,
            data,
        }
    }

    /// Render a glyph into the pixel buffer.
    pub fn render_glyph(
        &self,
        buffer: &mut [u8],
        buffer_width: usize,
        x: usize,
        y: usize,
        ch: char,
        color: [u8; 4],
    ) {
        if let Some(&idx) = self
            .glyph_indices
            .get(&ch)
            .or_else(|| self.glyph_indices.get(&'?'))
        {
            let glyph_offset = idx * self.glyph_width * self.glyph_height;
            let color_f = [color[0] as f32, color[1] as f32, color[2] as f32];

            for gy in 0..self.glyph_height {
                let buffer_y = y + gy;
                let glyph_row_offset = glyph_offset + gy * self.glyph_width;
                let buffer_row_start = (buffer_y * buffer_width + x) * 4;

                for gx in 0..self.glyph_width {
                    let mask = self.data[glyph_row_offset + gx];
                    if mask > 0 {
                        let pixel_idx = buffer_row_start + gx * 4;

                        if pixel_idx + 3 < buffer.len() {
                            let effective_alpha = (mask as f32 / 255.0) * (color[3] as f32 / 255.0);
                            if effective_alpha >= 1.0 {
                                buffer[pixel_idx..pixel_idx + 3].copy_from_slice(&color[0..3]);
                            } else {
                                let inv_alpha = 1.0 - effective_alpha;

                                buffer[pixel_idx] = (color_f[0] * effective_alpha
                                    + buffer[pixel_idx] as f32 * inv_alpha)
                                    as u8;
                                buffer[pixel_idx + 1] = (color_f[1] * effective_alpha
                                    + buffer[pixel_idx + 1] as f32 * inv_alpha)
                                    as u8;
                                buffer[pixel_idx + 2] = (color_f[2] * effective_alpha
                                    + buffer[pixel_idx + 2] as f32 * inv_alpha)
                                    as u8;
                            }
                            buffer[pixel_idx + 3] = 255;
                        }
                    }
                }
            }
        }
    }

    /// Draw `ch` into an 8x20 alpha buffer.
    ///
    /// Box-drawing glyphs are absent by design: they are synthesized by
    /// [`super::box_drawing`] and never reach here. The `|`, `-` and `+` arms are
    /// the ASCII and dotted border styles, which stay font glyphs.
    fn render_glyph_placeholder(glyph_data: &mut [u8], ch: char) {
        match ch {
            ' ' => {}
            '|' => {
                for y in 0..20 {
                    glyph_data[y * 8 + 4] = 255;
                }
            }
            '-' => {
                glyph_data[10 * 8..10 * 8 + 8].fill(255);
            }
            '+' => {
                for y in 0..20 {
                    glyph_data[y * 8 + 4] = 255;
                }
                glyph_data[10 * 8..10 * 8 + 8].fill(255);
            }
            '#' => {
                for y in 4..16 {
                    glyph_data[y * 8 + 1..y * 8 + 7].fill(255);
                }
            }
            '0'..='9' | 'A'..='Z' | 'a'..='z' => {
                // Minimal I-beam fallback for characters
                for y in 4..16 {
                    glyph_data[y * 8 + 4] = 128;
                }
                glyph_data[4 * 8 + 2..4 * 8 + 7].fill(128);
                glyph_data[15 * 8 + 2..15 * 8 + 7].fill(128);
            }
            '·' => {
                glyph_data[7 * 8 + 4] = 255;
            }
            '•' => {
                glyph_data[6 * 8 + 4] = 255;
                glyph_data[7 * 8 + 3..7 * 8 + 6].fill(255);
                glyph_data[8 * 8 + 4] = 255;
            }
            '●' | '◆' => {
                for y in 6..14 {
                    glyph_data[y * 8 + 2..y * 8 + 6].fill(255);
                }
            }
            '▲' => {
                for y in 5..15 {
                    let w = (y - 5) / 2;
                    let start = 4 - w;
                    let end = (4 + w + 1).min(8);
                    if start < end {
                        glyph_data[y * 8 + start..y * 8 + end].fill(255);
                    }
                }
            }
            '▼' => {
                for y in 5..15 {
                    let w = (14 - y) / 2;
                    let start = 4 - w;
                    let end = (4 + w + 1).min(8);
                    if start < end {
                        glyph_data[y * 8 + start..y * 8 + end].fill(255);
                    }
                }
            }
            '◄' => {
                for x in 1..7 {
                    let h = (x - 1) * 3 / 2;
                    let start = 10 - h;
                    let end = (10 + h + 1).min(20);
                    for y in start..end {
                        glyph_data[y * 8 + x] = 255;
                    }
                }
            }
            '►' => {
                for x in 1..7 {
                    let h = (6 - x) * 3 / 2;
                    let start = 10 - h;
                    let end = (10 + h + 1).min(20);
                    for y in start..end {
                        glyph_data[y * 8 + x] = 255;
                    }
                }
            }
            '╱' => {
                for i in 0..8 {
                    let y = 14 - (i * 10 / 8);
                    if y < 20 {
                        glyph_data[y * 8 + i] = 255;
                    }
                }
            }
            '╲' => {
                for i in 0..8 {
                    let y = 6 + (i * 10 / 8);
                    if y < 20 {
                        glyph_data[y * 8 + i] = 255;
                    }
                }
            }
            _ => {
                if !ch.is_whitespace() {
                    glyph_data[8 + 1..8 + 7].fill(128);
                    glyph_data[12 * 8 + 1..12 * 8 + 7].fill(128);
                    for y in 1..13 {
                        glyph_data[y * 8 + 1] = 128;
                        glyph_data[y * 8 + 6] = 128;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The alpha mask the atlas would actually blit for `ch`.
    fn glyph(atlas: &FontAtlas, ch: char) -> Vec<u8> {
        let idx = *atlas
            .glyph_indices
            .get(&ch)
            .unwrap_or_else(|| panic!("{ch} is missing from the atlas"));
        let start = idx * atlas.glyph_width * atlas.glyph_height;
        atlas.data[start..start + atlas.glyph_width * atlas.glyph_height].to_vec()
    }

    #[test]
    fn test_font_atlas_new() {
        let atlas = FontAtlas::new();
        assert_eq!(atlas.glyph_width, 8);
        assert_eq!(atlas.glyph_height, 20);
        assert!(atlas.glyph_indices.contains_key(&'A'));
        assert!(atlas.glyph_indices.contains_key(&'┌'));
    }

    /// `new()` blits the synthesized buffer with `copy_from_slice`, which panics
    /// on a length mismatch. Pin the two constants together before it does.
    #[test]
    fn atlas_cell_matches_the_box_drawing_geometry() {
        let atlas = FontAtlas::new();
        assert_eq!(atlas.glyph_width, box_drawing::CELL_W as usize);
        assert_eq!(atlas.glyph_height, box_drawing::CELL_H as usize);
    }

    /// ISSUE-001 at the atlas boundary: a vertical border glyph must ink every
    /// row, because the blit starts a fresh glyph at each row boundary.
    #[test]
    fn synthesized_vertical_glyphs_cover_the_full_cell_height() {
        let atlas = FontAtlas::new();
        for ch in ['│', '║', '┃'] {
            let mask = glyph(&atlas, ch);
            for y in 0..20 {
                assert!(
                    mask.iter().skip(y * 8).take(8).any(|&a| a > 0),
                    "{ch} leaves atlas row {y} blank, so a stacked border dashes there"
                );
            }
        }
    }

    /// The regression the browser used to reintroduce: `uploadFontAtlas()` sends
    /// a 13px raster whose ink stops short of the cell. The atlas must ignore it.
    #[test]
    fn uploading_a_font_raster_cannot_break_a_box_drawing_glyph() {
        let mut atlas = FontAtlas::new();
        // Exactly what a font gives at 13px in a 20px cell: rows 2..17 only.
        let mut dashed = vec![0u8; 8 * 20];
        for y in 2..17 {
            dashed[y * 8 + 2] = 255;
        }
        atlas.update_glyph('│', &dashed);

        assert_eq!(glyph(&atlas, '│'), glyph(&FontAtlas::new(), '│'));
    }

    /// …while an ordinary glyph still takes the uploaded bytes verbatim, or the
    /// fix would have replaced the whole font instead of one class of glyph.
    #[test]
    fn non_box_drawing_uploads_still_land() {
        let mut atlas = FontAtlas::new();
        let custom = vec![7u8; 8 * 20];
        atlas.update_glyph('A', &custom);
        assert_eq!(glyph(&atlas, 'A'), custom);
    }

    #[test]
    fn test_render_glyph_placeholder() {
        let mut glyph_data = vec![0u8; 8 * 20];
        FontAtlas::render_glyph_placeholder(&mut glyph_data, '-');
        // Row 10 should be filled with 255
        for x in 0..8 {
            assert_eq!(glyph_data[10 * 8 + x], 255);
        }

        glyph_data.fill(0);
        FontAtlas::render_glyph_placeholder(&mut glyph_data, '#');
        for y in 4..16 {
            for x in 1..7 {
                assert_eq!(glyph_data[y * 8 + x], 255);
            }
        }
    }

    #[test]
    fn test_render_glyph() {
        let mut atlas = FontAtlas::new();
        // Manually update '?' glyph to have known mask values
        let mut custom_mask = vec![0u8; 8 * 20];
        custom_mask[0] = 255; // Opaque fast-path
        custom_mask[1] = 128; // Alpha-blend path (approx 50%)
        atlas.update_glyph('?', &custom_mask);

        let mut buffer = vec![0u8; 8 * 20 * 4];
        let fg_color = [200, 100, 50, 255];

        // Background is [0, 0, 0, 0]
        atlas.render_glyph(&mut buffer, 8, 0, 0, '?', fg_color);

        // Check opaque pixel (mask == 255)
        assert_eq!(buffer[0], fg_color[0]);
        assert_eq!(buffer[1], fg_color[1]);
        assert_eq!(buffer[2], fg_color[2]);
        assert_eq!(buffer[3], 255);

        // Check blended pixel (mask == 128)
        // alpha = 128 / 255.0 approx 0.50196
        // Expected = color * alpha + background * (1 - alpha)
        // Since background is 0: expected = color * alpha
        let alpha = 128.0 / 255.0;
        assert_eq!(buffer[4], (fg_color[0] as f32 * alpha) as u8);
        assert_eq!(buffer[5], (fg_color[1] as f32 * alpha) as u8);
        assert_eq!(buffer[6], (fg_color[2] as f32 * alpha) as u8);
        assert_eq!(buffer[7], 255);
    }

    #[test]
    fn test_render_glyph_semi_transparent() {
        let mut atlas = FontAtlas::new();
        let mut custom_mask = vec![0u8; 8 * 20];
        custom_mask[0] = 255; // Mask is opaque
        atlas.update_glyph('?', &custom_mask);

        let mut buffer = vec![0u8; 8 * 20 * 4];
        let fg_color = [200, 100, 50, 128]; // Semi-transparent (approx 50% opacity)

        atlas.render_glyph(&mut buffer, 8, 0, 0, '?', fg_color);

        // effective_alpha = 1.0 * (128.0 / 255.0) approx 0.50196
        let effective_alpha = 128.0 / 255.0;
        // Background is 0, so expected = fg_color * effective_alpha
        assert_eq!(buffer[0], (fg_color[0] as f32 * effective_alpha) as u8);
        assert_eq!(buffer[1], (fg_color[1] as f32 * effective_alpha) as u8);
        assert_eq!(buffer[2], (fg_color[2] as f32 * effective_alpha) as u8);
        assert_eq!(buffer[3], 255);
    }
}
