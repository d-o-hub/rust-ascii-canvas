# ADR-040: App-Only Distribution — crates.io / npm Publishing Decision

## Status
Accepted - 2026-07-28

## Context

Under issue **F-33** (crates.io / npm package metadata), the project was tasked with deciding whether to publish the core Rust crate and/or npm packages to their respective public registries, or explicitly designate and document the project as an **"app-only"** distribution.

Currently:
- The project has a standard root structure with `Cargo.toml` representing the Rust-to-WASM portion and `package.json` representing the workspace dependencies and helper scripts.
- The `web/` directory contains a fully featured, single-page static web application built using Vite, TypeScript, and the compiled WASM module.
- `Cargo.toml` contains some package metadata (version `0.1.1`, description, license, repository, etc.), as do the root and web `package.json` files.

## Decision

We decide **not** to publish `ascii-canvas` as a reusable library crate on crates.io or as a reusable npm library package. Instead, we formally designate the project as an **"app-only" distribution**.

The primary artifact of this repository is the deployed ASCII Canvas Editor web application. It is distributed exclusively as a hosted static website (e.g., via Netlify or local production build in `dist/`), not as a library for external consumption.

## Rationale

1. **Monolithic Design & Strong Coupling**:
   The WASM bindings (`src/wasm/`), custom pixel-buffer rendering pipeline, dirty-rect tracking, and front-end font atlas logic are highly specific, custom-tailored, and tightly integrated with the browser-based Vite application in `/web`. They are not designed to serve as a general-purpose, reusable Rust drawing library or standard WASM diagramming package.

2. **No Clear Consumer Base**:
   Publishing `ascii-canvas` on crates.io or npm would result in package packages that provide very little value to general developers, as the APIs are geared specifically toward powering our own dark-themed Figma-like diagramming UI.

3. **Maintenance Overhead**:
   Supporting public package registry releases requires extensive version management, keeping public API surfaces stable, managing backward compatibility, and managing registry credentials and CI pipelines. Given that the goal is an end-user application, this overhead is not justified.

4. **Clarity for Contributors**:
   Documenting this decision clearly prevents future contributors from attempting to refactor the workspace in anticipation of a dual-library publication model.

## Consequences

- **Backlog & Issue Closure**:
  Issue **F-33** is closed as `wontfix (app-only)`.
- **Repository Metadata**:
  The existing metadata in `Cargo.toml` and `package.json` (such as repo URLs, authors, descriptions, licenses) remains accurate for repository visibility but will not be used for publishing.
- **Distribution Method**:
  Static-asset hosting remains the single target distribution method, as currently implemented with Netlify (`netlify.toml`) and local standard builds (`npm run build`).
- **No Registry Publish**:
  No `cargo publish` or `npm publish` workflows will be established or run.

## References
- [F-33 Backlog Item](./../FOLLOW_UPS.md)
- [ADR-005b: Package Metadata Consistency](./005b-package-metadata-consistency.md)
