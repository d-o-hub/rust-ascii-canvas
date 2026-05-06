//! Bitmap font renderer for WASM pixel buffer.
//!
//! Provides a precomputed font atlas for common ASCII characters
//! and methods to render them into a raw RGBA buffer.

use std::collections::HashMap;

/// A simple 8x14 monospace bitmap font.
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
    pub fn update_glyph(&mut self, ch: char, glyph_data: &[u8]) {
        if let Some(&idx) = self.glyph_indices.get(&ch) {
            let offset = idx * self.glyph_width * self.glyph_height;
            if offset + glyph_data.len() <= self.data.len() {
                self.data[offset..offset + glyph_data.len()].copy_from_slice(glyph_data);
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
            Self::render_glyph_placeholder(&mut glyph_data, ch);
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
                            if mask == 255 {
                                buffer[pixel_idx..pixel_idx + 3].copy_from_slice(&color[0..3]);
                            } else {
                                let alpha = mask as f32 / 255.0;
                                let inv_alpha = 1.0 - alpha;

                                buffer[pixel_idx] = (color_f[0] * alpha + buffer[pixel_idx] as f32 * inv_alpha) as u8;
                                buffer[pixel_idx + 1] = (color_f[1] * alpha + buffer[pixel_idx + 1] as f32 * inv_alpha) as u8;
                                buffer[pixel_idx + 2] = (color_f[2] * alpha + buffer[pixel_idx + 2] as f32 * inv_alpha) as u8;
                            }
                            buffer[pixel_idx + 3] = 255;
                        }
                    }
                }
            }
        }
    }

    fn render_glyph_placeholder(glyph_data: &mut [u8], ch: char) {
        match ch {
            ' ' => {}
            '|' | '│' | '┃' | '║' => {
                for y in 0..20 {
                    glyph_data[y * 8 + 4] = 255;
                }
            }
            '-' | '─' | '━' | '═' => {
                glyph_data[10 * 8..10 * 8 + 8].fill(255);
            }
            '+' => {
                for y in 0..20 {
                    glyph_data[y * 8 + 4] = 255;
                }
                glyph_data[10 * 8..10 * 8 + 8].fill(255);
            }
            '┌' | '┏' | '╔' | '╭' => {
                glyph_data[10 * 8 + 4..10 * 8 + 8].fill(255);
                for y in 10..20 {
                    glyph_data[y * 8 + 4] = 255;
                }
            }
            '┐' | '┓' | '╗' | '╮' => {
                glyph_data[10 * 8..10 * 8 + 4].fill(255);
                for y in 10..20 {
                    glyph_data[y * 8 + 4] = 255;
                }
            }
            '└' | '┗' | '╚' | '╰' => {
                glyph_data[10 * 8 + 4..10 * 8 + 8].fill(255);
                for y in 0..10 {
                    glyph_data[y * 8 + 4] = 255;
                }
            }
            '┘' | '┛' | '╝' | '╯' => {
                glyph_data[10 * 8..10 * 8 + 4].fill(255);
                for y in 0..10 {
                    glyph_data[y * 8 + 4] = 255;
                }
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

    #[test]
    fn test_font_atlas_new() {
        let atlas = FontAtlas::new();
        assert_eq!(atlas.glyph_width, 8);
        assert_eq!(atlas.glyph_height, 20);
        assert!(atlas.glyph_indices.contains_key(&'A'));
        assert!(atlas.glyph_indices.contains_key(&'┌'));
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
}
