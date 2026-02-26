//! Font metrics for canvas rendering.

use serde::{Deserialize, Serialize};

/// Result of measuring text.
#[derive(Clone, Debug, Copy)]
pub struct MeasureResult {
    /// Width of the measured text in pixels
    pub width: f64,
    /// Height of the text line
    pub height: f64,
    /// Actual bounding box ascent
    pub ascent: f64,
    /// Actual bounding box descent
    pub descent: f64,
}

/// Font metrics for monospace rendering.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FontMetrics {
    /// Font family
    pub family: String,
    /// Font size in pixels
    pub size: f64,
    /// Character width (for monospace)
    pub char_width: f64,
    /// Line height
    pub line_height: f64,
    /// Baseline offset
    pub baseline: f64,
    /// Canvas-estimated char width (from measureText)
    pub measured_width: Option<f64>,
}

impl Default for FontMetrics {
    fn default() -> Self {
        Self {
            family: "JetBrains Mono, Fira Code, Consolas, monospace".to_string(),
            size: 14.0,
            char_width: 8.4, // Approximate for 14px JetBrains Mono
            line_height: 20.0,
            baseline: 12.0,
            measured_width: None,
        }
    }
}

impl FontMetrics {
    /// Create new font metrics.
    pub fn new(family: impl Into<String>, size: f64) -> Self {
        Self {
            family: family.into(),
            size,
            char_width: size * 0.6, // Approximate ratio
            line_height: size * 1.4,
            baseline: size * 0.85,
            measured_width: None,
        }
    }

    /// Set the measured character width from canvas.
    pub fn set_measured_width(&mut self, width: f64) {
        self.measured_width = Some(width);
        self.char_width = width;
    }

    /// Get cell size (char_width, line_height).
    pub fn cell_size(&self) -> (f64, f64) {
        (self.char_width, self.line_height)
    }

    /// Convert grid coordinates to canvas pixel coordinates.
    pub fn grid_to_canvas(&self, x: i32, y: i32) -> (f64, f64) {
        let px = x as f64 * self.char_width;
        let py = y as f64 * self.line_height;
        (px, py)
    }

    /// Convert canvas pixel coordinates to grid coordinates.
    pub fn canvas_to_grid(&self, px: f64, py: f64) -> (i32, i32) {
        let x = (px / self.char_width).floor() as i32;
        let y = (py / self.line_height).floor() as i32;
        (x, y)
    }

    /// Get the canvas width needed for a grid of given dimensions.
    pub fn canvas_size(&self, grid_width: usize, grid_height: usize) -> (f64, f64) {
        (
            grid_width as f64 * self.char_width,
            grid_height as f64 * self.line_height,
        )
    }

    /// Get CSS font string for canvas.
    pub fn css_font(&self) -> String {
        format!("{}px {}", self.size, self.family)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_metrics_default() {
        let metrics = FontMetrics::default();
        assert!(!metrics.family.is_empty());
        assert!(metrics.size > 0.0);
        assert!(metrics.char_width > 0.0);
    }

    #[test]
    fn test_grid_to_canvas() {
        let metrics = FontMetrics::default();

        let (px, py) = metrics.grid_to_canvas(5, 3);

        assert!((px - 5.0 * metrics.char_width).abs() < 0.001);
        assert!((py - 3.0 * metrics.line_height).abs() < 0.001);
    }

    #[test]
    fn test_canvas_to_grid() {
        let metrics = FontMetrics::default();

        let (x, y) = metrics.canvas_to_grid(
            5.0 * metrics.char_width + 0.5,
            3.0 * metrics.line_height + 0.5,
        );

        assert_eq!(x, 5);
        assert_eq!(y, 3);
    }
}
