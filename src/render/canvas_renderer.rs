//! Canvas renderer - draws the ASCII grid to an HTML canvas.

use crate::core::grid::Grid;
use crate::render::{DirtyRect, FontMetrics};
use serde::{Deserialize, Serialize};

/// Canvas renderer for the ASCII grid.
pub struct CanvasRenderer {
    /// Font metrics
    metrics: FontMetrics,
    /// Canvas width
    canvas_width: f64,
    /// Canvas height
    canvas_height: f64,
    /// Background color
    bg_color: String,
    /// Foreground color
    fg_color: String,
    /// Selection highlight color
    selection_color: String,
    /// Grid line color
    grid_color: String,
    /// Whether to show grid lines
    show_grid: bool,
    /// Zoom level
    zoom: f64,
    /// Pan offset X
    pan_x: f64,
    /// Pan offset Y
    pan_y: f64,
}

impl Default for CanvasRenderer {
    fn default() -> Self {
        Self {
            metrics: FontMetrics::default(),
            canvas_width: 800.0,
            canvas_height: 600.0,
            bg_color: "#1e1e1e".to_string(),
            fg_color: "#d4d4d4".to_string(),
            selection_color: "#264f78".to_string(),
            grid_color: "#333333".to_string(),
            show_grid: false,
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
        }
    }
}

impl CanvasRenderer {
    /// Create a new canvas renderer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set font metrics.
    pub fn set_metrics(&mut self, metrics: FontMetrics) {
        self.metrics = metrics;
    }

    /// Get font metrics.
    pub fn metrics(&self) -> &FontMetrics {
        &self.metrics
    }

    /// Set canvas size.
    pub fn set_canvas_size(&mut self, width: f64, height: f64) {
        self.canvas_width = width;
        self.canvas_height = height;
    }

    /// Set background color.
    pub fn set_bg_color(&mut self, color: impl Into<String>) {
        self.bg_color = color.into();
    }

    /// Set foreground color.
    pub fn set_fg_color(&mut self, color: impl Into<String>) {
        self.fg_color = color.into();
    }

    /// Set selection color.
    pub fn set_selection_color(&mut self, color: impl Into<String>) {
        self.selection_color = color.into();
    }

    /// Set grid visibility.
    pub fn set_show_grid(&mut self, show: bool) {
        self.show_grid = show;
    }

    /// Set zoom level.
    pub fn set_zoom(&mut self, zoom: f64) {
        self.zoom = zoom.clamp(0.25, 4.0);
    }

    /// Get zoom level.
    pub fn zoom(&self) -> f64 {
        self.zoom
    }

    /// Set pan offset.
    pub fn set_pan(&mut self, x: f64, y: f64) {
        self.pan_x = x;
        self.pan_y = y;
    }

    /// Get pan offset.
    pub fn pan(&self) -> (f64, f64) {
        (self.pan_x, self.pan_y)
    }

    /// Convert screen coordinates to grid coordinates.
    pub fn screen_to_grid(&self, screen_x: f64, screen_y: f64) -> (i32, i32) {
        let x = ((screen_x - self.pan_x) / self.zoom) / self.metrics.char_width;
        let y = ((screen_y - self.pan_y) / self.zoom) / self.metrics.line_height;
        (x.floor() as i32, y.floor() as i32)
    }

    /// Convert grid coordinates to screen coordinates.
    pub fn grid_to_screen(&self, grid_x: i32, grid_y: i32) -> (f64, f64) {
        let x = (grid_x as f64 * self.metrics.char_width * self.zoom) + self.pan_x;
        let y = (grid_y as f64 * self.metrics.line_height * self.zoom) + self.pan_y;
        (x, y)
    }

