set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# Build outside `[R] Rust` (jemalloc's configure rejects spaces); persistent, portable.
export CARGO_TARGET_DIR := env("HOME") / ".cache/cargo-target/lele-bevy-state-machine"

# Toolchain: rust-toolchain.toml. Extra tools: cargo-nextest, just, and
# `lele-lint` / `lele-bevy-lint`.

build:
    cargo build --all-targets --all-features

clippy:
    cargo clippy --all-targets --all-features -- -D warnings

fmt:
    cargo fmt -- --check

test:
    cargo nextest run --all-targets --all-features

lint:
    lele-lint .

bevy-lint:
    lele-bevy-lint .

sync-methods:
    lele-lint --sync-methods .

setup:
    prek install -t pre-commit -t pre-push

check:
    prek run --all-files

ci: build check
