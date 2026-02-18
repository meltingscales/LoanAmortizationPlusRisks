# Loan Amortization Calculator - Rust Edition
# Config: Edit config.toml to customize loan parameters and scenarios

default:
    @just --list

# Build and run (debug mode - faster compile)
run:
    cargo run --bin loan-calc

# Build and run (release mode - optimized)
run-release:
    cargo run --bin loan-calc --release

# Run interactive GUI with sliders
run-gui:
    cargo run --bin loan-calc-gui --features gui

# Run GUI (release mode)
run-gui-release:
    cargo run --bin loan-calc-gui --features gui --release

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
    rm -f *.png loan_*.csv output_summary.txt loan_data.json

# Open the generated chart and CSV files
show:
    xdg-open loan_comparison.png
    @ls -1 loan_*.csv 2>/dev/null | head -1 | xargs -r xdg-open

# Install cargo-edit for dependency management
install-tools:
    cargo install cargo-edit
