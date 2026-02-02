# Makefile for handy-local-rules
# Common development commands

.PHONY: all setup build release check fmt lint test clean run dev help

# Default target
all: check

# Initial setup (run once after cloning)
setup:
	@echo "Installing Node.js dependencies for git hooks..."
	pnpm install
	@echo "Setting up Rust toolchain..."
	rustup component add rustfmt clippy
	@echo "Setup complete!"

# Build debug version
build:
	cargo build

# Build release version
release:
	cargo build --release

# Run all checks (format check + clippy + tests)
check: fmt-check lint test

# Format code (Rust + Markdown)
fmt:
	cargo fmt
	pnpm prettier --write "**/*.md"

# Check formatting without modifying
fmt-check:
	cargo fmt -- --check
	pnpm prettier --check "**/*.md"

# Run clippy linter
lint:
	cargo clippy -- -D warnings

# Run clippy with all features
lint-all:
	cargo clippy --all-features -- -D warnings

# Run clippy with pedantic lints (stricter)
lint-pedantic:
	cargo clippy -- -D warnings -W clippy::pedantic -W clippy::nursery

# Run tests
test:
	cargo test

# Run tests with all features
test-all:
	cargo test --all-features

# Clean build artifacts
clean:
	cargo clean

# Run the server (debug)
run:
	cargo run -- serve

# Run the server (release)
run-release:
	cargo run --release -- serve

# Development mode with hot-reload
dev:
	cargo watch -x 'run -- serve'

# Watch for changes and run checks
watch:
	cargo watch -x "clippy -- -D warnings" -x test

# Generate documentation
docs:
	cargo doc --no-deps --open

# Security audit
audit:
	cargo audit

# Update dependencies
update:
	cargo update

# Show outdated dependencies
outdated:
	cargo outdated

# Pre-commit hook (run before committing)
pre-commit: fmt-check lint test
	@echo "All checks passed!"

# Help
help:
	@echo "Available targets:"
	@echo "  setup        - Initial setup (install git hooks)"
	@echo "  build        - Build debug version"
	@echo "  release      - Build release version"
	@echo "  check        - Run all checks (fmt, lint, test)"
	@echo "  fmt          - Format code"
	@echo "  fmt-check    - Check formatting"
	@echo "  lint         - Run clippy linter"
	@echo "  lint-pedantic- Run clippy with pedantic lints"
	@echo "  test         - Run tests"
	@echo "  clean        - Clean build artifacts"
	@echo "  run          - Run server (debug)"
	@echo "  run-release  - Run server (release)"
	@echo "  dev          - Development mode with hot-reload"
	@echo "  watch        - Watch and run checks on changes"
	@echo "  docs         - Generate documentation"
	@echo "  audit        - Security audit"
	@echo "  pre-commit   - Run pre-commit checks"
