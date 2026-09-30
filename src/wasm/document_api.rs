//! Document binding adapter; schema and validation belong to pure core.

use super::AsciiEditor;
use crate::core::document::Document;

impl AsciiEditor {
    pub(crate) fn serialize_document_impl(&self) -> String {
        let grid = self.committed_active_grid();
        Document::snapshot(&self.layer_stack, &grid)
            .to_json()
            .unwrap_or_else(|_| "{}".to_string())
    }

    pub(crate) fn load_document_impl(&mut self, json: &str) -> bool {
        let Some(document) = Document::parse(json) else {
            return false;
        };
        let layers = document.into_layers();
        // Failed loads leave the current interaction untouched; only a fully
        // validated replacement can cancel it and replace the live document.
        self.reset_interaction();
        self.layer_stack = layers;
        self.state.grid = self.layer_stack.active().grid().clone();
        self.clipboard.clear();
        self.pixel_buffer = vec![0; self.width() * 8 * self.height() * 20 * 4];
        self.dirty_tracker.request_full_redraw();
        true
    }
}
