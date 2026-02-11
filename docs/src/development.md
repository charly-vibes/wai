# Development

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)
- [just](https://github.com/casey/just) (command runner)

## Building

```bash
just build           # Debug build
just build-release   # Release build
```

## Testing

```bash
just test            # Run all tests
just test-verbose    # Run tests with output
just test-one <name> # Run a specific test
```

## Linting & formatting

```bash
just lint            # Run clippy
just fmt             # Format code
just fmt-check       # Check formatting without changes
```

## Full CI pipeline

Run the same checks that CI runs:

```bash
just ci
```

## Documentation

Build and preview the docs locally:

```bash
just docs            # Build docs
just docs-serve      # Live preview at localhost:3000
```

Requires [mdBook](https://rust-lang.github.io/mdBook/guide/installation.html):

```bash
cargo install mdbook
```
