//! Render module - canvas rendering and metrics.

mod canvas_renderer;
mod metrics;
mod dirty_rect;

pub use canvas_renderer::CanvasRenderer;
pub use metrics::{FontMetrics, MeasureResult};
pub use dirty_rect::{DirtyRect, DirtyTracker};
