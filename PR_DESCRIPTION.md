## 🧪 Testing Improvement: Add missing test for approx_eq function

### 🎯 What
This PR addresses a testing gap in the `src/utils/math.rs` module. The `approx_eq` function, which performs a straightforward but critical float comparison within a specified epsilon, was missing a dedicated unit test.

### 📊 Coverage
The new test suite covers the following scenarios for floating point comparisons:
*   **Exact equality**: e.g., `approx_eq(1.0, 1.0, 1e-6)`
*   **Within epsilon (edge boundaries)**: Ensures values strictly inside the threshold return true.
*   **Outside epsilon**: Ensures values slightly outside the threshold return false.
*   **Zero and Negative values**: Validates correct absolute difference handling for negative and zero edge cases (`0.0`, `-0.0`).
*   **Special floating-point values (NaN and Infinity)**: Confirms that Infinity and NaN behave deterministically and correctly yield `false`.

### ✨ Result
The reliability and coverage of mathematical float comparison in the editor's utility module has been significantly increased. We are now protected against regressions related to subtle floating-point errors.

This PR also includes a fix for `ascii_export.rs` to correctly handle `max_width` limits with multi-byte unicode characters, replacing byte slicing with `.chars().take()`.

Additionally, restored the correct `npm run build` instructions in the `README.md` to avoid missing the critical post-processing steps.

Finally, added back the missing `test_export_max_width` and `test_export_max_width_multibyte` unit tests for `ascii_export.rs`.
