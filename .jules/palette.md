## 2026-04-23 - [Modal and State Resilience]
**Learning:** Modals require explicit focus management (focusing the close button) and standard keyboard shortcuts (Escape) to be fully accessible. Additionally, long-running keyboard-driven states (like Space-drag panning) should be reset on window blur to prevent "stuck" UI states when users switch context.
**Action:** Always implement focus traps or focus management for modals and use the window blur event to clean up persistent keyboard-driven interactions.
