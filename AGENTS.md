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
