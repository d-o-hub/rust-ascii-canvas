# ASCII Canvas Editor

A **production-grade ASCII diagram editor** built with Rust and WebAssembly. Features a polished dark UI inspired by Figma, running at 60 FPS with zero browser text-selection artifacts.

## Features

- 🎨 **8 Drawing Tools**: Rectangle, Line, Arrow, Diamond, Text, Freehand, Select, Eraser
- ✋ **Select Tool**: Click and drag to select regions, move selected objects, and see blue highlights
- 🖼️ **6 Border Styles**: Single, Double, Heavy, Rounded, ASCII, Dotted
- 🔠 **Dynamic Font Atlas**: High-fidelity character rendering using frontend rasterization (JetBrains Mono)
- ↩️ **Full Undo/Redo**: Command pattern with configurable history depth
- 🔍 **Zoom & Pan**: Mouse wheel zoom, Space+drag panning
- 📋 **One-Click Copy**: Export trimmed ASCII to clipboard
- ⌨️ **Keyboard-First**: Full keyboard shortcut support
- 🌙 **Dark Theme**: Professional Figma-inspired dark UI
- ⚡ **60 FPS Rendering**: Dirty-rect optimization, high-performance WASM pixel buffer path
- 📦 **Offline-First**: No external dependencies, works offline

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (1.75+)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) (0.13.1+)
- [Node.js](https://nodejs.org/) (22+)

### Build

```shell
# Clone the repository
git clone https://github.com/d-o-hub/rust-ascii-canvas.git
cd rust-ascii-canvas

# Install dependencies and build both WASM and Web
npm install
npm run build

# Start dev server
npm run dev &
```

The editor will be available at `http://localhost:3003`.

## Project Structure

```
ascii-canvas/
├── Cargo.toml              # Rust dependencies
├── package.json            # Root workspace config
├── src/
│   ├── lib.rs              # Library entry point
│   ├── core/               # Pure Rust logic (no WASM)
│   ├── render/             # Canvas rendering
│   ├── wasm/               # WASM bindings
│   ├── ui/                 # UI components
│   └── utils/              # Utilities
├── web/
│   ├── index.html          # HTML template
│   ├── style.css           # Dark theme styles
│   ├── main.ts             # TypeScript entry
│   ├── package.json        # Frontend config
│   └── vite.config.ts      # Vite build config
├── tests/                  # Rust integration tests
├── benches/                # Rust benchmarks
├── e2e/                    # Playwright E2E tests
├── playwright.config.ts    # Playwright config
└── wasm-pack.toml          # wasm-pack config
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
| `V` | Select tool |
| `R` | Rectangle tool |
| `L` | Line tool |
| `A` | Arrow tool |
| `D` | Diamond tool |
| `T` | Text tool |
| `F` | Freehand tool |
| `E` | Eraser tool |
| `B` | Cycle Border Style |
| `Ctrl+Z` | Undo |
| `Ctrl+Shift+Z` / `Ctrl+Y` | Redo |
| `Ctrl+C` | Copy ASCII |
| `Ctrl+X` | Cut selected region |
| `Ctrl+V` | Paste from clipboard |
| `Ctrl+A` | Select all |
| `Delete/Backspace` | Delete selected region |
| `Space+Drag` | Pan canvas |
| `Scroll` | Zoom |
| `Escape` | Cancel/Deselect |

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
- **Dynamic Font Atlas**: Rasterizes characters on the frontend and uploads alpha masks to WASM for pixel-perfect alignment
- **Font Metrics**: Precise 8x20 monospace character measurement

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

### Unit Tests (Rust)

```shell
npm run test:unit
```

### End-to-End Tests (Playwright)

```shell
npm run test:e2e
```

### Frontend Tests (Vitest)

```shell
cd web
npm test
```



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

```shell
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

### Build WASM (Development)

```bash
wasm-pack build --dev --target web
```

### Build WASM (Production)

```bash
wasm-pack build --release --target web
```

### Run Linter

```bash
cargo clippy -- -D warnings
cargo fmt --check
```

### Size Check

```shell
npm run check-size
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
