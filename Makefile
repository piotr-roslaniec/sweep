.PHONY: help build test format lint clean release install check-all

# Default target
help:
	@echo "Available targets:"
	@echo "  make build       - Build debug binary"
	@echo "  make release     - Build release binary"
	@echo "  make test        - Run all tests"
	@echo "  make format      - Format code with rustfmt"
	@echo "  make lint        - Run clippy linter"
	@echo "  make clean       - Clean build artifacts"
	@echo "  make install     - Install using install.sh script"
	@echo "  make uninstall   - Remove installation using uninstall.sh"
	@echo "  make check-all   - Run format check, lint, and tests (CI)"

# Build debug binary
build:
	cargo build

# Build release binary
release:
	cargo build --release

# Run all tests
test:
	cargo test

# Format code
format:
	cargo fmt

# Run linter
lint:
	cargo clippy --all-targets --all-features -- -D warnings

# Clean build artifacts
clean:
	cargo clean

# Install using our install script
install:
	./install.sh

# Uninstall using our uninstall script
uninstall:
	./uninstall.sh

# Run all checks (for CI)
check-all:
	cargo fmt -- --check
	cargo clippy --all-targets --all-features -- -D warnings
	cargo test