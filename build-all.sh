#!/usr/bin/env bash

set -eu

cargo +nightly test

cargo +nightly contract build --release --manifest-path bucket/Cargo.toml
cargo +nightly contract build --release --manifest-path ddc_nft_registry/Cargo.toml
