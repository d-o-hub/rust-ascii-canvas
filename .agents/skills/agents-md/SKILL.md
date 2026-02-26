---
name: agents-md
description: Agent documentation specialist - creates and maintains AGENTS.md files, documentation structure, and best practices guides
---

# Agents MD

## Purpose

Specializes in creating and maintaining agent documentation, best practices guides, and documentation structure for multi-agent systems.

## When to Use

- Creating new AGENTS.md files from scratch
- Updating existing agent documentation
- Organizing agent skills and workflows
- Creating reference documentation for agent teams
- Documenting task execution patterns and learnings

## Don't Invoke When

- Writing code (use rust-engineer or other coding skills)
- Testing applications (use agent-browser or dogfood)
- Database work (use database-optimizer)

## Core Capabilities

### Documentation Creation
- Creating AGENTS.md with best practices
- Organizing skill-based agent documentation
- Setting up agents-docs/ folder structure
- Writing SKILL.md templates for new skills

### Documentation Maintenance
- Updating existing documentation with new learnings
- Keeping PROJECT_STATUS.md current
- Maintaining TECHNICAL_ANALYSIS.md with findings
- Organizing ADRs in plans/ folder

### Best Practices Implementation
- Following 120 LOC max for AGENTS.md
- Creating clear agent invocation patterns
- Documenting escalation triggers
- Setting up integration patterns between skills

## Quick Start

### Create New AGENTS.md
1. Define available agents and their purposes
2. Document best practices for task execution
3. Include file organization structure
4. Add testing workflow guidelines

### Update Documentation After Tasks
1. Run tests and capture results
2. Update PROJECT_STATUS.md with test outcomes
3. Add technical findings to TECHNICAL_ANALYSIS.md
4. Create new ADRs for significant decisions

## Documentation Template

```markdown
# Agent Best Practices - <Project Name>

## Agent System Overview

<Description of agent system>

## Available Agents

### <Agent Name>
- **Purpose**: <What it does>
- **Location**: <File path>
- **Trigger**: <When to use>

## Best Practices

### Task Execution
1. Analyze - Understand requirements
2. Plan - Create steps
3. Execute - Run commands
4. Document - Update plans/

## File Organization
```

## Best Practices

1. Keep AGENTS.md under 120 lines
2. Always document in plans/ folder for architectural decisions
3. Update documentation after each successful task
4. Use consistent formatting across all docs
5. Link skills to their SKILL.md files
