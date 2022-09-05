# DDC v2 Smart Contracts

These smart contracts orchestrate the DDC network around clusters and buckets.

**[Background information](https://docs.cere.network/ddc/protocols/topology)**

**[Documentation homepage](https://docs.cere.network/ddc/protocols/smart-contracts)**

## Usage

See the [JavaScript SDK](sdk/).

    yarn add @cere-ddc-sdk/smart-contract

## Contract Deployments

See the latest deployments in [deployments.js](sdk/src/deployments.js). Use git tags to find previous versions.

To deploy:

- Update the version in the relevant [Cargo.toml](bucket/Cargo.toml) and [package.json](sdk/package.json).
- Build the contracts:
```bash 
cargo test && 
cargo contract build --release --manifest-path bucket/Cargo.toml && 
cargo contract build --release --manifest-path ddc_nft_registry/Cargo.toml
```
- Update the [SDK ABIs](sdk/src/abi/) using
```bash
cp target/ink/ddc_bucket/metadata.json sdk/src/abi/ddc_bucket.json &&
cp target/ink/ddc_nft_registry/metadata.json sdk/src/abi/ddc_nft_registry.json
```
- Use the script [deploy.js](deploy.js) or the [Explorer](https://explorer.cere.network/) to deploy the contracts.
- Update the [Admin SDK default contracts](sdk/src/deployments.js).
- Publish the Admin SDK (this requires an `NPM_TOKEN`): `cd sdk && npm publish`
- Similarly, update the [DDC SDK](https://github.com/Cerebellum-Network/cere-ddc-sdk-js) for apps.
- Regenerate the documentation, then sync [docs.cere.network](https://github.com/Cerebellum-Network/docs.cere.network/blob/main/ddc/protocols/smart-contract-api.md):
```bash
ABI_PATH=target/ink/ddc_bucket/metadata.json  node ink-doc-gen
```

## Development Setup

    rustup install nightly-2022-06-28
    rustup component add rust-src --toolchain nightly-2022-06-28
    rustup target add wasm32-unknown-unknown --toolchain nightly-2022-06-28
    cargo install cargo-contract --version ^0.14 --force --locked

    # Install binaryen in a version >= 99
    #apt-get install binaryen
    #brew install binaryen

    # Install the documentation generator
    git clone https://github.com/Cerebellum-Network/ink-doc-gen.git
    (cd ink-doc-gen && git checkout v0.1.0 && yarn)

See also some information about the code structure: [ink-tips](https://github.com/Cerebellum-Network/ink-tips)

See also the custom documentation generator: [ink-doc-gen](https://github.com/Cerebellum-Network/ink-doc-gen)

## Test

    # Fast test off-chain
    cargo test

    # Long test after an on-chain deployment (see instructions above)
    node demo.js

    # Visualize the state of the network.
    # Read this .md file, ideally with a Markdown and Mermaid editor (e.g., VSCode with Markdown Preview Mermaid Support).
    node print_ddc_bucket.js > local/Network_state.md
