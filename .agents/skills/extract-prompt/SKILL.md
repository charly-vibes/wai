---
name: extract-prompt
description: Analyze a conversation and extract a reusable prompt that captures the successful pattern. Generalizes specific instructions into templates.
---

# Extract Reusable Prompt from Conversation

Analyze this conversation and extract a reusable prompt that captures the successful pattern.

## Process

### Step 1: Analyze the Conversation

**What was the goal?** What were we trying to accomplish?
**What made it successful?** What specific instructions led to good results?
**What was the key pattern?** Can this be generalized?

### Step 2: Extract the Pattern

**The problem it solves:** [General class of problems]
**The approach:** [The method or structure that worked]
**Critical elements:** [Key instructions that made it work]

### Step 3: Generalize the Instructions

Convert specific instructions into general ones:
- **From specific:** "Read src/auth/oauth.ts and explain how it works"
- **To general:** "Read [FILE] and explain how it works"

### Step 4: Structure the Prompt

Create a structured prompt document with:
- Applicability section (when to use, when NOT to use)
- The actual prompt text (generalized)
- Usage scenario with example
- Success criteria

### Step 5: Determine Filename

**For workflows (multi-step):** `prompt-workflow-[descriptive-slug].md`
**For tasks (single focused task):** `prompt-task-[descriptive-slug].md`

### Step 6: Present Draft

```
I've extracted a reusable prompt from our conversation.

**Pattern identified:** [Name of pattern]
**Key insight:** [What made this work]
**Proposed filename:** [path]

[Show the structured prompt document]

Shall I save this prompt?
```

## Guidelines

1. **Generalize without losing specificity** - Keep concrete examples but make instructions general
2. **Capture the "why"** - Don't just transcribe, explain what made it work
3. **Include anti-patterns** - Document when NOT to use it
4. **Start with "draft" status** - Needs testing before "verified"
