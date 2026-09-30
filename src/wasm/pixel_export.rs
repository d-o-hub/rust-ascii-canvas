//! Clean committed-artwork export, independent of the interactive pixel buffer.

use super::{render_api::parse_hex_color, AsciiEditor};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl AsciiEditor {
    /// Return owned RGBA pixels (Uint8Array): width*8 by height*20, row-major.
    /// Uses the editor's current theme and glyph atlas, with visible committed
    /// layers only. No selection, preview, pan or zoom is included. This does not
    /// modify tool state, history, dirty flags, metrics, or the display buffer.
    #[wasm_bindgen(js_name = exportPixelBuffer)]
    pub fn export_pixel_buffer(&self) -> Vec<u8> {
        let grid = self.composite_visible_grid();
        let width = grid.width() * 8;
        let bg = parse_hex_color(&self.theme.background).unwrap_or([30, 30, 30, 255]);
        let fg = parse_hex_color(&self.theme.foreground).unwrap_or([212, 212, 212, 255]);
        let mut pixels = vec![0; width * grid.height() * 20 * 4];
        for pixel in pixels.chunks_exact_mut(4) {
            pixel.copy_from_slice(&bg);
        }
        for (x, y, cell) in grid.iter_with_coords() {
            if cell.is_visible() {
                self.font_atlas.render_glyph(
                    &mut pixels,
                    width,
                    x as usize * 8,
                    y as usize * 20,
                    cell.ch,
                    fg,
                );
            }
        }
        pixels
    }
}
