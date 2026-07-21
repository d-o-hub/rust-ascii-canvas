## 2026-07-21 - [Visible Text Caret and Multi-Line Polish]
**Learning:** Visual text cursor (caret) feedback is critical in text input flows to provide immediately clear positioning to the user. Additionally, multi-line editing states (like typing list labels or multiline annotations) are much more intuitive and reliable when the local line buffer and start positions are isolated per line on Enter, preventing keystroke desyncs when backspacing.
**Action:** When designing canvas-based text inputs, implement a visible, blinking caret and reset typing parameters (start position, line-local buffers) per-line when the user creates a newline.

## 2026-07-16 - [Cross-Editor Clipboard Fidelity]
**Learning:** Proper clipboard integration with external editors (Windows Notepad, macOS TextEdit, VS Code) requires complete CRLF (`\r\n`) normalization, exact preservation of right box borders, and uniform line widths. Validating this across targets through automated headless testing combined with manual copy-paste checks ensures that high-fidelity ASCII shapes can be used seamlessly in external environments without visual degradation.
**Action:** Always normalize external exports to CRLF and maintain rigid character bounding geometries to avoid rendering glitches when drawings are pasted in environments with varying newline/monospace line-wrapping rules.

## 2026-04-23 - [Modal and State Resilience]
**Learning:** Modals require explicit focus management (focusing the close button) and standard keyboard shortcuts (Escape) to be fully accessible. Additionally, long-running keyboard-driven states (like Space-drag panning) should be reset on window blur to prevent "stuck" UI states when users switch context.
**Action:** Always implement focus traps or focus management for modals and use the window blur event to clean up persistent keyboard-driven interactions.

## 2026-04-24 - [Contextual Cursors and Tool Instructions]
**Learning:** Providing immediate visual (cursor) and textual (status bar instructions) feedback when switching tools significantly reduces the cognitive load for new users. Monospace diagrams can be complex to edit; simple "how-to" snippets in the status bar provide just-in-time guidance.
**Action:** Implement tool-specific cursors and status bar hints to guide user interactions without cluttering the main UI.

## 2026-05-28 - [Contextual Feedback and Navigation]
**Learning:** Contextual visual feedback for specialized tools (like Eraser or Selection) reduces user error and increases confidence. Providing a dedicated "Reset Zoom" shortcut (and documenting it) improves navigation in large diagrams.
**Action:** Always consider tool-specific cursor indicators and "escape hatches" like zoom reset for canvas-based interfaces.

## 2026-05-29 - [Shortcut Discoverability and ARIA metadata]
**Learning:** Essential productivity shortcuts (like "Select All" or "Reset Zoom") should be mirrored in the primary side-panel reference, not just hidden in help modals. Additionally, using `aria-keyshortcuts` on toolbar buttons provides a standard way for assistive technologies to communicate keyboard triggers that are otherwise only visible in tooltips.
**Action:** Always verify that the quick-reference shortcut list covers all high-frequency actions and ensure toolbar buttons have explicit ARIA shortcut metadata.

## 2026-07-10 - [Centering Logic and Robust Shortcuts]
**Learning:** Centering the grid content in the viewport after zoom or fit-to-view actions significantly improves the professional feel and interaction quality of the editor. Furthermore, providing character-based fallbacks for shortcuts (e.g., handling ')' for Shift+0) ensures that keyboard interactions remain accessible across different international keyboard layouts.
**Action:** Always calculate centering pan offsets when modifying canvas scale and implement character-level checks for shortcuts that involve Shift or Alt modifiers.
