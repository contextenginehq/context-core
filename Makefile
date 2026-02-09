.PHONY: build test clean release check

build:
	cargo build

test:
	cargo test

check:
	cargo check
	cargo clippy -- -D warnings

clean:
	cargo clean

release:
	cargo build --release
