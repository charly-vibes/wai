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


# TDD loop (most obvious entry, DDL test-runner standard — see DDL-85p in
# the dulce-de-leche repo): testaruda exec — select the tests the working
# diff affects, run only those, ingest, calibrate. Tolerance contract:
# exit 0/10/20 green; NOT CALIBRATED with 0 actual failures is a vacuous
# pass (tambor-cpd policy), with failures it blocks. Local store
# (.testaruda/, gitignored) initializes on first use; a stale store
# self-heals (wild-z56).
test:
    #!/usr/bin/env bash
    set -uo pipefail
    if [ ! -d .testaruda ]; then
      testaruda init || { echo "❌ testaruda init failed" >&2; exit 1; }
    fi
    code=0
    testaruda exec || code=$?
    case "$code" in
      0|10|20) exit 0 ;;
      4)
        echo "⚠️ runner exit 4 — pruning stale local calibration store and retrying once (wild-z56)" >&2
        rm -rf .testaruda
        testaruda init || { echo "❌ testaruda re-init failed" >&2; exit 1; }
        testaruda exec ;;
      *) exit "$code" ;;
    esac

# Blast-radius preview: which tests a change WOULD affect — runs nothing.
# Useful before a refactor; optionally name files: just test-blast src/foo.rs
test-blast *files:
    #!/usr/bin/env bash
    if [ -z "{{files}}" ]; then
      testaruda select --pre-edit --human
    else
      testaruda select --pre-edit --human --files "{{files}}"
    fi

test-full:
    cargo test

# Validate spec-test correspondence (requires espectacular)
validate:
    ah check

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Run a specific test
test-one name:
    cargo test {{name}} -- --nocapture

# === Lint Commands ===

# Run clippy linter (checks all targets, including tests)
lint:
    cargo clippy --all-targets -- -D warnings

# Format all Rust files
fmt:
    cargo fmt

# Check formatting (no changes)
fmt-check:
    cargo fmt -- --check

# === CI Commands ===

# Full CI pipeline
ci: fmt-check lint test build-release
    pretender check --diff-only
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

# Regenerate llms.txt and llm.txt from aix module metadata
aix-gen:
    cargo run --example gen-aix

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
