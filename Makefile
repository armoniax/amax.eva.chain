.PHONY: setup fmt-check fmt clippy dev clean check build test-build release test-release test ci-clippy

setup:
	bash ./scripts/setup.sh

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

test-build: fmt
	cargo build --no-default-features --features manual-seal

release: fmt
	cargo build --release

test-release: fmt
	cargo build --release --no-default-features --features manual-seal

test: fmt
	cargo test --all

ci-clippy:
	cargo +nightly clippy --all --all-targets --features runtime-benchmarks,try-runtime -- -D warnings