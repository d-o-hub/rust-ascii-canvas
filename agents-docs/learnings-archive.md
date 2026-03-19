# Historical Learnings Archive

## Production Readiness Learnings (2026-03-01)

### FromStr Trait Implementation
When implementing string parsing in Rust, prefer the `FromStr` trait over standalone methods.

```rust
// ✅ Prefer: Implement the standard trait
impl std::str::FromStr for MyType {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> { ... }
}
```

### Safe Downcasting Pattern
Replace unsafe pointer casts with `Any` trait downcasting.

### GitHub Actions Best Practices 2026

1. **Prefer direct commands over actions** when customization is needed.
2. **Artifact sharing requires matching paths**.
3. **Version pinning** avoids network issues: use `--version 0.12.1`.

### Playwright E2E Best Practices 2026

1. **Use role-based locators** over CSS selectors.
2. **Replace waitForTimeout** with proper assertions.
3. **Page Object Model** for maintainability.
4. **Cross-browser testing** configuration.

### TypeScript Strict Mode Best Practices 2026

1. **Defensive DOM element access**.
2. **Use unknown instead of any**.
3. **ARIA attributes** for accessibility.

---

## Learnings from Phase 1 Implementation (2026-03-03)

### Rust Code Hygiene
- Finding unused dependencies.
- Dead code detection.
- Duplicate type patterns.

### ADR Management
- Numbering conflicts and format (e.g., 001-099).

---

## Select Tool Move Implementation (2026-03-04)
- Implemented move functionality at editor level (not tool level) to reuse existing clipboard infrastructure.

---

## Release Workflow (2026-03-04)
- Use semantic versioning and automated releases with guard-rails (`cargo test`, `clippy`, `fmt`, `wasm-pack`).

---

## WASM Event Handling (2026-03-20)
- **Text Tool Input**: Browser `keydown` events provide strings like `"Enter"`, `"Backspace"`, or `"Tab"`. These must be explicitly mapped to control characters (e.g., `\n`, `\x08`, `\t`) before being passed to the Rust logic. Alphanumeric keys can be reliably extracted by checking if the `key` string length is exactly 1.
- **Dynamic Font Atlas**: To ensure perfect character and symbol rendering in the WASM pixel buffer path, glyphs are rasterized on the frontend using a hidden canvas at startup. The resulting alpha mask data is then uploaded to the WASM `FontAtlas`. This bypasses limitations of hardcoded bitmap patterns and allows the editor to use any font loaded in the browser (e.g., JetBrains Mono).
- **Font Loading Synchronization**: When using a dynamic font atlas, it is critical to wait for `document.fonts.ready` before rasterizing glyphs. Rasterizing before the web font is fully loaded results in fallback "empty" or "blocky" glyphs being cached in the WASM memory.
