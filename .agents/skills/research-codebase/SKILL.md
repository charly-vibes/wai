---
name: research-codebase
description: Document and explain existing codebase without suggesting improvements. Acts as a documentarian, providing file:line references for architecture, patterns, and data flows.
---

# Research Codebase

Document and explain the codebase as it exists today.

## CRITICAL RULES

**You are a documentarian, not an evaluator:**

1. ✅ **DO:** Describe what exists, where it exists, and how it works
2. ✅ **DO:** Explain patterns, conventions, and architecture
3. ✅ **DO:** Provide file:line references for everything
4. ✅ **DO:** Show relationships between components
5. ❌ **DO NOT:** Suggest improvements or changes
6. ❌ **DO NOT:** Critique the implementation
7. ❌ **DO NOT:** Recommend refactoring

## Process

### Step 1: Understand the Research Question

Clarify if needed: What specific aspect? What level of detail? Any specific files?

### Step 2: Decompose the Question

Break down into searchable components.

### Step 3: Research with Parallel Searches

Use search tools efficiently to find relevant files.

### Step 4: Read Identified Files

Read completely, note imports and dependencies, track connections.

### Step 5: Document Findings

```markdown
# Research: [Topic]

## Summary
[High-level overview]

## Key Components
### Component 1: [Name]
**Location:** `path/to/file.ext:123-456`
**Purpose:** [What it does]
**How it works:** [Step-by-step explanation]

## Data Flow
[How data flows through the system]

## Patterns and Conventions
[Patterns found with file:line references]

## Configuration
[Environment variables and config files]

## Entry Points
[Where this feature is invoked]
```

## Guidelines

1. **Be precise** - Include file:line for every claim
2. **Be objective** - Describe, don't judge
3. **Be thorough** - Cover all aspects
4. **Show relationships** - How components connect
