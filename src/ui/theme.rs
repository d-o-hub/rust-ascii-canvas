//! Theme configuration for the UI.

use serde::{Deserialize, Serialize};

/// Color scheme for the editor.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Theme {
    /// Name of the theme
    pub name: String,
    /// Background color
    pub background: String,
    /// Foreground (text) color
    pub foreground: String,
    /// Accent color for selections and highlights
    pub accent: String,
    /// Secondary background (toolbar, panels)
    pub secondary_background: String,
    /// Border color
    pub border: String,
    /// Hover state color
    pub hover: String,
    /// Active/selected state color
    pub active: String,
    /// Muted/disabled color
    pub muted: String,
    /// Success color
    pub success: String,
    /// Error color
    pub error: String,
    /// Warning color
    pub warning: String,
    /// Grid line color
    pub grid: String,
    /// Selection highlight color
    pub selection: String,
    /// Cursor color
    pub cursor: String,
    /// Font family
    pub font_family: String,
    /// Font size (CSS string)
    pub font_size: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self::figma_dark()
    }
}

impl Theme {
    /// Create the default Figma-inspired dark theme.
    pub fn figma_dark() -> Self {
        Self {
            name: "Figma Dark".to_string(),
            background: "#1e1e1e".to_string(),
            foreground: "#d4d4d4".to_string(),
            accent: "#0d99ff".to_string(),
            secondary_background: "#2c2c2c".to_string(),
            border: "#3c3c3c".to_string(),
            hover: "#3c3c3c".to_string(),
            active: "#0d99ff33".to_string(),
            muted: "#6b6b6b".to_string(),
            success: "#14ae5c".to_string(),
            error: "#f24822".to_string(),
            warning: "#ffcd29".to_string(),
            grid: "#333333".to_string(),
            selection: "#264f78".to_string(),
            cursor: "#ffffff".to_string(),
            font_family: "'JetBrains Mono', 'Fira Code', 'Consolas', monospace".to_string(),
            font_size: "14px".to_string(),
        }
    }

    /// Create a light theme variant.
    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
            background: "#ffffff".to_string(),
            foreground: "#1e1e1e".to_string(),
            accent: "#0d99ff".to_string(),
            secondary_background: "#f5f5f5".to_string(),
            border: "#e0e0e0".to_string(),
            hover: "#e8e8e8".to_string(),
            active: "#0d99ff22".to_string(),
            muted: "#999999".to_string(),
            success: "#14ae5c".to_string(),
            error: "#f24822".to_string(),
            warning: "#ffcd29".to_string(),
            grid: "#e0e0e0".to_string(),
            selection: "#b4d7ff".to_string(),
            cursor: "#000000".to_string(),
            font_family: "'JetBrains Mono', 'Fira Code', 'Consolas', monospace".to_string(),
            font_size: "14px".to_string(),
        }
    }

    /// Create a high contrast theme.
    pub fn high_contrast() -> Self {
        Self {
            name: "High Contrast".to_string(),
            background: "#000000".to_string(),
            foreground: "#ffffff".to_string(),
            accent: "#00ffff".to_string(),
            secondary_background: "#1a1a1a".to_string(),
            border: "#ffffff".to_string(),
            hover: "#333333".to_string(),
            active: "#00ffff44".to_string(),
            muted: "#888888".to_string(),
            success: "#00ff00".to_string(),
            error: "#ff0000".to_string(),
            warning: "#ffff00".to_string(),
            grid: "#333333".to_string(),
            selection: "#00ffff44".to_string(),
            cursor: "#ffffff".to_string(),
            font_family: "'JetBrains Mono', 'Fira Code', 'Consolas', monospace".to_string(),
            font_size: "14px".to_string(),
        }
    }

    /// Convert theme to CSS variables.
    pub fn to_css_variables(&self) -> String {
        format!(
            r#":root {{
    --bg: {background};
    --fg: {foreground};
    --accent: {accent};
    --bg-secondary: {secondary_background};
    --border: {border};
    --hover: {hover};
    --active: {active};
    --muted: {muted};
    --success: {success};
    --error: {error};
    --warning: {warning};
    --grid: {grid};
    --selection: {selection};
    --cursor: {cursor};
    --font-family: {font_family};
    --font-size: {font_size};
}}"#,
            background = self.background,
            foreground = self.foreground,
            accent = self.accent,
            secondary_background = self.secondary_background,
            border = self.border,
            hover = self.hover,
            active = self.active,
            muted = self.muted,
            success = self.success,
            error = self.error,
            warning = self.warning,
            grid = self.grid,
            selection = self.selection,
            cursor = self.cursor,
            font_family = self.font_family,
            font_size = self.font_size,
        )
    }

    /// Get all available themes.
    pub fn all() -> Vec<Self> {
        vec![Self::figma_dark(), Self::light(), Self::high_contrast()]
    }

    /// Find a theme by name.
    pub fn find(name: &str) -> Option<Self> {
        Self::all()
            .into_iter()
            .find(|t| t.name.eq_ignore_ascii_case(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme() {
        let theme = Theme::default();
        assert_eq!(theme.name, "Figma Dark");
        assert!(theme.background.starts_with('#'));
    }

    #[test]
    fn test_css_variables() {
        let theme = Theme::figma_dark();
        let css = theme.to_css_variables();

        assert!(css.contains("--bg:"));
        assert!(css.contains("--fg:"));
        assert!(css.contains("--accent:"));
    }

    #[test]
    fn test_find_theme() {
        let theme = Theme::find("Light");
        assert!(theme.is_some());
        assert_eq!(theme.unwrap().name, "Light");

        let theme = Theme::find("nonexistent");
        assert!(theme.is_none());
    }
}
