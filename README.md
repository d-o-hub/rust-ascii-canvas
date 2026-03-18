# ASCII Canvas Editor

A **production-grade ASCII diagram editor** built with Rust and WebAssembly. Features a polished dark UI inspired by Figma, running at 60 FPS with zero browser text-selection artifacts.

## Features

- 🎨 **8 Drawing Tools**: Rectangle, Line, Arrow, Diamond, Text, Freehand, Select, Eraser
- ✋ **Select Tool**: Click and drag to select regions, move selected objects
- 🖼️ **6 Border Styles**: Single, Double, Heavy, Rounded, ASCII, Dotted
- ↩️ **Full Undo/Redo**: Command pattern with configurable history depth
- 🔍 **Zoom & Pan**: Mouse wheel zoom, Space+drag panning
- 📋 **One-Click Copy**: Export trimmed ASCII to clipboard
- ⌨️ **Keyboard-First**: Full keyboard shortcut support
- 🌙 **Dark Theme**: Professional Figma-inspired dark UI
- ⚡ **60 FPS Rendering**: Dirty-rect optimization, zero per-frame allocations
- 📦 **Offline-First**: No external dependencies, works offline

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (1.75+)
- [mise](https://mise.jdx.dev/) (modern toolchain manager)
- [Node.js](https://nodejs.org/) (22+)

The project uses `mise` to automatically manage and pin the correct versions of Rust, Node, `wasm-bindgen-cli`, and `wasm-opt`.

### Build

```bash
# Clone the repository
git clone https://github.com/d-o-hub/rust-ascii-canvas.git
cd rust-ascii-canvas

# Build WASM module
npm run build:wasm

# Install dependencies and start dev server
cd web
npm install
npm run dev
```

The editor will be available at `http://localhost:3000`.

## Project Structure

```
ascii-canvas/
├── Cargo.toml              # Rust dependencies
├── src/
│   ├── lib.rs              # Library entry point
│   ├── core/               # Pure Rust logic (no WASM)
│   │   ├── cell.rs         # Cell representation
│   │   ├── grid.rs         # 2D grid model
│   │   ├── tools/          # Drawing tools
│   │   ├── commands/       # Command pattern
│   │   ├── history.rs      # Undo/redo system
│   │   ├── selection.rs    # Selection model
│   │   └── ascii_export.rs # Export utilities
│   ├── render/             # Canvas rendering
│   │   ├── canvas_renderer.rs
│   │   ├── metrics.rs      # Font metrics
│   │   └── dirty_rect.rs   # Dirty region tracking
│   ├── wasm/               # WASM bindings
│   │   ├── bindings.rs     # Main editor class
│   │   ├── events.rs       # Event handling
│   │   └── clipboard.rs    # Clipboard utilities
│   ├── ui/                 # UI components
│   │   ├── shortcuts.rs    # Keyboard shortcuts
│   │   ├── toolbar.rs      # Toolbar config
│   │   └── theme.rs        # Theme definitions
│   └── utils/              # Utilities
├── web/
│   ├── index.html          # HTML template
│   ├── style.css           # Dark theme styles
│   └── main.ts             # TypeScript entry
├── tests/
│   ├── core/               # Core unit tests
│   └── wasm/               # WASM browser tests
├── wasm-pack.toml          # wasm-pack config
└── vite.config.ts          # Vite build config
```

## Select Tool

The Select tool (V) allows you to:
- **Click and drag** to create a selection rectangle
- **Click inside selection** and drag to **move** the selected content
- **Delete** selected region with `Delete` or `Backspace`
- **Copy** selected region to clipboard

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `R` | Rectangle tool |
| `L` | Line tool |
| `A` | Arrow tool |
| `D` | Diamond tool |
| `T` | Text tool |
| `F` | Freehand tool |
| `V` | Select tool |
| `E` | Eraser tool |
| `Ctrl+Z` | Undo |
| `Ctrl+Shift+Z` | Redo |
| `Ctrl+C` | Copy ASCII |
| `Ctrl+X` | Cut selected region |
| `Ctrl+V` | Paste from clipboard |
| `Delete/Backspace` | Delete selected region |
| `Space+Drag` | Pan canvas |
| `Scroll` | Zoom |

## Architecture

### Core Layer (Pure Rust)

The core layer contains all business logic with no WASM/web dependencies:

- **Grid Model**: Flat `Vec<Cell>` with O(1) index math
- **Tools**: Trait-based tool system with `ToolResult`
- **Commands**: Command pattern for atomic, undoable operations
- **History**: Ring buffer with configurable depth

### Render Layer

- **Canvas Renderer**: Converts grid to render commands
- **Dirty Rects**: Only redraws changed regions
- **Font Metrics**: Precise monospace character measurement

### WASM Layer

- **Bindings**: `AsciiEditor` class exposed to JavaScript
- **Events**: Pointer, keyboard, wheel event translation
- **Clipboard**: Async clipboard API integration

## API Reference

### `AsciiEditor`

```typescript
// Create editor
const editor = new AsciiEditor(width: number, height: number);

// Properties
editor.width: number;
editor.height: number;
editor.tool: string;
editor.zoom: number;
editor.pan: [number, number];
editor.canUndo: boolean;
editor.canRedo: boolean;
editor.needsRedraw: boolean;

// Methods
editor.setTool(name: string): void;
editor.setToolByShortcut(key: string): boolean;
editor.setBorderStyle(style: string): void;
editor.setZoom(zoom: number): void;
editor.setPan(x: number, y: number): void;
editor.setFontMetrics(charWidth: number, lineHeight: number, size: number): void;

// Events
editor.onPointerDown(x: number, y: number): EventResult;
editor.onPointerMove(x: number, y: number): EventResult;
editor.onPointerUp(x: number, y: number): EventResult;
editor.onKeyDown(key: string, ctrl: boolean, shift: boolean): EventResult;
editor.onKeyUp(key: string): void;
editor.onWheel(delta: number, x: number, y: number): EventResult;

// Actions
editor.undo(): boolean;
editor.redo(): boolean;
editor.clear(): void;
editor.exportAscii(): string;

// Rendering
editor.getRenderCommands(): RenderCommand[];
editor.getDirtyRenderCommands(): RenderCommand[];
editor.requestRedraw(): void;
```

### `EventResult`

```typescript
interface EventResult {
    needs_redraw: boolean;
    tool: string;
    can_undo: boolean;
    can_redo: boolean;
    should_copy: boolean;
    ascii: string | null;
}
```

## Testing

### Core Tests

```bash
cargo test
```

### WASM Tests

```bash
wasm-pack test --headless --firefox
```

## Performance

- **Dirty-Rect Rendering**: Only redraws modified regions
- **Zero Per-Frame Allocations**: Pre-allocated buffers
- **SmallVec**: Stack allocation for small collections
- **Optimized WASM**: LTO, stripping, `opt-level = "z"`

Target: < 1.5MB WASM bundle

## Deployment

### Netlify

This project is configured for one-click deployment to Netlify.

1. Connect your repository to Netlify.
2. The `netlify.toml` file automatically configures:
   - **Build command**: `npm run netlify:build`
   - **Publish directory**: `dist`
   - **Environment**: Node 22, Rust stable
3. Netlify will handle WASM MIME types and SPA routing automatically.

### Local Production Build

```bash
npm run build
```
The final static assets will be in the `dist/` directory.

## Performance

- **Dirty-Rect Rendering**: Only redraws modified regions, significantly reducing CPU usage during edits.
- **Zero Per-Frame Allocations**: Pre-allocated buffers for core rendering loops.
- **SmallVec**: Stack allocation for small collections to avoid heap thrashing.
- **Optimized WASM**: LTO, stripping, and `opt-level = "z"` for minimum binary size.
- **Instrumentation**: `AsciiEditor` tracks `fullRenderCount` and `dirtyRenderCount` for performance auditing.
- **History Coalescing**: Consecutive small draw operations (like freehand drawing) are automatically coalesced into single undo/redo steps.

## Browser Support

- Chrome 80+
- Firefox 75+
- Safari 14+
- Edge 80+

Requires:
- WebAssembly
- Clipboard API
- CSS Grid

## Development

### Build WASM

```bash
npm run build:wasm
```

### Run Linter

```bash
cargo clippy -- -D warnings
cargo fmt --check
```

### Size Check

```bash
ls -lh pkg/ascii_canvas_bg.wasm
```

## Troubleshooting

### WASM build fails with "cargo: command not found"
Ensure Rust is installed and `cargo` is in your PATH:
```bash
source ~/.cargo/env
```

### wasm-pack install fails
Try specifying the target explicitly:
```bash
wasm-pack build --target web
```

### Browser shows "WebAssembly not supported"
Ensure you're using a modern browser (Chrome 80+, Firefox 75+, Safari 14+, Edge 80+).

### Build errors about missing dependencies
```bash
cargo update
```

## Version

Current: **v0.1.0**

## License

MIT License

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Format code: `cargo fmt`
6. Submit a pull request

## Acknowledgments

- Inspired by [Monodraw](https://monodraw.helftone.com/) and [Figma](https://figma.com)
- Built with [wasm-bindgen](https://rustwasm.github.io/wasm-bindgen/)
- Font: [JetBrains Mono](https://www.jetbrains.com/lp/mono/)
