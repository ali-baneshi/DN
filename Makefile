SHELL := /bin/bash

.PHONY: help setup fmt lint test build run clean docker-up docker-down check health scan scan-json

help:
	@echo "Targets:"
	@echo "  make setup       - create python venv and install worker deps"
	@echo "  make fmt         - format Rust code"
	@echo "  make lint        - run clippy"
	@echo "  make test        - run tests"
	@echo "  make build       - build workspace"
	@echo "  make run         - run dn-cli help"
	@echo "  make health      - run healthcheck"
	@echo "  make scan        - scan current repo"
	@echo "  make scan-json   - scan current repo as json"
	@echo "  make clean       - cargo clean"
	@echo "  make docker-up   - start docker compose"
	@echo "  make docker-down - stop docker compose"
	@echo "  make check       - fmt + lint + test"

setup:
	@python -m venv .venv
	@.venv/bin/python -m pip install --upgrade pip
	@if [ -f workers/python/requirements.txt ]; then .venv/bin/python -m pip install -r workers/python/requirements.txt; else echo "no python requirements"; fi

fmt:
	cargo fmt --all

lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
	cargo test --workspace

build:
	cargo build --workspace

run:
	cargo run -p dn-cli -- --help

health:
	./scripts/healthcheck.sh

scan:
	cargo run -p dn-cli -- scan .

scan-json:
	cargo run -p dn-cli -- scan . --json

clean:
	cargo clean

docker-up:
	docker compose up --build

docker-down:
	docker compose down

check: fmt lint test
