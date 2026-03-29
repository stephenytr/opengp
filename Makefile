.PHONY: test test-ui test-domain test-infra test-config build run dev fmt lint check clean watch help

# Run all tests
test:
	cargo test

# Run UI crate tests
test-ui:
	cargo test -p opengp-ui

# Run domain crate tests
test-domain:
	cargo test -p opengp-domain

# Run infrastructure crate tests
test-infra:
	cargo test -p opengp-infrastructure

# Run config crate tests
test-config:
	cargo test -p opengp-config

# Build release binary
build:
	cargo build --release

# Run release binary
run:
	cargo run --release

# Run debug binary
dev:
	cargo run

# Format code
fmt:
	cargo fmt

# Run clippy linter with warnings as errors
lint:
	cargo clippy -- -D warnings

# Run format check, clippy, and tests
check:
	cargo fmt -- --check && cargo clippy -- -D warnings && cargo test

# Clean build artifacts
clean:
	cargo clean

# Watch for changes and run tests
watch:
	cargo watch -x test

# Display this help message
help:
	@echo "OpenGP Makefile targets:"
	@echo ""
	@echo "  test              Run all tests"
	@echo "  test-ui           Run UI crate tests"
	@echo "  test-domain       Run domain crate tests"
	@echo "  test-infra        Run infrastructure crate tests"
	@echo "  test-config       Run config crate tests"
	@echo "  build             Build release binary"
	@echo "  run               Run release binary"
	@echo "  dev               Run debug binary"
	@echo "  fmt               Format code with rustfmt"
	@echo "  lint              Run clippy linter with warnings as errors"
	@echo "  check             Run format check, clippy, and tests"
	@echo "  clean             Clean build artifacts"
	@echo "  watch             Watch for changes and run tests (requires cargo-watch)"
	@echo "  help              Display this help message"
