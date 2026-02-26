# Agent Skills - Reference Documentation

This folder contains reference documentation for all agent skills available in the project.

## Available Skills

### rust-engineer
- **Location**: `.agents/skills/rust-engineer/SKILL.md`
- **Purpose**: Rust specialist with expertise in async programming, ownership patterns, FFI, and WebAssembly development
- **Use when**: Building high-performance backend services, systems programming, WASM development

### rust-best-practices
- **Location**: `.agents/skills/rust-best-practices/SKILL.md`
- **Purpose**: Guide for writing idiomatic Rust code based on best practices
- **Use when**: Writing new Rust code, reviewing/refactoring, deciding ownership patterns

### agent-browser
- **Location**: `.agents/skills/agent-browser/SKILL.md`
- **Purpose**: Browser automation CLI for AI agents
- **Use when**: Navigating pages, filling forms, clicking buttons, taking screenshots, testing web apps

### dogfood
- **Location**: `.agents/skills/dogfood/SKILL.md`
- **Purpose**: Systematically explore and test a web application to find bugs and UX issues
- **Use when**: QA testing, exploratory testing, bug hunting, finding issues

### skill-creator
- **Location**: `.agents/skills/skill-creator/SKILL.md`
- **Purpose**: Guide for creating effective skills that extend agent capabilities
- **Use when**: Creating new skills or updating existing ones

### goap-adr-planner
- **Location**: `.agents/skills/goap-adr-planner/SKILL.md`
- **Purpose**: Goal-Oriented Action Planning with Architectural Decision Records
- **Use when**: Planning multi-step tasks, creating plans in plans/ folder, documenting decisions with ADRs

## Agent Configuration

### goap-adr-analyzer
- **Location**: `.opencode/agent/goap-adr-analyzer.md`
- **Purpose**: Analyze GOAP implementations and manage ADRs in plans/ folder
- **Use when**: Multi-step tasks requiring architectural decisions

## Best Practices Summary

1. Always use appropriate skill for the task
2. Document architectural decisions in plans/ folder
3. Run tests before marking tasks complete
4. Keep code under 500 LOC per file
5. Update PROJECT_STATUS.md and TECHNICAL_ANALYSIS.md with findings
