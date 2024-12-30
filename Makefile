.PHONY: all build test clean check fmt lint doc run release

# Default target
all: check test build

# Build the project in debug mode
build:
	cargo build

# Build for release
release:
	cargo build --release

# Run the project
run:
	cargo run

# Run tests
test:
	cargo test
	cargo test --doc

# Clean build artifacts
clean:
	cargo clean
	rm -rf target/

# Format code
fmt:
	cargo fmt
	cargo fix --allow-dirty --allow-staged

# Run clippy lints
lint:
	cargo clippy -- -D warnings

# Check if the project compiles
check:
	cargo check

# Generate documentation
doc:
	cargo doc --no-deps
	cargo doc --open

# Watch for changes and run tests
watch-test:
	cargo watch -x test

# Watch for changes and run the project
watch-run:
	cargo watch -x run

# Update dependencies
update:
	cargo update

# Run security audit
audit:
	cargo audit

# Check formatting
fmt-check:
	cargo fmt -- --check

# Build and run with release optimizations
run-release: release
	cargo run --release

# Show dependency tree
deps:
	cargo tree

cov:
	cargo tarpaulin --engine llvm --out Html
	open tarpaulin-report.html

pre-commit: fmt fmt-check lint test
