# wai

Workflow manager for AI-driven development.

## Design Principles

- **Desire Path Alignment** — Pave the cowpaths, make common workflows shortest
- **Self-Healing Errors** — Errors suggest fixes, not just report problems  
- **Progressive Disclosure** — Simple by default, powerful when needed
- **Context-Aware** — Offer next steps based on current state

## Installation

```bash
cargo install --path .
```

## Quick Start

```bash
# Initialize in current directory
wai init

# Or create a new project
wai new project my-app

# Check status and get suggestions
wai status

# Create a work unit (bead)
wai new bead "Add user authentication"
```

## Project Structure

```
.para/
├── config.toml      # Project configuration
├── beads/           # Work units (features, fixes, etc.)
├── research/        # Research notes and findings
└── plugins/         # Installed plugins
```

## Development

```bash
# Build
cargo build

# Run
cargo run -- status

# Test
cargo test
```
