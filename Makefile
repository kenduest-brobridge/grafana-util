.PHONY: help build build-python build-rust test test-python test-rust

PYTHON ?= python3
PIP ?= $(PYTHON) -m pip
CARGO ?= cargo
RUST_DIR ?= rust
PYTHON_DIST_DIR ?= dist

help:
	@printf '%s\n' \
		'Available targets:' \
		'  make build         Build both Python and Rust artifacts' \
		'  make build-python  Build the Python wheel into dist/' \
		'  make build-rust    Build Rust release binaries in rust/target/release/' \
		'  make test          Run both Python and Rust test suites' \
		'  make test-python   Run the Python unittest suite' \
		'  make test-rust     Run the Rust cargo test suite'

build: build-python build-rust

build-python:
	$(PIP) wheel --no-deps --no-build-isolation --wheel-dir $(PYTHON_DIST_DIR) .

build-rust:
	cd $(RUST_DIR) && $(CARGO) build --release

test: test-python test-rust

test-python:
	$(PYTHON) -m unittest -v

test-rust:
	cd $(RUST_DIR) && $(CARGO) test
