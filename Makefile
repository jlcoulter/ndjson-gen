# Rust CLI — Makefile

BINARY := target/release/rust-cli-template

.PHONY: run build test lint clippy fmt docker clean

run:
	cargo run -- example

build:
	cargo build --release

test:
	cargo test --all

clippy:
	cargo clippy --all-targets -- -D warnings

fmt:
	cargo fmt --all -- --check

lint: fmt clippy

docker:
	docker build -t rust-cli-template .

clean:
	cargo clean