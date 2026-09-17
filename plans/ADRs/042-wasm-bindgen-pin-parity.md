# ADR-042: Coordinated wasm-bindgen 0.2.128 Upgrade + Pin-Parity Sensor

## Status
Accepted — 2026-09-17

## Context

`wasm-bindgen` requires the Rust dependency crate and the `wasm-bindgen-cli`
binary to share the exact schema version. Dependabot bumps `Cargo.toml` alone
(`=0.2.126` → `=0.2.127` in #769f42f), while the CLI pin lives separately in
`mise.toml`, `package.json` `netlify:build`, and CI (via mise-action). Prior
skew caused `WASM Build` failures (`schema 0.2.128 vs 0.2.126`, harness L-005,
PR #186 closed unmerged for the same class).

At the start of this work the tree was skewed three ways: `Cargo.toml`
`=0.2.127`, `mise.toml`/`netlify:build`/gate hints `0.2.126`, `Cargo.lock`
(stale, gitignored) `0.2.121`, local `~/.cargo/bin/wasm-bindgen` `0.2.121`,
while crates.io latest was `0.2.128`.

## Decision

Coordinated upgrade to **0.2.128** everywhere (per 2026 best practice: move dep
and CLI together, then verify with a real `build:wasm`):

- `Cargo.toml`: `wasm-bindgen = "=0.2.128"` (+ `cargo update -p` for lockfile;
  note `Cargo.lock` is gitignored in this repo so CI resolves fresh).
- `mise.toml`: `"cargo:wasm-bindgen-cli" = "0.2.128"`.
- `package.json` `netlify:build`: `cargo install wasm-bindgen-cli --version 0.2.128`.
- `scripts/quality-gates.sh`: FIX hints updated to 0.2.128 **plus a new
  computational sensor** (§2b "wasm-bindgen pin parity (L-005)") that greps the
  three pins and fails fast with a FIX hint on skew.
- Local CLI: `cargo install wasm-bindgen-cli --version =0.2.128 --locked`.
- Incidental (required to keep `-D warnings` green under the newer toolchain's
  Clippy): inline format args (`uninlined_format_args`) in
  `src/lib.rs`, `src/core/commands/draw.rs`, `src/ui/toolbar.rs`,
  `src/wasm/render_api.rs`, `src/core/ascii_export.rs`, and `benches/`; replaced
  `assert!(!VERSION.is_empty())` with `assert_eq!(VERSION,
  env!("CARGO_PKG_VERSION"))` (`const_is_empty`).

## Consequences

- `wasm-bindgen` schema matches between compiled WASM and CLI; `build:wasm`
  verified locally (261105 → 243260 bytes after `wasm-opt -O3`, within the
  1.5MB budget).
- Future lone-Dependabot bumps now fail locally at gate time with an actionable
  FIX instead of surfacing only in CI's WASM job.
- Precedent: any future coordinated bump must touch the same four files
  (+ lockfile refresh) and re-run `npm run build:wasm`. Dependabot config
  groups all deps weekly; consider excluding `wasm-bindgen` or adding a
  CODEOWNERS note (not done in this change).
