# ADR-027: File Persistence (Save/Load)

## Status
Proposed

## Date
2026-03-03

## Context

The ASCII Canvas Editor has no persistence mechanism. All work is lost when the browser tab is closed or refreshed. Users must:
1. Copy the ASCII output to clipboard and paste it somewhere else
2. Redo their work from scratch if the browser crashes
3. Cannot share diagram source files with collaborators

This is a critical usability gap. Every production diagram tool supports save/load.

## Decision

### File Format: `.asc` (JSON-based)

```json
{
  "format": "ascii-canvas",
  "version": 1,
  "metadata": {
    "title": "My Diagram",
    "created": "2026-03-03T12:00:00Z",
    "modified": "2026-03-03T14:30:00Z",
    "tool_version": "0.1.0"
  },
  "canvas": {
    "width": 80,
    "height": 40
  },
  "layers": [
    {
      "id": 1,
      "name": "Layer 1",
      "visible": true,
      "locked": false,
      "cells": [
        {"x": 0, "y": 0, "ch": "┌", "style": 0},
        {"x": 1, "y": 0, "ch": "─", "style": 0}
      ]
    }
  ],
  "active_layer_id": 1,
  "view": {
    "zoom": 1.0,
    "pan_x": 0,
    "pan_y": 0
  }
}
```

**Key design choices:**
- Only non-empty cells are serialized (sparse representation)
- View state (zoom/pan) is preserved
- Layer structure is preserved
- Format version enables future migration

### Storage Mechanisms

#### 1. localStorage Auto-Save
- Debounced (500ms after last change)
- Key: `ascii-canvas:autosave`
- Loaded on startup if present
- "Recover unsaved work?" prompt if autosave is newer than 5 minutes

#### 2. File Download (Save As)
- `Ctrl+S` / Cmd+S triggers save
- If no filename, prompt for name
- Creates `.asc` file via `Blob` + `URL.createObjectURL` + download
- Also supports "Export as ASCII" (plain text, current behavior)

#### 3. File Open
- `Ctrl+O` / Cmd+O triggers open
- File picker filtered to `.asc` files
- `FileReader` API reads JSON
- Validates format version, migrates if needed
- Replaces current editor state

### Rust Implementation

```rust
// src/core/persistence.rs
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ProjectFile {
    pub format: String,
    pub version: u32,
    pub metadata: ProjectMetadata,
    pub canvas: CanvasConfig,
    pub layers: Vec<LayerData>,
    pub active_layer_id: u32,
    pub view: ViewState,
}

impl ProjectFile {
    pub fn from_editor(editor: &AsciiEditor) -> Self;
    pub fn apply_to_editor(&self, editor: &mut AsciiEditor) -> Result<(), PersistenceError>;
}

pub fn serialize_project(editor: &AsciiEditor) -> Result<String, serde_json::Error>;
pub fn deserialize_project(json: &str) -> Result<ProjectFile, PersistenceError>;
```

### WASM Bindings

```rust
pub fn save_project(&self) -> JsValue;       // Returns JSON string
pub fn load_project(&mut self, json: &str) -> bool;  // Returns success
pub fn get_autosave_json(&self) -> JsValue;   // Compact format for localStorage
```

### TypeScript Integration

```typescript
// web/src/persistence.ts
export class PersistenceManager {
  private autosaveTimer: number | null = null;

  startAutosave(editor: AsciiEditor, intervalMs: number = 500): void;
  stopAutosave(): void;
  saveToFile(editor: AsciiEditor, filename?: string): void;
  loadFromFile(editor: AsciiEditor): Promise<boolean>;
  checkAutosaveRecovery(): ProjectFile | null;
}
```

## Consequences

### Positive
- Users never lose work (autosave)
- Diagrams can be shared as `.asc` files
- Version field enables format evolution
- Sparse cell storage keeps files small (typical diagram: 2-10KB)
- View state preserved (zoom/pan remembered)

### Negative
- localStorage has ~5MB limit — large diagrams could exceed
- JSON format is human-readable but not the most compact
- Format versioning adds migration complexity over time
- File picker UX varies across browsers

### Alternatives Considered

1. **Binary format (MessagePack)**: More compact but not human-readable, harder to debug
2. **IndexedDB instead of localStorage**: More storage but more complex API — defer to v2
3. **Cloud storage**: Requires backend infrastructure — defer to P3 roadmap

### Risks
- Autosave on every change could cause performance issues on large grids — mitigated by debouncing and sparse representation
- File format changes require migration code — keep format simple initially
