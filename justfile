set shell := ["bash", "-lc"]

export CARGO := "~/.cargo/bin/cargo"

default:
    @just --list

fmt:
    {{ CARGO }} fmt --all

check:
    {{ CARGO }} check --workspace

clippy:
    {{ CARGO }} clippy --workspace --all-targets

test:
    {{ CARGO }} test --workspace

package:
    {{ CARGO }} package -p ta-lib-codegen
    {{ CARGO }} package -p ta-lib --no-verify

nextest:
    ~/.cargo/bin/cargo-nextest run --workspace

codegen:
    {{ CARGO }} run -p ta-lib-codegen -- --manifest-root .
