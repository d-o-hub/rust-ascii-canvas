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

        // Box drawing characters
        let box_chars = [
            '┌', '┐', '└', '┘', '─', '│', '╔', '╗', '╚', '╝', '═', '║', '┏', '┓', '┗', '┛', '━',
            '┃', '╭', '╮', '╰', '╯', '+', '-', '|', '*', '·', '•', '●',
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
                                buffer[pixel_idx + i] = ((color[i] as f32 * alpha)
                                    + (buffer[pixel_idx + i] as f32 * (1.0 - alpha)))
                                    as u8;
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
                for x in 0..8 {
                    glyph_data[10 * 8 + x] = 255;
                }
            }
            '+' => {
                for y in 0..20 {
                    glyph_data[y * 8 + 4] = 255;
                }
                for x in 0..8 {
                    glyph_data[10 * 8 + x] = 255;
                }
            }
            '┌' | '┏' | '╔' | '╭' => {
                for x in 4..8 {
                    glyph_data[10 * 8 + x] = 255;
                }
                for y in 10..20 {
                    glyph_data[y * 8 + 4] = 255;
                }
            }
            '┐' | '┓' | '╗' | '╮' => {
                for x in 0..4 {
                    glyph_data[10 * 8 + x] = 255;
                }
                for y in 10..20 {
                    glyph_data[y * 8 + 4] = 255;
                }
            }
            '└' | '┗' | '╚' | '╰' => {
                for x in 4..8 {
                    glyph_data[10 * 8 + x] = 255;
                }
                for y in 0..10 {
                    glyph_data[y * 8 + 4] = 255;
                }
            }
            '┘' | '┛' | '╝' | '╯' => {
                for x in 0..4 {
                    glyph_data[10 * 8 + x] = 255;
                }
                for y in 0..10 {
                    glyph_data[y * 8 + 4] = 255;
                }
            }
            '#' => {
                for y in 4..16 {
                    for x in 1..7 {
                        glyph_data[y * 8 + x] = 255;
                    }
                }
            }
            '0'..='9' | 'A'..='Z' | 'a'..='z' => {
                // Simple 5x7 bitmap representation for alphanumeric characters
                // Embedded in a 8x20 cell, centered at (1, 6)
                let pixels: &[u8] = match ch {
                    '0' => &[0x70, 0x88, 0x88, 0x88, 0x88, 0x88, 0x70],
                    '1' => &[0x20, 0x60, 0x20, 0x20, 0x20, 0x20, 0x70],
                    '2' => &[0x70, 0x88, 0x08, 0x10, 0x20, 0x40, 0xf8],
                    '3' => &[0xf8, 0x10, 0x20, 0x10, 0x08, 0x88, 0x70],
                    '4' => &[0x10, 0x30, 0x50, 0x90, 0xf8, 0x10, 0x10],
                    '5' => &[0xf8, 0x80, 0xf0, 0x08, 0x08, 0x88, 0x70],
                    '6' => &[0x70, 0x80, 0xf0, 0x88, 0x88, 0x88, 0x70],
                    '7' => &[0xf8, 0x08, 0x10, 0x20, 0x40, 0x40, 0x40],
                    '8' => &[0x70, 0x88, 0x70, 0x88, 0x88, 0x88, 0x70],
                    '9' => &[0x70, 0x88, 0x88, 0x78, 0x08, 0x10, 0x60],
                    'A' => &[0x20, 0x50, 0x88, 0x88, 0xf8, 0x88, 0x88],
                    'B' => &[0xf0, 0x88, 0xf0, 0x88, 0x88, 0x88, 0xf0],
                    'C' => &[0x70, 0x88, 0x80, 0x80, 0x80, 0x88, 0x70],
                    'D' => &[0xf0, 0x88, 0x88, 0x88, 0x88, 0x88, 0xf0],
                    'E' => &[0xf8, 0x80, 0xf0, 0x80, 0x80, 0x80, 0xf8],
                    'F' => &[0xf8, 0x80, 0xf0, 0x80, 0x80, 0x80, 0x80],
                    'G' => &[0x70, 0x88, 0x80, 0x98, 0x88, 0x88, 0x70],
                    'H' => &[0x88, 0x88, 0x88, 0xf8, 0x88, 0x88, 0x88],
                    'I' => &[0x70, 0x20, 0x20, 0x20, 0x20, 0x20, 0x70],
                    'J' => &[0x38, 0x08, 0x08, 0x08, 0x08, 0x88, 0x70],
                    'K' => &[0x88, 0x90, 0xa0, 0xc0, 0xa0, 0x90, 0x88],
                    'L' => &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0xf8],
                    'M' => &[0x88, 0xd8, 0xa8, 0xa8, 0x88, 0x88, 0x88],
                    'N' => &[0x88, 0xc8, 0xa8, 0x98, 0x88, 0x88, 0x88],
                    'O' => &[0x70, 0x88, 0x88, 0x88, 0x88, 0x88, 0x70],
                    'P' => &[0xf0, 0x88, 0x88, 0xf0, 0x80, 0x80, 0x80],
                    'Q' => &[0x70, 0x88, 0x88, 0x88, 0xa8, 0x90, 0x68],
                    'R' => &[0xf0, 0x88, 0x88, 0xf0, 0xa0, 0x90, 0x88],
                    'S' => &[0x70, 0x88, 0x40, 0x20, 0x10, 0x88, 0x70],
                    'T' => &[0xf8, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20],
                    'U' => &[0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x70],
                    'V' => &[0x88, 0x88, 0x88, 0x88, 0x88, 0x50, 0x20],
                    'W' => &[0x88, 0x88, 0x88, 0xa8, 0xa8, 0xd8, 0x88],
                    'X' => &[0x88, 0x88, 0x50, 0x20, 0x50, 0x88, 0x88],
                    'Y' => &[0x88, 0x88, 0x50, 0x20, 0x20, 0x20, 0x20],
                    'Z' => &[0xf8, 0x08, 0x10, 0x20, 0x40, 0x80, 0xf8],
                    'a' => &[0x00, 0x00, 0x70, 0x08, 0x78, 0x88, 0x78],
                    'b' => &[0x80, 0x80, 0xf0, 0x88, 0x88, 0x88, 0xf0],
                    'c' => &[0x00, 0x00, 0x70, 0x80, 0x80, 0x88, 0x70],
                    'd' => &[0x08, 0x08, 0x78, 0x88, 0x88, 0x88, 0x78],
                    'e' => &[0x00, 0x00, 0x70, 0x88, 0xf8, 0x80, 0x70],
                    'f' => &[0x30, 0x40, 0xf0, 0x40, 0x40, 0x40, 0x40],
                    'g' => &[0x00, 0x00, 0x78, 0x88, 0x78, 0x08, 0x70],
                    'h' => &[0x80, 0x80, 0xf0, 0x88, 0x88, 0x88, 0x88],
                    'i' => &[0x20, 0x00, 0x60, 0x20, 0x20, 0x20, 0x70],
                    'j' => &[0x10, 0x00, 0x10, 0x10, 0x10, 0x90, 0x60],
                    'k' => &[0x80, 0x80, 0x90, 0xa0, 0xc0, 0xa0, 0x90],
                    'l' => &[0x60, 0x20, 0x20, 0x20, 0x20, 0x20, 0x70],
                    'm' => &[0x00, 0x00, 0xd0, 0xa8, 0xa8, 0xa8, 0xa8],
                    'n' => &[0x00, 0x00, 0xf0, 0x88, 0x88, 0x88, 0x88],
                    'o' => &[0x00, 0x00, 0x70, 0x88, 0x88, 0x88, 0x70],
                    'p' => &[0x00, 0x00, 0xf0, 0x88, 0xf0, 0x80, 0x80],
                    'q' => &[0x00, 0x00, 0x78, 0x88, 0x78, 0x08, 0x08],
                    'r' => &[0x00, 0x00, 0xb0, 0xc8, 0x80, 0x80, 0x80],
                    's' => &[0x00, 0x00, 0x78, 0x80, 0x70, 0x08, 0xf0],
                    't' => &[0x40, 0x40, 0xf0, 0x40, 0x40, 0x48, 0x30],
                    'u' => &[0x00, 0x00, 0x88, 0x88, 0x88, 0x98, 0x68],
                    'v' => &[0x00, 0x00, 0x88, 0x88, 0x88, 0x50, 0x20],
                    'w' => &[0x00, 0x00, 0x88, 0xa8, 0xa8, 0xa8, 0x50],
                    'x' => &[0x00, 0x00, 0x88, 0x50, 0x20, 0x50, 0x88],
                    'y' => &[0x00, 0x00, 0x88, 0x88, 0x78, 0x08, 0x70],
                    'z' => &[0x00, 0x00, 0xf8, 0x10, 0x20, 0x40, 0xf8],
                    _ => &[0x00; 7],
                };

                for (iy, &row) in pixels.iter().enumerate() {
                    for ix in 0..5 {
                        if (row & (0x80 >> ix)) != 0 {
                            glyph_data[(iy + 6) * 8 + (ix + 1)] = 255;
                        }
                    }
                }
            }
            '·' => {
                glyph_data[7 * 8 + 4] = 255;
            }
            '•' => {
                glyph_data[6 * 8 + 4] = 255;
                glyph_data[7 * 8 + 3] = 255;
                glyph_data[7 * 8 + 4] = 255;
                glyph_data[7 * 8 + 5] = 255;
                glyph_data[8 * 8 + 4] = 255;
            }
            '●' => {
                for y in 6..9 {
                    for x in 3..6 {
                        glyph_data[y * 8 + x] = 255;
                    }
                }
            }
            _ => {
                if !ch.is_whitespace() {
                    for x in 1..7 {
                        glyph_data[8 + x] = 128;
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
