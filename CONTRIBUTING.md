# Contributing to ASCII Canvas

Thank you for your interest in contributing!

## Development Setup

### Prerequisites

- Rust (latest stable)
- Node.js 18+
- wasm-pack (`cargo install wasm-pack`)

### Quick Start

```bash
# Clone the repository
git clone https://github.com/anomalyco/ascii-canvas.git
cd ascii-canvas

# Install Node dependencies
npm install

# Build WASM
wasm-pack build --release --target web --out-dir web/pkg

# Start development server
cd web && npm run dev
```

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` to catch common mistakes
- Keep lines under 100 characters when possible

## Testing

```bash
# Run Rust unit tests
cargo test

# Run E2E tests (requires dev server)
npx playwright test --project=chromium
```

## Pull Request Process

1. Create a feature branch from `main`
2. Make your changes
3. Ensure all tests pass
4. Update documentation if needed
5. Submit a pull request

## Areas to Contribute

- Drawing tools (select, rectangle, line, arrow, diamond, text, freehand, eraser)
- Border styles
- UI/UX improvements
- Performance optimizations
- Documentation

## Code of Conduct

Be respectful and constructive. Follow the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct).
