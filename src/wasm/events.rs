//! Event handling utilities for WASM.

use serde::{Deserialize, Serialize};

/// Key modifier state.
/// Reserved for future use with advanced event handling from JavaScript.
#[allow(dead_code)]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
}

impl KeyModifiers {
    /// Reserved for future use.
    #[allow(dead_code)]
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
/// Reserved for future use with advanced event handling from JavaScript.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

impl MouseButton {
    /// Reserved for future use.
    #[allow(dead_code)]
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
/// Reserved for future use with advanced event handling from JavaScript.
#[allow(dead_code)]
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
/// Reserved for future use with advanced event handling from JavaScript.
#[allow(dead_code)]
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
/// Reserved for future use with advanced event handling from JavaScript.
#[allow(dead_code)]
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
