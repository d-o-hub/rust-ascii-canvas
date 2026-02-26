//! UI module - user interface components.

mod shortcuts;
mod theme;
mod toolbar;

pub use shortcuts::ShortcutManager;
pub use theme::Theme;
pub use toolbar::{ToolbarConfig, ToolbarItem};
