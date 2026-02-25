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

This creates a `.wai/` directory to store your artifacts and configuration.

## 3. Create a Project

In `wai`, a "Project" is an active work item with a specific goal. Create your first one:

```bash
wai new project my-feature
```

## 4. Capture Research

Projects start in the **Research** phase. Record your initial thoughts or findings:

```bash
wai add research "Evaluated options A and B. Choosing A for performance."
```

## 5. Advance Through Phases

As your work progresses, advance the project's phase:

```bash
# Advance: Research → Design
wai phase next

# Add a design artifact
wai add design "New API will use standard REST patterns."
```

## 6. Check Your Progress

Get a context-aware overview of your workspace:

```bash
wai status
```

Status shows your active projects, their current phases, and **smart suggestions** for what to do next.

---

## Next Steps

Now that you've got the basics down, explore the more powerful features of `wai`:

- **[Commands Reference](./commands.md)** — Comprehensive list of all commands and flags.
- **[PARA Method](./concepts/para-method.md)** — Learn how `wai` organizes your work.
- **[Project Phases](./concepts/phases.md)** — Master the workflow from Research to Archive.
- **[Agent Config Sync](./concepts/agent-config-sync.md)** — Keep your AI tools in sync.
- **[Diagnostics & Health](./commands.md#diagnostics)** — Learn the difference between `wai doctor` and `wai way`.

---

## Troubleshooting

If you run into issues, check the **[Troubleshooting](./troubleshooting.md)** guide or run:

```bash
wai doctor  # Check workspace health
```
