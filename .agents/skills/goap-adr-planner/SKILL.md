---
name: goap-adr-planner
description: Goal-Oriented Action Planning (GOAP) with Architectural Decision Records (ADR). Use when planning multi-step tasks, analyzing architectural choices, creating plans in plans/ folder, documenting decisions with ADRs, or managing project status tracking.
---

# GOAP-ADR Planner

## Purpose

Provides structured planning using Goal-Oriented Action Planning combined with Architectural Decision Records. Manages project progress through the plans/ folder with ADRs, PROJECT_STATUS.md, and TECHNICAL_ANALYSIS.md.

## When to Use

- Creating multi-step implementation plans
- Documenting architectural decisions with ADRs
- Tracking project status and progress
- Analyzing technical tradeoffs
- Planning refactoring or new features

## Quick Start

### Invoke When

- User asks to "plan" something, "create a roadmap", or "break down" a task
- Making architectural/technical decisions that need documentation
- Updating project status or tracking progress
- Creating new ADRs for design choices

### Don't Invoke When

- Simple one-off coding tasks (just do it)
- Quick questions without planning context
- Reading existing code without modification

## Folder Structure

```
plans/
├── PROJECT_STATUS.md      # Current project state, test results, blockers
├── TECHNICAL_ANALYSIS.md  # Technical findings, research notes
└── ADRs/                 # Architectural Decision Records
    └── *.md              # ADR files (e.g., 001-title.md)
```

## Workflow

### 1. Analyze Request

Break down the user's request into:

- **Goals**: What needs to be achieved?
- **Actions**: What steps are needed?
- **Dependencies**: What must happen first?
- **ADRs**: What decisions need documenting?

### 2. Create/Update Plans

#### For New Plans

1. Create `plans/` subdirectory if needed
2. Write to appropriate file:
   - **Implementation plans** → Individual plan files in plans/
   - **Status updates** → plans/PROJECT_STATUS.md
   - **Technical research** → plans/TECHNICAL_ANALYSIS.md
   - **Decisions** → plans/ADRs/###-title.md

#### ADR Format

```markdown
# ADR-XXX: Title

## Status
[Proposed | Accepted | Deprecated | Replaced]

## Context
What is the issue motivating this decision?

## Decision
What is the change being proposed/decided?

## Consequences
What becomes easier or more difficult because of this choice?
```

### 3. Execute and Document

1. Perform planned actions
2. Update PROJECT_STATUS.md with results
3. Add technical learnings to TECHNICAL_ANALYSIS.md

## Key Files

### PROJECT_STATUS.md

Track:

- Current phase/goals
- Recent completions
- Blockers and issues
- Test results

### TECHNICAL_ANALYSIS.md

Document:

- Research findings
- Tradeoff analysis
- Performance observations
- Code patterns discovered

### ADR Naming

- Format: `###-title-slug.md` (e.g., `001-disable-shortcuts-when-tools-active.md`)
- Number sequentially
- Use descriptive slugs

## Integration Patterns

### With rust-engineer

- **Handoff**: goap-adr-planner creates plan → rust-engineer implements
- **Output**: Implementation plan with ADRs

### With dogfood

- **Handoff**: goap-adr-planner plans testing → dogfood executes QA
- **Output**: Test plan with success criteria

### With agents-md

- **Handoff**: goap-adr-planner documents decision → agents-md formats
- **Output**: Polished ADR in plans/ADRs/
