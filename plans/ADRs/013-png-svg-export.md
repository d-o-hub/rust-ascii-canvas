# ADR-013: PNG/SVG Export

## Status
Proposed

## Context
Currently, users can only export ASCII text via clipboard. They cannot save diagrams as images for embedding in documents, presentations, or sharing.

### Current Implementation
- `export_ascii()` returns plain text representation
- No image export capability
- Users must screenshot the canvas manually

### User Impact
Cannot easily share diagrams in:
- Documentation (Markdown, Confluence, Notion)
- Presentations (PowerPoint, Google Slides)
- Design tools (Figma, Sketch)

## Decision
Implement image export in two formats:

### 1. PNG Export
Raster image suitable for quick sharing, presentations, documents.

**Implementation**:
- Use `canvas.toDataURL('image/png')` 
- Create off-screen canvas at higher resolution for export
- Options: 2x resolution, include/exclude grid, transparent background

### 2. SVG Export
Vector image suitable for scaling without quality loss, design tool import.

**Implementation**:
- Generate SVG from grid content
- Each character as `<text>` element
- Editable in design tools, smaller file size for text

### 3. UI Integration
Add export menu to toolbar:

| Option | Shortcut | Description |
|--------|----------|-------------|
| Copy ASCII | Ctrl+C | Current behavior |
| Export PNG | Ctrl+Shift+E | Download PNG file |
| Export SVG | - | Download SVG file |

### 4. File Download
Use browser download API with blob URLs.

## Consequences

### Positive
- Easy sharing to any platform
- Professional output for presentations
- Vector format for design integration

### Negative
- Additional code complexity
- SVG export may be large for dense diagrams
- Need to handle font embedding for SVG

## Implementation Plan
1. Create `exportPng()` function using canvas API
2. Create `exportSvg()` function generating SVG string
3. Add download trigger with `URL.createObjectURL()`
4. Add export buttons to UI
5. Add E2E tests for export functionality
