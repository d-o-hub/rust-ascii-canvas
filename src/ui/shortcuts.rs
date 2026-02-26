//! Keyboard shortcuts management.

use crate::core::tools::ToolId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A keyboard shortcut definition.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Shortcut {
    /// Key (case-insensitive)
    pub key: String,
    /// Whether Ctrl is required
    pub ctrl: bool,
    /// Whether Shift is required
    pub shift: bool,
    /// Whether Alt is required
    pub alt: bool,
    /// Action identifier
    pub action: String,
    /// Human-readable description
    pub description: String,
}

impl Shortcut {
    /// Create a simple key shortcut.
    pub fn key(
        key: impl Into<String>,
        action: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into().to_uppercase(),
            ctrl: false,
            shift: false,
            alt: false,
            action: action.into(),
            description: description.into(),
        }
    }

    /// Create a Ctrl+key shortcut.
    pub fn ctrl(
        key: impl Into<String>,
        action: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into().to_uppercase(),
            ctrl: true,
            shift: false,
            alt: false,
            action: action.into(),
            description: description.into(),
        }
    }

    /// Create a Ctrl+Shift+key shortcut.
    pub fn ctrl_shift(
        key: impl Into<String>,
        action: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into().to_uppercase(),
            ctrl: true,
            shift: true,
            alt: false,
            action: action.into(),
            description: description.into(),
        }
    }

    /// Check if a key event matches this shortcut.
    pub fn matches(&self, key: &str, ctrl: bool, shift: bool, alt: bool) -> bool {
        self.key.to_uppercase() == key.to_uppercase()
            && self.ctrl == ctrl
            && self.shift == shift
            && self.alt == alt
    }
}

/// Shortcut manager for the editor.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShortcutManager {
    /// All registered shortcuts
    shortcuts: Vec<Shortcut>,
    /// Lookup by action
    by_action: HashMap<String, usize>,
}

impl Default for ShortcutManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ShortcutManager {
    /// Create a new shortcut manager with default shortcuts.
    pub fn new() -> Self {
        let mut manager = Self {
            shortcuts: Vec::new(),
            by_action: HashMap::new(),
        };

        manager.register_defaults();
        manager
    }

    /// Register default keyboard shortcuts.
    fn register_defaults(&mut self) {
        // Tool shortcuts
        self.register(Shortcut::key("R", "tool_rectangle", "Rectangle tool"));
        self.register(Shortcut::key("L", "tool_line", "Line tool"));
        self.register(Shortcut::key("A", "tool_arrow", "Arrow tool"));
        self.register(Shortcut::key("D", "tool_diamond", "Diamond tool"));
        self.register(Shortcut::key("T", "tool_text", "Text tool"));
        self.register(Shortcut::key("F", "tool_freehand", "Freehand tool"));
        self.register(Shortcut::key("V", "tool_select", "Select tool"));
        self.register(Shortcut::key("E", "tool_eraser", "Eraser tool"));

        // Edit shortcuts
        self.register(Shortcut::ctrl("Z", "undo", "Undo"));
        self.register(Shortcut::ctrl_shift("Z", "redo", "Redo"));
        self.register(Shortcut::ctrl("Y", "redo", "Redo (alternative)"));
        self.register(Shortcut::ctrl("C", "copy", "Copy ASCII"));
        self.register(Shortcut::ctrl("A", "select_all", "Select all"));

        // Navigation
        self.register(Shortcut::key("Space", "pan_mode", "Pan mode (hold)"));
        self.register(Shortcut::key("Escape", "cancel", "Cancel/Deselect"));
    }

    /// Register a new shortcut.
    pub fn register(&mut self, shortcut: Shortcut) {
        let idx = self.shortcuts.len();
        self.by_action.insert(shortcut.action.clone(), idx);
        self.shortcuts.push(shortcut);
    }

    /// Find shortcut that matches the given key event.
    pub fn find_match(&self, key: &str, ctrl: bool, shift: bool, alt: bool) -> Option<&Shortcut> {
        self.shortcuts
            .iter()
            .find(|s| s.matches(key, ctrl, shift, alt))
    }

    /// Get shortcut by action name.
    pub fn get_by_action(&self, action: &str) -> Option<&Shortcut> {
        self.by_action.get(action).map(|&idx| &self.shortcuts[idx])
    }

    /// Get all shortcuts.
    pub fn all(&self) -> &[Shortcut] {
        &self.shortcuts
    }

    /// Get shortcut display string.
    pub fn display_string(&self, action: &str) -> Option<String> {
        self.get_by_action(action).map(|s| {
            let mut parts = Vec::new();
            if s.ctrl {
                parts.push("Ctrl");
            }
            if s.shift {
                parts.push("Shift");
            }
            if s.alt {
                parts.push("Alt");
            }
            parts.push(&s.key);
            parts.join("+")
        })
    }

    /// Get action from tool ID.
    pub fn action_from_tool_id(tool_id: ToolId) -> &'static str {
        match tool_id {
            ToolId::Rectangle => "tool_rectangle",
            ToolId::Line => "tool_line",
            ToolId::Arrow => "tool_arrow",
            ToolId::Diamond => "tool_diamond",
            ToolId::Text => "tool_text",
            ToolId::Freehand => "tool_freehand",
            ToolId::Select => "tool_select",
            ToolId::Eraser => "tool_eraser",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcut_matches() {
        let shortcut = Shortcut::ctrl("Z", "undo", "Undo");

        assert!(shortcut.matches("z", true, false, false));
        assert!(shortcut.matches("Z", true, false, false));
        assert!(!shortcut.matches("z", false, false, false));
        assert!(!shortcut.matches("z", true, true, false));
    }

    #[test]
    fn test_shortcut_manager() {
        let manager = ShortcutManager::new();

        let matched = manager.find_match("R", false, false, false);
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().action, "tool_rectangle");

        let matched = manager.find_match("z", true, false, false);
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().action, "undo");
    }

    #[test]
    fn test_display_string() {
        let manager = ShortcutManager::new();

        let display = manager.display_string("undo");
        assert_eq!(display, Some("Ctrl+Z".to_string()));

        // "redo" has two shortcuts: Ctrl+Shift+Z and Ctrl+Y
        // The last registered one (Ctrl+Y) is stored in by_action
        let display = manager.display_string("redo");
        assert_eq!(display, Some("Ctrl+Y".to_string()));
    }
}