    /// Build JavaScript render commands for the grid.
    pub fn build_render_commands(&self, grid: &Grid, dirty: &DirtyRect) -> Vec<RenderCommand> {
        let mut commands = Vec::new();

        if dirty.is_empty() {
            return commands;
        }

        // Draw each visible cell in the dirty region
        for (x, y) in dirty.iter() {
            if let Some(cell) = grid.get(x, y) {
                if cell.is_visible() {
                    let (sx, sy) = self.grid_to_screen(x, y);
                    commands.push(RenderCommand::DrawChar {
                        x: sx,
                        y: sy + self.metrics.baseline * self.zoom,
                        char: cell.ch,
                        scale: self.zoom,
                    });
                }
            }
        }

        commands
    }

    /// Build full render commands (for initial render).
    pub fn build_full_render(&self, grid: &Grid) -> Vec<RenderCommand> {
        let mut commands = vec![
            RenderCommand::Clear { color: self.bg_color.clone() },
        ];

        // Draw grid lines if enabled
        if self.show_grid {
            commands.push(RenderCommand::DrawGrid {
                width: grid.width() as f64 * self.metrics.char_width * self.zoom,
                height: grid.height() as f64 * self.metrics.line_height * self.zoom,
                cell_width: self.metrics.char_width * self.zoom,
                cell_height: self.metrics.line_height * self.zoom,
                color: self.grid_color.clone(),
                pan_x: self.pan_x,
                pan_y: self.pan_y,
            });
        }

        // Set font
        commands.push(RenderCommand::SetFont {
            font: self.metrics.css_font(),
            scale: self.zoom,
        });

        // Draw all visible cells
        for (x, y, cell) in grid.iter_with_coords() {
            if cell.is_visible() {
                let (sx, sy) = self.grid_to_screen(x, y);
                commands.push(RenderCommand::DrawChar {
                    x: sx,
                    y: sy + self.metrics.baseline * self.zoom,
                    char: cell.ch,
                    scale: self.zoom,
                });
            }
        }

        commands
    }

    /// Build selection highlight render commands.
    pub fn build_selection_render(
        &self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
    ) -> Vec<RenderCommand> {
        let min_x = x1.min(x2);
        let min_y = y1.min(y2);
        let max_x = x1.max(x2);
        let max_y = y1.max(y2);

        let (sx1, sy1) = self.grid_to_screen(min_x, min_y);
        let (sx2, sy2) = self.grid_to_screen(max_x + 1, max_y + 1);

        vec![RenderCommand::DrawRect {
            x: sx1,
            y: sy1,
            width: sx2 - sx1,
            height: sy2 - sy1,
            color: self.selection_color.clone(),
        }]
    }
}

/// Render command for the canvas.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RenderCommand {
    /// Clear the canvas with a color
    Clear { color: String },
    /// Set the current font
    SetFont { font: String, scale: f64 },
    /// Draw a character at position
    DrawChar { x: f64, y: f64, char: char, scale: f64 },
    /// Draw a rectangle (for selection)
    DrawRect { x: f64, y: f64, width: f64, height: f64, color: String },
    /// Draw grid lines
    DrawGrid {
        width: f64,
        height: f64,
        cell_width: f64,
        cell_height: f64,
        color: String,
        pan_x: f64,
        pan_y: f64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_default() {
        let renderer = CanvasRenderer::new();
        assert_eq!(renderer.zoom(), 1.0);
    }

    #[test]
    fn test_zoom_clamp() {
        let mut renderer = CanvasRenderer::new();
        renderer.set_zoom(10.0);
        assert_eq!(renderer.zoom(), 4.0);

        renderer.set_zoom(0.1);
        assert_eq!(renderer.zoom(), 0.25);
    }

    #[test]
    fn test_coordinate_transform() {
        let renderer = CanvasRenderer::new();

        // Grid to screen and back
        let (sx, sy) = renderer.grid_to_screen(5, 3);
        let (gx, gy) = renderer.screen_to_grid(sx, sy);

        assert_eq!(gx, 5);
        assert_eq!(gy, 3);
    }

    #[test]
    fn test_build_render_commands() {
        let renderer = CanvasRenderer::new();
        let grid = Grid::new(10, 10);
        let dirty = DirtyRect::empty();

        let commands = renderer.build_render_commands(&grid, &dirty);
        assert!(commands.is_empty());
    }
}
