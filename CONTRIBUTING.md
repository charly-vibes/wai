# Contributing to wai

Thank you for your interest in contributing to wai!

## Development Setup

Requires [Rust](https://www.rust-lang.org/tools/install) (stable toolchain):

```bash
# Clone and build
git clone https://github.com/charly-vibes/wai
cd wai
cargo build

# Install git hooks
prek install

# Run tests
just test
```

## Common Tasks

```bash
just build    # Build the project
just test     # Run tests
just lint     # Run clippy
just fmt      # Format code
just check    # Run all checks (fmt + lint + test)
```

## Making Changes

1. Check open issues via `bd ready` (wai uses beads for issue tracking)
2. Create a branch for your change
3. Write code and tests
4. Ensure `just check` passes before committing
5. Open a pull request

## Code Style

- Follow standard Rust idioms (`cargo clippy` enforces this)
- Format with `cargo fmt`
- Write tests for new behaviour

## Reporting Issues

Open an issue on [GitHub](https://github.com/charly-vibes/wai/issues).

## See Also

- **[AGENTS.md](./AGENTS.md)** — AI agent workflow instructions, including the tool-work vs repo-maintenance distinction, TDD conventions, and session protocols.
- **[OpenSpec](./openspec/AGENTS.md)** — How to propose and apply system changes via specifications.
- **[Development docs](./docs/src/development.md)** — Architecture overview and code organization.
