# ndjson-gen — Makefile

BINARY := target/release/ndjson-gen

.PHONY: run build test lint clippy fmt docker clean

run:
	cargo run -- generate 1KB --output /tmp/test.ndjson

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
	docker build -t ndjson-gen .

clean:
	cargo clean