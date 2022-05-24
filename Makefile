.PHONY: setup check build test fmt-check fmt lint clean

setup:
	bash ./scripts/setup/dev_setup.sh

fmt-check:
	taplo fmt --check
	cargo fmt --all -- --check

fmt:
	taplo fmt
	cargo fmt --all

clippy:
	cargo +nightly clippy --all --all-targets -- -D warnings

dev: fmt clippy

clean:
	cargo clean

check:
	cargo check

build: fmt
	cargo build

release: fmt
	cargo build --release

test: fmt
	cargo test --all

test-build: fmt
	cargo build --release --no-default-features --features manual-seal,evm-debug,evm-tracing

tracing-build: fmt
	cargo build --release --features evm-tracing
