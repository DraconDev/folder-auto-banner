# Justfile for cfm development

# Default recipe
default: help

# Build the project
build:
    cargo build

# Build release
build-release:
    cargo build --release

# Run with debug output
dev:
    cargo run --

# Run with args
run args:
    cargo run -- {{args}}

# Run tests
test:
    cargo test

# Run clippy
lint:
    cargo clippy -- -W clippy::all -W clippy::pedantic

# Check formatting
fmt:
    cargo fmt --check

# Format code
format:
    cargo fmt

# Clean build artifacts
clean:
    cargo clean

# Run with banner command
banner:
    cargo run -- banner

# Run with env command
env:
    cargo run -- env

# Install (requires cargo install)
install:
    cargo install --path .

# Uninstall (requires manual removal)
uninstall:
    @echo "Run: rm ~/.cargo/bin/fm"

# Show help
help:
    @echo "cfm - Contextual File Manager"
    @echo ""
    @echo "Available recipes:"
    @echo "  build         - Build the project (debug)"
    @echo "  build-release - Build the project (release)"
    @echo "  dev           - Run with default banner"
    @echo "  run <args>    - Run with custom args"
    @echo "  test          - Run tests"
    @echo "  lint          - Run clippy lints"
    @echo "  fmt           - Check code formatting"
    @echo "  format        - Format code"
    @echo "  clean         - Clean build artifacts"
    @echo "  banner        - Run banner command"
    @echo "  env           - Run env command"
    @echo "  install       - Install to ~/.cargo/bin"