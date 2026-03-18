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

        // Box drawing characters
        let box_chars = [
            '┌', '┐', '└', '┘', '─', '│',
            '╔', '╗', '╚', '╝', '═', '║',
            '┏', '┓', '┗', '┛', '━', '┃',
            '╭', '╮', '╰', '╯',
            '+', '-', '|', '*', '·', '•', '●'
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
        if let Some(&idx) = self.glyph_indices.get(&ch).or_else(|| self.glyph_indices.get(&'?')) {
            let glyph_offset = idx * self.glyph_width * self.glyph_height;

            for gy in 0..self.glyph_height {
                let buffer_y = y + gy;
                let glyph_row_offset = glyph_offset + gy * self.glyph_width;
                let buffer_row_start = (buffer_y * buffer_width + x) * 4;

                for gx in 0..self.glyph_width {
                    let mask = self.data[glyph_row_offset + gx];
                    if mask > 0 {
                        let pixel_idx = buffer_row_start + gx * 4;

                        if pixel_idx + 3 < buffer.len() {
                            let alpha = mask as f32 / 255.0;
                            for i in 0..3 {
                                buffer[pixel_idx + i] = ((color[i] as f32 * alpha) + (buffer[pixel_idx + i] as f32 * (1.0 - alpha))) as u8;
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
            ' ' => {},
            '|' | '│' | '┃' | '║' => {
                for y in 0..20 { glyph_data[y * 8 + 4] = 255; }
            },
            '-' | '─' | '━' | '═' => {
                for x in 0..8 { glyph_data[10 * 8 + x] = 255; }
            },
            '+' => {
                for y in 0..20 { glyph_data[y * 8 + 4] = 255; }
                for x in 0..8 { glyph_data[10 * 8 + x] = 255; }
            },
            '┌' | '┏' | '╔' | '╭' => {
                for x in 4..8 { glyph_data[10 * 8 + x] = 255; }
                for y in 10..20 { glyph_data[y * 8 + 4] = 255; }
            },
            '┐' | '┓' | '╗' | '╮' => {
                for x in 0..4 { glyph_data[10 * 8 + x] = 255; }
                for y in 10..20 { glyph_data[y * 8 + 4] = 255; }
            },
            '└' | '┗' | '╚' | '╰' => {
                for x in 4..8 { glyph_data[10 * 8 + x] = 255; }
                for y in 0..10 { glyph_data[y * 8 + 4] = 255; }
            },
            '┘' | '┛' | '╝' | '╯' => {
                for x in 0..4 { glyph_data[10 * 8 + x] = 255; }
                for y in 0..10 { glyph_data[y * 8 + 4] = 255; }
            },
            '#' => {
                for y in 4..16 {
                    for x in 1..7 {
                        glyph_data[y * 8 + x] = 255;
                    }
                }
            },
            '0'..='9' | 'A'..='Z' | 'a'..='z' => {
                for x in 2..6 {
                    glyph_data[4 * 8 + x] = 255;
                    glyph_data[15 * 8 + x] = 255;
                }
                for y in 4..16 {
                    glyph_data[y * 8 + 2] = 255;
                    glyph_data[y * 8 + 5] = 255;
                }
                glyph_data[9 * 8 + 3] = 255;
                glyph_data[9 * 8 + 4] = 255;
                glyph_data[10 * 8 + 3] = 255;
                glyph_data[10 * 8 + 4] = 255;
            },
            '·' => {
                glyph_data[7 * 8 + 4] = 255;
            },
            '•' => {
                glyph_data[6 * 8 + 4] = 255;
                glyph_data[7 * 8 + 3] = 255;
                glyph_data[7 * 8 + 4] = 255;
                glyph_data[7 * 8 + 5] = 255;
                glyph_data[8 * 8 + 4] = 255;
            },
            '●' => {
                for y in 6..9 {
                    for x in 3..6 {
                        glyph_data[y * 8 + x] = 255;
                    }
                }
            },
            _ => {
                if !ch.is_whitespace() {
                    for x in 1..7 {
                        glyph_data[1 * 8 + x] = 128;
                        glyph_data[12 * 8 + x] = 128;
                    }
                    for y in 1..13 {
                        glyph_data[y * 8 + 1] = 128;
                        glyph_data[y * 8 + 6] = 128;
                    }
                }
            }
        }
    }
}
