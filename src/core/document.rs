//! Shared `.asc` v1 schema, size policy, and lossless document snapshots.
//!
//! Compatibility intentionally matches the original reader: positive future
//! versions, unknown fields, first-scalar cell strings, clipped coordinates,
//! last duplicate cell wins, and a clamped active index remain accepted.

use serde::{Deserialize, Serialize};

use super::{Grid, Layer, LayerStack};

/// Maximum width shared by document loading and editor creation/resizing.
pub const MAX_CANVAS_WIDTH: usize = 400;
/// Maximum height shared by document loading and editor creation/resizing.
pub const MAX_CANVAS_HEIGHT: usize = 200;
/// Maximum number of layers in a document.
pub const MAX_LAYERS: usize = 32;

/// Whether dimensions can be serialized and restored by the editor.
pub fn valid_dimensions(width: usize, height: usize) -> bool {
    (1..=MAX_CANVAS_WIDTH).contains(&width) && (1..=MAX_CANVAS_HEIGHT).contains(&height)
}

#[derive(Serialize, Deserialize)]
struct DocCell {
    x: i32,
    y: i32,
    ch: String,
}

#[derive(Serialize, Deserialize)]
struct DocLayer {
    name: String,
    #[serde(default = "default_true")]
    visible: bool,
    #[serde(default)]
    locked: bool,
    cells: Vec<DocCell>,
}

fn default_true() -> bool {
    true
}

#[derive(Serialize, Deserialize)]
struct CanvasSize {
    width: usize,
    height: usize,
}

/// Version-one interchange DTO. Construction is through validated parsing or
/// a snapshot of an editor-owned stack; runtime history is never serialized.
#[derive(Serialize, Deserialize)]
pub struct Document {
    format: String,
    version: u32,
    canvas: CanvasSize,
    #[serde(default)]
    active_layer: usize,
    layers: Vec<DocLayer>,
}

impl Document {
    /// Capture metadata and committed cells, using the supplied live active grid.
    pub fn snapshot(stack: &LayerStack, active_grid: &Grid) -> Self {
        let active = stack.active_index();
        let layers = stack
            .layers()
            .iter()
            .enumerate()
            .map(|(index, layer)| {
                let grid = if index == active {
                    active_grid
                } else {
                    layer.grid()
                };
                let cells = grid
                    .iter_with_coords()
                    .filter(|(_, _, cell)| cell.is_visible())
                    .map(|(x, y, cell)| DocCell {
                        x,
                        y,
                        ch: cell.ch.to_string(),
                    })
                    .collect();
                DocLayer {
                    name: layer.name().to_string(),
                    visible: layer.is_visible(),
                    locked: layer.is_locked(),
                    cells,
                }
            })
            .collect();
        Self {
            format: "ascii-canvas".into(),
            version: 1,
            canvas: CanvasSize {
                width: active_grid.width(),
                height: active_grid.height(),
            },
            active_layer: active,
            layers,
        }
    }

    /// Parse using the existing v1 compatibility rules and shared resource limits.
    pub fn parse(json: &str) -> Option<Self> {
        let doc: Self = serde_json::from_str(json).ok()?;
        (doc.format == "ascii-canvas"
            && doc.version > 0
            && valid_dimensions(doc.canvas.width, doc.canvas.height)
            && !doc.layers.is_empty()
            && doc.layers.len() <= MAX_LAYERS)
            .then_some(doc)
    }

    /// Serialize the unchanged v1 JSON shape.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Restore layers with fresh per-layer histories and preserved v1 metadata.
    pub fn into_layers(self) -> LayerStack {
        let layers = self
            .layers
            .into_iter()
            .map(|layer| {
                let mut grid = Grid::new(self.canvas.width, self.canvas.height);
                for cell in layer.cells {
                    if let Some(ch) = cell.ch.chars().next() {
                        grid.set_char(cell.x, cell.y, ch);
                    }
                }
                let mut restored = Layer::with_grid(layer.name, grid);
                restored.set_visible(layer.visible);
                restored.set_locked(layer.locked);
                restored
            })
            .collect();
        LayerStack::from_layers(layers, self.active_layer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_existing_import_compatibility_and_defaults() {
        let json = r#"{"format":"ascii-canvas","version":9,"unknown":true,
            "canvas":{"width":2,"height":1},"active_layer":999,
            "layers":[{"name":"Legacy","cells":[
                {"x":0,"y":0,"ch":"AB"}, {"x":0,"y":0,"ch":"éZ"},
                {"x":1,"y":0,"ch":""}, {"x":99,"y":0,"ch":"X"}]}]}"#;
        let stack = Document::parse(json).unwrap().into_layers();
        assert_eq!(stack.active_index(), 0);
        assert!(stack.active().is_visible());
        assert!(!stack.active().is_locked());
        assert_eq!(stack.active().grid().get(0, 0).unwrap().ch, 'é');
        assert_eq!(stack.active().grid().get(1, 0).unwrap().ch, ' ');
    }

    #[test]
    fn snapshot_roundtrips_maximum_dimensions_and_metadata() {
        let mut stack = LayerStack::new(MAX_CANVAS_WIDTH, MAX_CANVAS_HEIGHT);
        stack.active_mut().set_visible(false);
        stack.active_mut().set_locked(true);
        let mut grid = stack.active().grid().clone();
        grid.set_char(399, 199, '界');
        let json = Document::snapshot(&stack, &grid).to_json().unwrap();
        let restored = Document::parse(&json).unwrap().into_layers();
        assert_eq!(restored.active().grid().get(399, 199).unwrap().ch, '界');
        assert!(!restored.active().is_visible());
        assert!(restored.active().is_locked());
        assert!(!restored.can_undo());
    }

    #[test]
    fn rejects_only_existing_schema_and_resource_errors() {
        for (width, height, count, version) in [
            (0, 1, 1, 1),
            (401, 1, 1, 1),
            (1, 201, 1, 1),
            (1, 1, 33, 1),
            (1, 1, 0, 1),
            (1, 1, 1, 0),
        ] {
            let layers = (0..count)
                .map(|_| r#"{"name":"L","cells":[]}"#)
                .collect::<Vec<_>>()
                .join(",");
            let json = format!(
                r#"{{"format":"ascii-canvas","version":{version},"canvas":{{"width":{width},"height":{height}}},"layers":[{layers}]}}"#
            );
            assert!(Document::parse(&json).is_none());
        }
    }
}
