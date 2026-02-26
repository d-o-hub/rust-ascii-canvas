//! Render module - canvas rendering and metrics.

mod canvas_renderer;
mod dirty_rect;
mod metrics;

pub use canvas_renderer::CanvasRenderer;
pub use dirty_rect::{DirtyRect, DirtyTracker};
pub use metrics::{FontMetrics, MeasureResult};
