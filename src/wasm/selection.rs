//! Selection, clipboard, and move operations for WASM.

#![allow(missing_docs)]

use wasm_bindgen::prelude::*;

use super::bindings::AsciiEditor;

#[wasm_bindgen]
impl AsciiEditor {
    #[wasm_bindgen(js_name = selectAll)]
    pub fn select_all(&mut self) {
        self.select_all_impl();
    }

    #[wasm_bindgen(js_name = copySelection)]
    pub fn copy_selection(&mut self) -> bool {
        self.copy_selection_impl()
    }

    #[wasm_bindgen(js_name = cutSelection)]
    pub fn cut_selection(&mut self) -> bool {
        self.cut_selection_impl()
    }

    #[wasm_bindgen]
    pub fn paste(&mut self) -> bool {
        self.paste_impl()
    }

    #[wasm_bindgen(js_name = deleteSelection)]
    pub fn delete_selection(&mut self) -> bool {
        self.delete_selection_impl()
    }

    #[wasm_bindgen(getter)]
    pub fn has_clipboard(&self) -> bool {
        !self.clipboard.is_empty()
    }

    #[wasm_bindgen(getter)]
    pub fn has_selection(&self) -> bool {
        self.current_selection.is_some()
    }
}
