set shell := ["bash", "-cu"]

default:
    @just --list

fmt:
    cargo fmt --all

clippy:
    cargo clippy --workspace --all-targets -- -D warnings

build:
    cargo build --workspace --release

paper:
    cargo run --bin toon -- run --mode paper --config config/default.toml --markets config/markets.toml

replay path:
    cargo run --bin toon -- replay --parquet {{path}} --config config/default.toml

pgo-collect:
    RUSTFLAGS="-Cprofile-generate=pgo-data" cargo run --bin toon -- run --mode paper

pgo-build:
    RUSTFLAGS="-Cprofile-use=pgo-data" cargo build --workspace --release
