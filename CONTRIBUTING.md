# Contributing to ASCII Canvas

Thank you for your interest in contributing!

## Development setup

### Prerequisites

- Rust (stable) with target `wasm32-unknown-unknown`
- Node.js 22+, Python 3.10+, and pnpm **10.34.5** (both manifests and CI pin it)
- [mise](https://mise.jdx.dev/) recommended (`mise.toml` pins wasm-bindgen-cli **0.2.128** and binaryen)

### Quick start

```bash
git clone https://github.com/d-o-hub/rust-ascii-canvas.git
cd rust-ascii-canvas

# Toolchain (if using mise)
mise install

# JS deps (root + web)
pnpm install --frozen-lockfile
(cd web && pnpm install --frozen-lockfile)

# Build WASM into web/pkg
pnpm run build:wasm

# Dev server
pnpm run dev       # loopback-only, the default (http://127.0.0.1:3003)
pnpm run dev:lan   # opt in to LAN visibility for device testing (binds 0.0.0.0)
```

`pnpm run dev` binds `127.0.0.1` on purpose. Three 2026 Vite advisories —
[GHSA-v2wj-q39q-566r](https://github.com/vitejs/vite/security/advisories/GHSA-v2wj-q39q-566r)
(`server.fs.deny` bypass), GHSA-p9ff-h696-f583 (arbitrary file read through the
HMR WebSocket) and
[CVE-2026-53571](https://github.com/advisories/GHSA-FX2H-PF6J-XCFF) (Windows
alternate-path bypass) — all list *"explicitly exposes the Vite dev server to the
network"* as a precondition, so the default is loopback and `dev:lan` is the
deliberate exception.

Two things to know before reaching for `dev:lan`:

- **WSL2 is different.** `dev:lan` is not by itself enough to reach the server
  from a Windows host on WSL2 — see
  [Microsoft's WSL networking notes](https://learn.microsoft.com/en-us/windows/wsl/networking#accessing-a-wsl-2-distribution-from-your-local-area-network-lan).
- **Port 3003 is strict.** If it is taken, the dev server exits instead of moving
  to 3004, because focused Playwright runs use `http://127.0.0.1:3003`. A silent port change would run the suite against
  whatever stale server still holds 3003.

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
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all
python3 scripts/wasm-freshness.py --ensure
(cd web && pnpm lint && pnpm test)
pnpm run test:wasm
pnpm run check-size
pnpm exec playwright test --project=chromium  # Playwright owns dev startup
# Production-shaped local/CI evidence, without a hand-started server:
pnpm run build:web
PRODUCTION_E2E=1 pnpm exec playwright test --project=chromium
```

Both workspaces must use the pinned pnpm version: mixing pnpm 10 and 11 can
make `pnpm run` attempt an implicit reinstall and fail without a TTY. Confirm
`pnpm --version` at the root and in `web/`. For an existing cross-major install,
explicitly run `CI=true pnpm install --frozen-lockfile` in both directories; never
fall back to an unlocked resolve. The CI coherence fixtures enforce pin parity.

Full gates require reachable advisory registries and installed cargo-audit /
cargo-deny: **unverified is nonzero**, not a successful skipped audit. The
production server uses strict port 4173 and is torn down by Playwright.
`BASE_URL=https://<deploy-preview>` skips local startup for shadow testing.

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
3. Version bumps: edit `VERSION` (single source), then run `./scripts/propagate-version.sh`
4. Fill `.github/PULL_REQUEST_TEMPLATE.md` honestly
5. Link issues (`Closes #…`)
6. Call out harness/CI/doc changes in the PR body

## Areas to contribute

- Drawing tools, border styles, UI/UX
- Layers, export, clipboard fidelity
- Performance, docs, tests, harness sensors

## Code of conduct

Be respectful and constructive. Follow the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct).
