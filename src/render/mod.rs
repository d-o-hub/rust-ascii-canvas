//! Render module - canvas rendering and metrics.
//!
//! `box_drawing` is public rather than re-exported because the WASM export layer
//! consumes it directly; the others stay curated behind single-item re-exports.

pub mod box_drawing;
mod canvas_renderer;
mod dirty_rect;
mod font_renderer;
mod metrics;

pub use canvas_renderer::CanvasRenderer;
pub use dirty_rect::{DirtyRect, DirtyTracker};
pub use font_renderer::FontAtlas;
pub use metrics::{FontMetrics, MeasureResult};
