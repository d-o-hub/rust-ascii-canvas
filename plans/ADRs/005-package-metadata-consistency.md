# ADR-005: Package Metadata Consistency

## Status

**Proposed** - 2026-02-26

## Context

The project has inconsistent and incomplete metadata across `Cargo.toml` and `package.json`, reducing discoverability and causing confusion.

### Current Issues

#### Cargo.toml

```toml
[package]
name = "ascii-canvas"
version = "0.1.0"
edition = "2021"
description = "A fast ASCII diagram editor compiled to WebAssembly"
license = "MIT"
repository = "https://github.com/user/ascii-canvas"  # Placeholder URL!
# Missing: authors, readme, keywords, categories, documentation
```

#### package.json (root)

```json
{
  "name": "ascii-canvas",
  "version": "1.0.0",
  "license": "ISC",           # Conflicts with MIT in Cargo.toml!
  "keywords": [],             # Empty
  "author": "",               # Empty
  "scripts": {
    "test": "echo \"Error: no test specified\" && exit 1"  # Placeholder!
  }
}
```

### Version Mismatch

| File | Version |
|------|---------|
| Cargo.toml | 0.1.0 |
| package.json (root) | 1.0.0 |
| package.json (web) | (not checked but likely different) |

### License Mismatch

| File | License |
|------|---------|
| Cargo.toml | MIT |
| package.json (root) | ISC |
| LICENSE file | Missing |

## Decision

### 1. Update Cargo.toml

```toml
[package]
name = "ascii-canvas"
version = "0.1.0"
edition = "2021"
description = "A fast ASCII diagram editor compiled to WebAssembly"
license = "MIT"
repository = "https://github.com/anomalyco/ascii-canvas"  # Actual URL
readme = "README.md"
keywords = ["wasm", "webassembly", "ascii", "diagram", "editor", "rust"]
categories = ["wasm", "web-programming", "graphics"]
authors = ["Anomaly"]
documentation = "https://docs.rs/ascii-canvas"
homepage = "https://github.com/anomalyco/ascii-canvas"

[badges]
maintenance = { status = "actively-developed" }
```

### 2. Update Root package.json

```json
{
  "name": "ascii-canvas",
  "version": "0.1.0",
  "description": "A production-grade ASCII diagram editor built with Rust and WebAssembly",
  "license": "MIT",
  "keywords": ["wasm", "webassembly", "ascii", "diagram", "editor", "rust"],
  "author": "Anomaly",
  "repository": {
    "type": "git",
    "url": "https://github.com/anomalyco/ascii-canvas.git"
  },
  "bugs": {
    "url": "https://github.com/anomalyco/ascii-canvas/issues"
  },
  "homepage": "https://github.com/anomalyco/ascii-canvas#readme",
  "scripts": {
    "build:wasm": "wasm-pack build --release --target web --out-dir web/pkg",
    "build": "npm run build:wasm && cd web && npm run build",
    "dev": "cd web && npm run dev",
    "test": "cargo test && npx playwright test --project=chromium",
    "test:unit": "cargo test",
    "test:e2e": "npx playwright test",
    "lint": "cargo clippy -- -D warnings && cargo fmt --check",
    "fmt": "cargo fmt"
  }
}
```

### 3. Sync Versions

Use semantic versioning consistently:
- Both files should reference the same version
- Consider using a tool like `cargo-release` or `standard-version` for synchronization

## Consequences

### Benefits

1. **Crates.io**: Complete metadata improves discoverability
2. **npm**: Proper keywords help users find the project
3. **License Clarity**: Consistent MIT license across all files
4. **Automation**: Proper scripts enable CI/CD integration
5. **Professionalism**: Complete metadata signals quality

### Package Manager Display

After fixes, the project will display properly on:
- crates.io (Rust)
- npmjs.com (if published)
- GitHub package registry

## References

- [Cargo.toml format](https://doc.rust-lang.org/cargo/reference/manifest.html)
- [npm package.json best practices](https://docs.npmjs.com/cli/v10/configuring-npm/package-json)
- [Semantic Versioning](https://semver.org/)
