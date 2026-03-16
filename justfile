# hpc-core — development task runner
# Install: cargo install just
# Usage:  just <recipe>

default:
    @just --list

# Type-check the workspace
check:
    cargo check --workspace

# Format all code
fmt:
    cargo fmt --all

# Check formatting (CI mode)
fmt-check:
    cargo fmt --all -- --check

# Run clippy lints (deny warnings)
lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run tests
test:
    #!/usr/bin/env bash
    set -euo pipefail
    if command -v cargo-nextest &>/dev/null; then
        cargo nextest run --workspace --all-features
    else
        cargo test --workspace --all-features
    fi

# Run the full test suite including slow tests
test-all:
    #!/usr/bin/env bash
    set -euo pipefail
    if command -v cargo-nextest &>/dev/null; then
        cargo nextest run --workspace --all-features --run-ignored all
    else
        cargo test --workspace --all-features -- --include-ignored
    fi

# Run only the slow (ignored) tests
test-slow:
    #!/usr/bin/env bash
    set -euo pipefail
    if command -v cargo-nextest &>/dev/null; then
        cargo nextest run --workspace --all-features --run-ignored ignored-only
    else
        cargo test --workspace --all-features -- --ignored
    fi

# Run cargo-deny checks
deny:
    cargo deny check

# Run advisory audit only
audit:
    cargo deny check advisories

# Run the full CI suite locally (fast tests)
all: fmt-check lint test deny

# Run the full CI suite locally (all tests)
all-full: fmt-check lint test-all deny
