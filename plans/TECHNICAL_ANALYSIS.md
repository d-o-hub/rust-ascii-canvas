# Technical Analysis - export_trimmed max_width optimization

## Context
When exporting an ASCII grid with `max_width` applied, `export_trimmed` was allocating a large vector of string slices: `result.lines().collect::<Vec<&str>>()`, and reallocating a new `String` object without preallocating any capacity.

## Changes Made
1. **Removed Vector Allocation**: We replaced `.collect()` with `.enumerate()` on the `.lines()` iterator directly.
2. **Pre-allocated String Capacity**: Used `String::with_capacity(result.len())` to pre-allocate memory for the modified string.
3. **Newline Handling**: Managed newlines via `if i > 0 { limited.push('\n') }` to match the exact string behavior without relying on a full collection `len()`.
4. **Char Bound Safety**: Preserved `chars().count()` for bounds-check instead of `.len()` to prevent panics and handle multibyte correctly, while safely finding the byte index using `.char_indices()`.

## Results
Benchmark of `export_grid` with a `max_width` over 10k cells x 100 iterations.
* Baseline: 259.069µs
* Optimized: 249.404µs (4% improvement, but more crucially eliminates heap allocations of Vec<&str> which helps high-frequency JS/WASM interop performance).
