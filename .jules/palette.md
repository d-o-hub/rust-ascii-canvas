## 2026-04-23 - [Modal and State Resilience]
**Learning:** Modals require explicit focus management (focusing the close button) and standard keyboard shortcuts (Escape) to be fully accessible. Additionally, long-running keyboard-driven states (like Space-drag panning) should be reset on window blur to prevent "stuck" UI states when users switch context.
**Action:** Always implement focus traps or focus management for modals and use the window blur event to clean up persistent keyboard-driven interactions.

## 2026-04-24 - [Contextual Cursor Feedback]
**Learning:** Providing immediate visual feedback through contextual cursors (e.g., 'move' when over a selection, 'text' for typing) significantly improves the professional feel and discoverability of tool behaviors. It reduces the user's cognitive load by confirming what action will happen before they click.
**Action:** Implement contextual cursors for all interactive regions, especially in complex canvas-based editors where the active tool behavior might change based on position.
