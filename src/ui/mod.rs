//! UI module - user interface components.

mod shortcuts;
mod toolbar;
mod theme;

pub use shortcuts::ShortcutManager;
pub use toolbar::{ToolbarItem, ToolbarConfig};
pub use theme::Theme;
