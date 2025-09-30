SHELL := /bin/bash
BIN := cleanshare

.PHONY: all build test fmt lint clean install run docker-build docker-run

all: build

build:
	cargo build --release

test:
	cargo test

fmt:
	cargo fmt --all

lint:
	cargo clippy --all-targets --all-features -- -D warnings

clean:
	cargo clean

install:
	cargo install --path .

run:
	cargo run -- -u "https://example.com/?utm_source=x&gclid=1"

docker-build:
	docker build -t $(BIN):latest .

docker-run:
	docker run --rm -i $(BIN):latest --help

