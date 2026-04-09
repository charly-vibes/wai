# Quick Start

Get up and running with `wai` in 5 minutes.

## 1. Run the Interactive Tutorial

The best way to learn is by doing. Run:

```bash
wai tutorial
```

This interactive guide covers the PARA method, project phases, and basic commands in a safe sandbox.

## 2. Initialize Your Workspace

Navigate to your project directory and initialize:

```bash
wai init
```

This creates a `.wai/` directory — the single source of truth for all reasoning and workflow state in this repository. Everything wai tracks lives here: artifacts, configs, plugin definitions, and project metadata. It's designed to be committed alongside your code.

## 3. Create a Project

In `wai`, a "Project" is an active work item with a specific goal. Create your first one:

```bash
wai new project my-feature
```

Projects use phases instead of jumping straight to code. This forces the research and design reasoning to exist *before* implementation, so when you or an agent revisits the project later, the *why* is already captured — not lost in a chat thread.

## 4. Capture Research

Projects start in the **Research** phase. Record your initial thoughts or findings:

```bash
wai add research "Evaluated options A and B. Choosing A for performance."
```

This creates a dated Markdown artifact in `.wai/projects/my-feature/research/`. The artifact persists across sessions — unlike chat history, it won't disappear when an agent's context resets.

## 5. Advance Through Phases

As your work progresses, advance the project's phase:

```bash
# Advance: Research → Design
wai phase next

# Add a design artifact
wai add design "New API will use standard REST patterns."
```

Phases (research → design → plan → implement → review → archive) guide what kind of work and artifacts are expected at each stage. They're flexible — skip forward or go back as needed.

## 6. Check Your Progress

Get a context-aware overview of your workspace:

```bash
wai status
```

Status is the context-aware next-step command. It shows active projects, their current phases, plugin context (git status, open issues), and **smart suggestions** for what to do next based on your current state. It's the first thing to run at the start of any session.

---

## Next Steps

Now that you've got the basics down, explore the more powerful features of `wai`:

- **[Commands Reference](./commands.md)** — Comprehensive list of all commands and flags.
- **[PARA Method](./concepts/para-method.md)** — Learn how `wai` organizes your work.
- **[Project Phases](./concepts/phases.md)** — Master the workflow from Research to Archive.
- **[Sessions](./concepts/sessions.md)** — The prime→close lifecycle for session continuity.
- **[Agent Config Sync](./concepts/agent-config-sync.md)** — Keep your AI tools in sync.

---

## Troubleshooting

If you run into issues, check the **[Troubleshooting](./troubleshooting.md)** guide or run:

```bash
wai doctor  # Check workspace health
```
