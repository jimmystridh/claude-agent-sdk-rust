# Claude Agents SDK Makefile
#
# Quick reference:
#   make test              - Run unit tests
#   make integration-test  - Run integration tests (requires auth)
#   make build             - Build the library
#   make check             - Run fmt check + clippy

.PHONY: build test integration-test check fmt clippy doc clean help

# Default target
help:
	@echo "Claude Agents SDK - Available commands:"
	@echo ""
	@echo "  make build              Build the library"
	@echo "  make test               Run unit tests (no auth required)"
	@echo "  make integration-test   Run integration tests in Docker"
	@echo "  make check              Run fmt check + clippy"
	@echo "  make fmt                Format code"
	@echo "  make clippy             Run linter"
	@echo "  make doc                Build documentation"
	@echo "  make clean              Clean build artifacts"
	@echo ""
	@echo "Integration tests require authentication. Set up with:"
	@echo "  1. Run: claude setup-token"
	@echo "  2. Copy .env.example to .env and paste your token"
	@echo "  3. Run: make integration-test"

# Build
build:
	cargo build

# Unit tests (no auth required)
test:
	cargo test

# Integration tests in Docker
integration-test:
	@./scripts/run-integration-tests.sh

# Integration tests with verbose output
integration-test-verbose:
	@./scripts/run-integration-tests.sh --verbose

# Interactive shell for debugging
integration-shell:
	@./scripts/run-integration-tests.sh --shell

# Code quality
check: fmt-check clippy

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

clippy:
	cargo clippy -- -D warnings

# Documentation
doc:
	cargo doc --open

# Clean
clean:
	cargo clean
