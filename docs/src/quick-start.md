# Quick Start

## Initialize

Set up wai in your project directory:

```bash
wai init
```

## Create a project

```bash
wai new project my-app
```

## Check status

Get an overview of your project with contextual suggestions:

```bash
wai status
```

## Manage phases

Projects progress through phases — research, design, plan, implement, review, archive:

```bash
wai phase              # Show current phase
wai phase next         # Advance to next phase
```

## Add artifacts

Capture research, plans, and designs as you work:

```bash
wai add research "Initial API analysis"
wai add plan "Implementation approach"
wai add design "Component architecture"
```

## Generate handoffs

Create a handoff document summarizing a project's state:

```bash
wai handoff create my-app
```

## Search

Find content across all artifacts:

```bash
wai search "authentication"
```

## View timeline

See a chronological view of project activity:

```bash
wai timeline my-app
```

## Sync agent configs

Push agent config files to their tool-specific locations:

```bash
wai sync
```
