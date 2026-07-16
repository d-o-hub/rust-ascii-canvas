# Contributing to ASCII Canvas

Thank you for your interest in contributing!

## Development setup

### Prerequisites

- Rust (stable) with target `wasm32-unknown-unknown`
- Node.js 22+ and pnpm 10
- [mise](https://mise.jdx.dev/) recommended (`mise.toml` pins wasm-bindgen-cli **0.2.121** and binaryen)

### Quick start

```bash
git clone https://github.com/d-o-hub/rust-ascii-canvas.git
cd rust-ascii-canvas

# Toolchain (if using mise)
mise install

# JS deps (root + web)
pnpm install
cd web && pnpm install && cd ..

# Build WASM into web/pkg
pnpm run build:wasm

# Dev server
pnpm run dev
```

## Quality harness (keep quality left)

This repo uses a **coding-agent harness**: guides + sensors. See `AGENTS.md` and `agents-docs/harness.md`.

| Tier | Command | When |
|------|---------|------|
| **fast** | `npm run gate:fast` | Every meaningful change / pre-commit |
| **full** | `npm run gate:full` | Before opening a PR |
| Architecture only | `npm run check:architecture` | Layer / import changes |

Optional git hook:

```bash
ln -sf ../../scripts/pre-commit-fast.sh .git/hooks/pre-commit
```

### Manual checks

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cd web && pnpm lint && pnpm exec tsc --noEmit && pnpm test
pnpm run build:wasm && pnpm run check-size
npx playwright test --project=chromium
```

## Architecture

- `src/core/` — pure Rust (no WASM/browser APIs)
- `src/render/`, `src/ui/` — may use core; not wasm
- `src/wasm/` — JS bindings only
- `web/` — Vite/TypeScript UI

Details: `agents-docs/architecture.md`. File size target: ≤500 lines (known debt in `.loc-allowlist`).

## Code style

- `cargo fmt` before commit
- Clippy clean with `-D warnings`
- Prefer tests next to Rust code (`#[cfg(test)]`) for units; `tests/` for public API

## Pull request process

1. Branch from `main`
2. Keep `gate:fast` green while iterating; `gate:full` before review
3. Fill `.github/PULL_REQUEST_TEMPLATE.md` honestly
4. Link issues (`Closes #…`)
5. Call out harness/CI/doc changes in the PR body

## Areas to contribute

- Drawing tools, border styles, UI/UX
- Layers, export, clipboard fidelity
- Performance, docs, tests, harness sensors

## Code of conduct

Be respectful and constructive. Follow the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct).
