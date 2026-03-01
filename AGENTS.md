# Agent Best Practices - ASCII Canvas Project

## Agent System Overview

This project uses specialized agents with skills for different tasks. All agents should document their work in the plans/ folder.

## Best Practices

### Task Execution
1. **Analyze** - Understand requirements before acting
2. **Plan** - Create steps in plans/ with goap with adr before execution
3. **Execute** - Run commands and tests
4. **Document** - Update plans/ with results

### Documentation Standards
- All architectural decisions → plans/ folder
- Use ADR format: title, status, date, context, decision, consequences
- Update PROJECT_STATUS.md with test results
- Add technical findings to TECHNICAL_ANALYSIS.md
- **Always document learnings** - Any new tool, workflow, fix, or best practice discovered during development must be added to plans/ (e.g., TECHNICAL_ANALYSIS.md for technical findings, new ADRs for decisions)

### Testing Workflow
1. Build dependencies (WASM, npm packages)
2. Run tests: `cargo test`, `npx playwright test`
3. Document results in plans/
4. Update ADRs with learnings

### File Organization
```
plans/
├── PROJECT_STATUS.md      # Current project state
├── TECHNICAL_ANALYSIS.md  # Technical findings
└── ADRs/                 # Architectural decisions
    └── *.md

.agents/skills/

```

### Code Quality
- Keep files under 500 LOC
- Use consistent naming conventions
- Add documentation comments for public APIs
- Run tests before marking tasks complete

### Communication
- Provide structured output with summaries
- Include specific issues and recommendations
- Verify ADRs match implementation

## Production Readiness Learnings (2026-03-01)

### FromStr Trait Implementation
When implementing string parsing in Rust, prefer the `FromStr` trait over standalone methods:

```rust
// ❌ Avoid: Clippy warns about this
impl MyType {
    pub fn from_str(s: &str) -> Self { ... }
}

// ✅ Prefer: Implement the standard trait
impl std::str::FromStr for MyType {
    type Err = std::convert::Infallible; // or appropriate error type
    
    fn from_str(s: &str) -> Result<Self, Self::Err> { ... }
}
```

### Safe Downcasting Pattern
Replace unsafe pointer casts with `Any` trait downcasting:

```rust
// ❌ Avoid: Unsafe and error-prone
let tool_ptr = self.tools.get_mut(&tool_id).unwrap() as *mut dyn Tool;
let concrete = unsafe { &mut *(tool_ptr as *mut LineTool) };

// ✅ Prefer: Safe Any trait downcasting
// 1. Add to Tool trait:
fn as_any_mut(&mut self) -> &mut dyn Any;

// 2. Implement in each tool:
fn as_any_mut(&mut self) -> &mut dyn Any { self }

// 3. Use for downcasting:
if let Some(line_tool) = tool.as_any_mut().downcast_mut::<LineTool>() {
    line_tool.set_direction(direction);
}
```

### GitHub Actions Best Practices 2026

1. **Prefer direct commands over actions when customization needed**:
   ```yaml
   # ❌ Avoid: Limited customization
   - uses: jetli/wasm-pack-action@v0.4.0
     with:
       version: latest  # Network errors!
   
   # ✅ Prefer: Full control
   - run: |
       cargo install wasm-pack --version 0.12.1
       wasm-pack build --release --target web --out-dir web/pkg
   ```

2. **Artifact sharing requires matching paths**:
   ```yaml
   - uses: actions/upload-artifact@v4
     with:
       name: wasm pkg
       path: web/pkg  # Must match download path
   
   - uses: actions/download-artifact@v4
     with:
       name: wasm pkg
       path: web/pkg  # Must match upload path
   ```

3. **Version pinning avoids network issues**:
   - Pin to specific versions: `wasm-pack --version 0.12.1`
   - Not `v0.12.1` (different format!)
   - Not `latest` (causes timeouts)

### Playwright E2E Best Practices 2026

1. **Use role-based locators over CSS selectors**:
   ```typescript
   // ❌ Avoid: Brittle CSS selectors
   await page.click('.tool-btn[data-tool="rectangle"]');
   
   // ✅ Prefer: Semantic role-based locators
   await page.getByRole('button', { name: 'Rectangle' }).click();
   ```

2. **Replace waitForTimeout with proper assertions**:
   ```typescript
   // ❌ Avoid: Flaky timeouts
   await page.waitForTimeout(1000);
   
   // ✅ Prefer: Auto-waiting assertions
   await expect(page.getByRole('button', { name: 'Undo' })).toBeEnabled();
   ```

3. **Page Object Model for maintainability**:
   ```typescript
   // pages/EditorPage.ts
   export class EditorPage {
     constructor(private page: Page) {}
     
     async selectTool(toolName: string) {
       await this.page.getByRole('button', { name: toolName }).click();
     }
     
     async drawRectangle(x: number, y: number, width: number, height: number) {
       // Encapsulated drawing logic
     }
   }
   ```

4. **Cross-browser testing configuration**:
   ```typescript
   // playwright.config.ts
   projects: [
     { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
     { name: 'firefox', use: { ...devices['Desktop Firefox'] } },
     { name: 'webkit', use: { ...devices['Desktop Safari'] } },
   ],
   fullyParallel: true,
   ```

### TypeScript Strict Mode Best Practices 2026

1. **Defensive DOM element access**:
   ```typescript
   // ❌ Avoid: Non-null assertions
   const canvas = document.getElementById('canvas')!;
   
   // ✅ Prefer: Type guards
   const canvas = document.getElementById('canvas');
   if (!(canvas instanceof HTMLCanvasElement)) {
     throw new Error('Canvas element not found or wrong type');
   }
   ```

2. **Use unknown instead of any**:
   ```typescript
   // ❌ Avoid: any bypasses type checking
   function processData(data: any) { ... }
   
   // ✅ Prefer: unknown requires type validation
   function processData(data: unknown) {
     if (typeof data === 'string') { ... }
   }
   ```

3. **ARIA attributes for accessibility**:
   ```html
   <div role="dialog" aria-modal="true" aria-labelledby="modal-title">
     <h2 id="modal-title">Keyboard Shortcuts</h2>
     <button aria-label="Close">×</button>
   </div>
   ```

### Summary Checklist for Future Production Readiness

- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] Run `cargo test` (unit + integration + doc tests)
- [ ] Build WASM with `wasm-pack build --release`
- [ ] Run TypeScript check: `tsc --noEmit`
- [ ] Run E2E tests: `npx playwright test`
- [ ] Verify GitHub Actions CI passes
- [ ] Update ADRs for any new decisions
- [ ] Document learnings in TECHNICAL_ANALYSIS.md
- [ ] Create atomic commits with conventional format
- [ ] Push feature branch and create PR

---

*Last Updated: 2026-03-01*
*Part of Production Readiness 2026 Initiative*
