//! Event handling utilities for WASM.

use serde::{Deserialize, Serialize};

/// Result of processing a user event.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EventResult {
    /// Whether the canvas needs redraw
    pub needs_redraw: bool,
    /// Current tool name
    pub tool: String,
    /// Whether undo is available
    pub can_undo: bool,
    /// Whether redo is available
    pub can_redo: bool,
    /// Whether to copy ASCII to clipboard
    pub should_copy: bool,
    /// ASCII content for clipboard
    pub ascii: Option<String>,
    /// Cursor position in grid coordinates
    pub cursor: Option<(i32, i32)>,
}

impl EventResult {
    /// Create a new event result.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set needs redraw flag.
    pub fn with_redraw(mut self, redraw: bool) -> Self {
        self.needs_redraw = redraw;
        self
    }

    /// Set tool name.
    pub fn with_tool(mut self, tool: impl Into<String>) -> Self {
        self.tool = tool.into();
        self
    }

    /// Set clipboard copy data.
    pub fn with_copy(mut self, ascii: String) -> Self {
        self.should_copy = true;
        self.ascii = Some(ascii);
        self
    }

    /// Set cursor position.
    pub fn with_cursor(mut self, x: i32, y: i32) -> Self {
        self.cursor = Some((x, y));
        self
    }
}

/// Key modifier state.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
}

impl KeyModifiers {
    pub fn new(ctrl: bool, shift: bool, alt: bool, meta: bool) -> Self {
        Self { ctrl, shift, alt, meta }
    }
}

/// Mouse button identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

impl MouseButton {
    pub fn from_button(button: i16) -> Option<Self> {
        match button {
            0 => Some(Self::Left),
            1 => Some(Self::Middle),
            2 => Some(Self::Right),
            _ => None,
        }
    }
}

/// Pointer event data.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PointerEventData {
    /// Screen X coordinate
    pub screen_x: f64,
    /// Screen Y coordinate
    pub screen_y: f64,
    /// Grid X coordinate
    pub grid_x: i32,
    /// Grid Y coordinate
    pub grid_y: i32,
    /// Mouse button
    pub button: MouseButton,
    /// Key modifiers
    pub modifiers: KeyModifiers,
    /// Whether pointer is down
    pub is_down: bool,
}

/// Keyboard event data.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyboardEventData {
    /// Key value
    pub key: String,
    /// Key code
    pub code: String,
    /// Key modifiers
    pub modifiers: KeyModifiers,
    /// Whether this is a key repeat
    pub repeat: bool,
}

/// Wheel event data.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WheelEventData {
    /// Screen X coordinate
    pub screen_x: f64,
    /// Screen Y coordinate
    pub screen_y: f64,
    /// Delta X (horizontal scroll)
    pub delta_x: f64,
    /// Delta Y (vertical scroll)
    pub delta_y: f64,
    /// Delta mode (pixel, line, page)
    pub delta_mode: u32,
    /// Key modifiers
    pub modifiers: KeyModifiers,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_result() {
        let result = EventResult::new()
            .with_redraw(true)
            .with_tool("Rectangle")
            .with_copy("test".to_string());

        assert!(result.needs_redraw);
        assert_eq!(result.tool, "Rectangle");
        assert!(result.should_copy);
        assert_eq!(result.ascii, Some("test".to_string()));
    }

    #[test]
    fn test_mouse_button() {
        assert_eq!(MouseButton::from_button(0), Some(MouseButton::Left));
        assert_eq!(MouseButton::from_button(1), Some(MouseButton::Middle));
        assert_eq!(MouseButton::from_button(2), Some(MouseButton::Right));
        assert_eq!(MouseButton::from_button(3), None);
    }
}
