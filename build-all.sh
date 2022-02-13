#!/usr/bin/env bash

set -eu

cargo +nightly test

cargo +nightly contract build --manifest-path billing/Cargo.toml
cargo +nightly contract build --manifest-path bucket/Cargo.toml
cargo +nightly contract build --manifest-path cluster/Cargo.toml
