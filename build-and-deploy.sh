#!/usr/bin/env bash

set -eu

cargo +nightly-2023-02-07 test

cargo +nightly-2023-02-07 contract build --release --manifest-path bucket/Cargo.toml
# cargo +nightly-2023-02-07 contract build --release --manifest-path ddc_nft_registry/Cargo.toml

cp target/ink/ddc_bucket/metadata.json scripts/sdk/src/abi/ddc_bucket.json
# cp target/ink/ddc_nft_registry/metadata.json scripts/sdk/src/abi/ddc_nft_registry.json

# yarn --cwd scripts deploy-ddc-bucket
# yarn --cwd scripts deploy-ddc-nft-registry