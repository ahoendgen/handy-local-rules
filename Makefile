# Makefile for handy-local-rules
# Common development commands

.PHONY: all setup build release check fmt lint test clean run dev dev-tmux help \
        install uninstall reinstall service-start service-stop service-status service-logs

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

# Development mode in tmux session (with debug logging)
dev-tmux:
	tmux new-session -d -s handy 'cargo watch -x "run -- serve --log-level debug"' 2>/dev/null || true
	tmux attach -t handy

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

# --- macOS launchd service ---

PLIST_NAME = dev.a9g.handy-local-rules
PLIST_SRC = $(CURDIR)/$(PLIST_NAME).plist
PLIST_DST = $(HOME)/Library/LaunchAgents/$(PLIST_NAME).plist
BINARY_PATH = $(CURDIR)/target/release/handy-rules
CONFIG_DIR = $(HOME)/.handy-local-rules

# Install as launchd service (macOS)
install: release
	@echo "Installing handy-local-rules as launchd service..."
	@mkdir -p $(CONFIG_DIR)
	@mkdir -p $(HOME)/Library/LaunchAgents
	@sed -e 's|__BINARY_PATH__|$(BINARY_PATH)|g' \
	     -e 's|__CONFIG_DIR__|$(CONFIG_DIR)|g' \
	     $(PLIST_SRC) > $(PLIST_DST)
	@launchctl load $(PLIST_DST)
	@echo "Installing CLI to /usr/local/bin/handy-rules..."
	@sudo ln -sf $(BINARY_PATH) /usr/local/bin/handy-rules
	@echo "Service installed and started!"
	@echo "  Binary: $(BINARY_PATH)"
	@echo "  CLI:    /usr/local/bin/handy-rules"
	@echo "  Config: $(CONFIG_DIR)"
	@echo "  Logs:   $(CONFIG_DIR)/handy-local-rules.log"

# Uninstall launchd service
uninstall:
	@echo "Uninstalling handy-local-rules service..."
	@launchctl unload $(PLIST_DST) 2>/dev/null || true
	@rm -f $(PLIST_DST)
	@sudo rm -f /usr/local/bin/handy-rules 2>/dev/null || true
	@echo "Service and CLI uninstalled!"

# Reinstall launchd service (rebuild + restart)
reinstall: uninstall install

# Start launchd service
service-start:
	@launchctl load $(PLIST_DST)
	@echo "Service started!"

# Stop launchd service
service-stop:
	@launchctl unload $(PLIST_DST)
	@echo "Service stopped!"

# Show service status
service-status:
	@launchctl list | grep $(PLIST_NAME) || echo "Service not running"

# Tail service logs
service-logs:
	@tail -f $(CONFIG_DIR)/handy-local-rules.log

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
	@echo "  dev-tmux     - Dev mode in tmux session (Claude accessible)"
	@echo "  watch        - Watch and run checks on changes"
	@echo "  docs         - Generate documentation"
	@echo "  audit        - Security audit"
	@echo "  pre-commit   - Run pre-commit checks"
	@echo ""
	@echo "macOS Service:"
	@echo "  install        - Install as launchd service (auto-start)"
	@echo "  uninstall      - Remove launchd service"
	@echo "  reinstall      - Rebuild and restart service"
	@echo "  service-start  - Start service"
	@echo "  service-stop   - Stop service"
	@echo "  service-status - Show service status"
	@echo "  service-logs   - Tail service logs"
