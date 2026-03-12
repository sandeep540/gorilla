.PHONY: build release clean test run check fmt lint

build:
	cargo build

release:
	cargo build --release

clean:
	cargo clean

test:
	cargo test

run:
	cargo run

check:
	cargo check

fmt:
	cargo fmt

lint:
	cargo clippy -- -D warnings

help:
	@echo "Available targets:"
	@echo "  build    - Build the project in debug mode"
	@echo "  release  - Build the project in release mode"
	@echo "  clean    - Remove build artifacts"
	@echo "  test     - Run tests"
	@echo "  run      - Run the project"
	@echo "  check    - Check the project without building"
	@echo "  fmt      - Format code"
	@echo "  lint     - Run clippy linter"