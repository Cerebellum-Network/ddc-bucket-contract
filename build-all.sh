#!/usr/bin/env bash

set -eu

cargo +nightly contract build --manifest-path cluster/Cargo.toml
cargo +nightly contract build --manifest-path payments/Cargo.toml
cargo +nightly contract build --manifest-path registry/Cargo.toml
cargo +nightly contract build
