//! Selection, clipboard, and move operations for WASM.

use wasm_bindgen::prelude::*;

use super::bindings::AsciiEditor;

#[wasm_bindgen]
impl AsciiEditor {
    /// Selects the entire canvas area.
    #[wasm_bindgen(js_name = selectAll)]
    pub fn select_all(&mut self) {
        self.select_all_impl();
    }

    /// Copies the active selection or the full grid content to the internal clipboard.
    /// Returns true if something was copied.
    #[wasm_bindgen(js_name = copySelection)]
    pub fn copy_selection(&mut self) -> bool {
        self.copy_selection_impl()
    }

    /// Cuts the active selection from the canvas and places it on the internal clipboard.
    /// Returns true if successful.
    #[wasm_bindgen(js_name = cutSelection)]
    pub fn cut_selection(&mut self) -> bool {
        self.cut_selection_impl()
    }

    /// Pastes content from the internal clipboard onto the active canvas.
    /// Returns true if successful.
    #[wasm_bindgen]
    pub fn paste(&mut self) -> bool {
        self.paste_impl()
    }

    /// Pastes external plain text at the cursor or selection origin.
    /// CRLF/LF line endings are normalized to grid rows.
    /// Respects canvas bounds and avoids clobbering with space/whitespace runs.
    /// Returns true if any characters were pasted successfully.
    #[wasm_bindgen(js_name = pasteText)]
    pub fn paste_text(&mut self, text: String) -> bool {
        self.paste_text_impl(&text)
    }

    /// Deletes the currently selected area, replacing cell content with spaces.
    /// Returns true if successful.
    #[wasm_bindgen(js_name = deleteSelection)]
    pub fn delete_selection(&mut self) -> bool {
        self.delete_selection_impl()
    }

    /// Returns whether the internal editor clipboard contains any elements.
    #[wasm_bindgen(getter)]
    pub fn has_clipboard(&self) -> bool {
        !self.clipboard.is_empty()
    }

    /// Returns whether there is an active selection region on the canvas.
    #[wasm_bindgen(getter)]
    pub fn has_selection(&self) -> bool {
        self.current_selection.is_some()
    }
}
