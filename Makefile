.DEFAULT_GOAL := all

.PHONY: install-rust-coverage
install-rust-coverage:
	cargo install rustfilt cargo-binutils
	rustup component add llvm-tools-preview

.PHONY: build-prod
build-prod:
	cargo build --release

.PHONY: format
format:
	cargo fmt

.PHONY: lint
lint:
	cargo fmt --version
	cargo fmt --all -- --check
	cargo clippy --version
	cargo clippy -- -D warnings -A incomplete_features

.PHONY: test
test:
	cargo test

.PHONY: bench
bench:
	cargo bench

.PHONY: testcov
testcov:
	RUSTFLAGS='-C instrument-coverage -A incomplete_features -C link-arg=-undefined -C link-arg=dynamic_lookup' cargo test
	./tests/rust_coverage_html.sh

.PHONY: all
all: format lint test
