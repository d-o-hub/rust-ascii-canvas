//! Added layers move between the stack and command without losing their history.

use super::layer::{below_id_at, insert_position, LayerCommand};
use crate::core::{Grid, Layer, LayerStack};

/// Command to add a layer and make it active.
pub struct AddLayerCommand {
    index: usize,
    below_id: Option<u64>,
    id: u64,
    /// Own the complete live layer only while it is absent from the stack.
    layer: Option<Layer>,
    active_before: u64,
}

impl AddLayerCommand {
    /// Build a command that appends an empty layer with a stable identity.
    pub fn adding_next(stack: &mut LayerStack) -> Self {
        let index = stack.len();
        let active = stack.active();
        let mut layer = Layer::with_grid(
            format!("Layer {}", index + 1),
            Grid::new(active.grid().width(), active.grid().height()),
        );
        let id = stack.reserve_layer_id();
        layer.set_id(id);
        Self {
            index,
            below_id: below_id_at(stack, index),
            id,
            layer: Some(layer),
            active_before: stack.active_id(),
        }
    }

    /// Stable id of the added layer.
    pub fn layer_id(&self) -> u64 {
        self.id
    }

    /// Original insertion index.
    pub fn index(&self) -> usize {
        self.index
    }
}

impl LayerCommand for AddLayerCommand {
    fn can_apply(&self, stack: &LayerStack) -> bool {
        self.layer.is_none() || stack.len() < crate::core::document::MAX_LAYERS
    }

    fn apply(&mut self, stack: &mut LayerStack) {
        if !self.can_apply(stack) {
            return;
        }
        if let Some(layer) = self.layer.take() {
            let target = insert_position(stack, self.below_id, self.index);
            stack.insert_layer_tracking(target, layer);
        }
        stack.set_active_id(self.id);
    }

    fn undo(&mut self, stack: &mut LayerStack) {
        if self.layer.is_none() {
            self.layer = stack.remove_layer_by_id(self.id);
            stack.set_active_id(self.active_before);
        }
    }

    fn changes_content(&self) -> bool {
        true
    }

    fn description(&self) -> &str {
        "Add layer"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::commands::{Command, SetCellCommand};
    use crate::core::Cell;

    #[test]
    fn undo_redo_moves_live_layer_payload_and_history_instead_of_cloning() {
        let mut stack = LayerStack::new(10, 4);
        let mut add = AddLayerCommand::adding_next(&mut stack);
        add.apply(&mut stack);
        let mut draw = SetCellCommand::new(0, 0, Cell::new('X'));
        draw.apply(stack.active_mut().grid_mut());
        stack.push_grid(Box::new(draw));
        stack.active_mut().set_name("Sketch".into());
        for _ in 0..3 {
            add.undo(&mut stack);
            assert_eq!(stack.len(), 1);
            add.apply(&mut stack);
            assert_eq!(stack.active().name(), "Sketch");
            assert_eq!(stack.active().grid().get(0, 0).unwrap().ch, 'X');
            assert_eq!(stack.undo_count(), 1);
        }
    }
}
