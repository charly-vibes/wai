# Plugin System

Wai auto-detects and integrates with external tools through its plugin system.

## Detected plugins

| Plugin | Detection | Description |
|--------|-----------|-------------|
| **beads** | `.beads/` directory | Issue tracking |
| **git** | `.git/` directory | Version control |
| **openspec** | `openspec/` directory | Specification management |

## How it works

Plugins are detected automatically based on the presence of their marker directories. No configuration is needed — wai finds them and integrates their status into commands like `wai status`.

## Listing plugins

```bash
wai plugin list
```

This shows all detected plugins and their current state.
