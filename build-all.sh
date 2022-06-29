#!/usr/bin/env bash

set -eu

cargo +nightly test

cargo +nightly contract build --release --manifest-path bucket/Cargo.toml
cargo +nightly contract build --release --manifest-path ddc_nft_registry/Cargo.toml

cp target/ink/ddc_bucket/metadata.json sdk/src/abi/ddc_bucket.json
cp target/ink/ddc_nft_registry/metadata.json sdk/src/abi/ddc_nft_registry.json

node deploy.js
