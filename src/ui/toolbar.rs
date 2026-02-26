//! Toolbar configuration.

use crate::core::tools::ToolId;
use serde::{Deserialize, Serialize};

/// A toolbar item definition.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolbarItem {
    /// Item identifier
    pub id: String,
    /// Display label
    pub label: String,
    /// Icon (emoji or icon name)
    pub icon: String,
    /// Keyboard shortcut
    pub shortcut: String,
    /// Whether this item is a tool
    pub is_tool: bool,
    /// Tool ID if applicable
    pub tool_id: Option<String>,
    /// Whether the item is a separator
    pub is_separator: bool,
}

impl ToolbarItem {
    /// Create a tool item.
    pub fn tool(tool_id: ToolId) -> Self {
        Self {
            id: tool_id.name().to_lowercase(),
            label: tool_id.name().to_string(),
            icon: Self::tool_icon(tool_id),
            shortcut: tool_id.shortcut().to_string(),
            is_tool: true,
            tool_id: Some(format!("{:?}", tool_id)),
            is_separator: false,
        }
    }

    /// Create an action item (non-tool).
    pub fn action(id: impl Into<String>, label: impl Into<String>, icon: impl Into<String>, shortcut: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: icon.into(),
            shortcut: shortcut.into(),
            is_tool: false,
            tool_id: None,
            is_separator: false,
        }
    }

    /// Create a separator.
    pub fn separator() -> Self {
        Self {
            id: String::new(),
            label: String::new(),
            icon: String::new(),
            shortcut: String::new(),
            is_tool: false,
            tool_id: None,
            is_separator: true,
        }
    }

    fn tool_icon(tool_id: ToolId) -> String {
        match tool_id {
            ToolId::Rectangle => "▢".to_string(),
            ToolId::Line => "─".to_string(),
            ToolId::Arrow => "→".to_string(),
            ToolId::Diamond => "◇".to_string(),
            ToolId::Text => "T".to_string(),
            ToolId::Freehand => "✎".to_string(),
            ToolId::Select => "⬚".to_string(),
            ToolId::Eraser => "⌫".to_string(),
        }
    }
}

/// Toolbar configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolbarConfig {
    /// All toolbar items
    pub items: Vec<ToolbarItem>,
}

impl Default for ToolbarConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolbarConfig {
    /// Create a default toolbar configuration.
    pub fn new() -> Self {
        let items = vec![
            ToolbarItem::tool(ToolId::Select),
            ToolbarItem::tool(ToolId::Rectangle),
            ToolbarItem::tool(ToolId::Line),
            ToolbarItem::tool(ToolId::Arrow),
            ToolbarItem::tool(ToolId::Diamond),
            ToolbarItem::separator(),
            ToolbarItem::tool(ToolId::Text),
            ToolbarItem::tool(ToolId::Freehand),
            ToolbarItem::tool(ToolId::Eraser),
            ToolbarItem::separator(),
            ToolbarItem::action("undo", "Undo", "↶", "Ctrl+Z"),
            ToolbarItem::action("redo", "Redo", "↷", "Ctrl+Shift+Z"),
            ToolbarItem::separator(),
            ToolbarItem::action("copy", "Copy", "⎘", "Ctrl+C"),
            ToolbarItem::action("clear", "Clear", "⌫", ""),
        ];

        Self { items }
    }

    /// Get items as JSON for JavaScript.
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.items).unwrap_or_else(|_| "[]".to_string())
    }

    /// Find item by ID.
    pub fn find(&self, id: &str) -> Option<&ToolbarItem> {
        self.items.iter().find(|item| item.id == id)
    }

    /// Get all tool items.
    pub fn tools(&self) -> impl Iterator<Item = &ToolbarItem> {
        self.items.iter().filter(|item| item.is_tool)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toolbar_item_tool() {
        let item = ToolbarItem::tool(ToolId::Rectangle);
        assert_eq!(item.id, "rectangle");
        assert_eq!(item.label, "Rectangle");
        assert!(item.is_tool);
        assert!(item.tool_id.is_some());
    }

    #[test]
    fn test_toolbar_separator() {
        let item = ToolbarItem::separator();
        assert!(item.is_separator);
        assert!(!item.is_tool);
    }

    #[test]
    fn test_toolbar_config() {
        let config = ToolbarConfig::new();
        assert!(!config.items.is_empty());

        let tools: Vec<_> = config.tools().collect();
        assert!(!tools.is_empty());
    }

    #[test]
    fn test_toolbar_json() {
        let config = ToolbarConfig::new();
        let json = config.to_json();
        assert!(json.starts_with('['));
        assert!(json.ends_with(']'));
    }
}
