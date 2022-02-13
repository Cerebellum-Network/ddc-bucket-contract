#!/usr/bin/env bash

set -eu

cargo +nightly test --workspace

cargo +nightly contract build --manifest-path bucket/Cargo.toml
cargo +nightly contract build --manifest-path cluster/Cargo.toml
cargo +nightly contract build --manifest-path payments/Cargo.toml
cargo +nightly contract build
