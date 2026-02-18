# Loan Amortization Calculator - Rust Edition
# Config: Edit config.toml to customize loan parameters and scenarios

default:
    @just --list

# Build and run (debug mode - faster compile)
run:
    cargo run

# Build and run (release mode - optimized)
run-release:
    cargo run --release

# Build release binary
build:
    cargo build --release

# Quick check without full build
check:
    cargo check

# Run tests
test:
    cargo test

# Format code
fmt:
    cargo fmt

# Lint with clippy
clippy:
    cargo clippy -- -D warnings

# Update dependencies
update:
    cargo update

# Clean build artifacts
clean:
    cargo clean
    rm -f *.png output_summary.txt loan_data.json

# Install cargo-edit for dependency management
install-tools:
    cargo install cargo-edit
