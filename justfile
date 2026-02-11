# wai justfile - unified local/CI workflow
#
# Same commands run locally and in CI for consistent diagnostics.
# Run `just` for default (build + test), `just ci` for full pipeline.

set shell := ["bash", "-uc"]

# Default: build and test
default: build test

# === Build Commands ===

# Build debug binary
build:
    cargo build

# Build release binary (optimized)
build-release:
    cargo build --release

# Run with arguments (e.g., `just run status`, `just run init`)
run *args:
    cargo run -- {{args}}

# Install locally to ~/.cargo/bin
install:
    cargo install --path .

# === Test Commands ===

# Run all tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Run a specific test
test-one name:
    cargo test {{name}} -- --nocapture

# === Lint Commands ===

# Run clippy linter
lint:
    cargo clippy -- -D warnings

# Format all Rust files
fmt:
    cargo fmt

# Check formatting (no changes)
fmt-check:
    cargo fmt -- --check

# === CI Commands ===

# Full CI pipeline
ci: fmt-check lint test build-release
    @echo "✅ CI pipeline passed"

# Pre-push checks (fast gate)
pre-push: fmt-check lint test
    @echo "✅ Pre-push checks passed"

# === Setup Commands ===

# Setup development environment
setup:
    @echo "Checking Rust installation..."
    rustc --version
    cargo --version
    @echo ""
    @echo "Installing dev tools..."
    rustup component add clippy rustfmt
    @echo ""
    @echo "Installing lefthook..."
    @command -v lefthook >/dev/null 2>&1 || cargo install lefthook
    lefthook install
    @echo ""
    @echo "✅ Development environment ready"
    @echo "Run 'just test' to verify setup"

# === Docs Commands ===

# Build docs locally (requires mdbook)
docs:
    mdbook build docs

# Live preview docs at localhost:3000 (requires mdbook)
docs-serve:
    mdbook serve docs

# === Utility Commands ===

# Clean build artifacts
clean:
    cargo clean

# Check without building (faster feedback)
check:
    cargo check

# Show dependency tree
deps:
    cargo tree

# Update dependencies
update:
    cargo update

# Show available commands
help:
    @just --list
