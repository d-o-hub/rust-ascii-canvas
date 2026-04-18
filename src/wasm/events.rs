//! Event handling utilities for WASM.
//!
//! These types are reserved for future use with advanced event handling from JavaScript.

use serde::{Deserialize, Serialize};

/// Key modifier state.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct KeyModifiers {
    /// Ctrl key is pressed
    pub ctrl: bool,
    /// Shift key is pressed
    pub shift: bool,
    /// Alt key is pressed
    pub alt: bool,
    /// Meta/Command key is pressed
    pub meta: bool,
}

impl KeyModifiers {
    /// Create a new KeyModifiers instance.
    pub fn new(ctrl: bool, shift: bool, alt: bool, meta: bool) -> Self {
        Self {
            ctrl,
            shift,
            alt,
            meta,
        }
    }
}

/// Mouse button identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseButton {
    /// Left mouse button
    Left,
    /// Middle mouse button
    Middle,
    /// Right mouse button
    Right,
}

impl MouseButton {
    /// Convert from numeric button code to MouseButton.
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
    fn test_mouse_button() {
        assert_eq!(MouseButton::from_button(0), Some(MouseButton::Left));
        assert_eq!(MouseButton::from_button(1), Some(MouseButton::Middle));
        assert_eq!(MouseButton::from_button(2), Some(MouseButton::Right));
        assert_eq!(MouseButton::from_button(3), None);
    }
}
