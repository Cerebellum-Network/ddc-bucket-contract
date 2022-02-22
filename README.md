# Usage

See the [JavaScript SDK](sdk/).

    yarn add @cere/ddc-contracts-sdk

# Contract Deployments

See the latest deployments in [deployments.js](sdk/src/deployments.js). Use git tags to find previous versions.

To deploy:

- Update the version in the relevant `CONTRACT/Cargo.toml` and [package.json](sdk/package.json).
- Build the contracts: `cd CONTRACT && cargo test && cargo contract build`
- Use the script [deploy.js](deploy.js) to deploy the contracts.
- Update the [SDK ABIs](sdk/src/abi/) using `target/ink/CONTRACT/metadata.json`.
- Update the [SDK default contracts](sdk/src/deployments.js).
- Publish the JS SDK (this requires an `NPM_TOKEN`): `cd sdk && npm publish`

# Contract Development

    rustup install nightly-2021-12-05
    rustup component add rust-src --toolchain nightly-2021-12-05
    rustup target add wasm32-unknown-unknown --toolchain nightly-2021-12-05
    cargo install cargo-contract --version ^0.14 --force --locked

    # Install binaryen in a version >= 99
    #apt-get install binaryen
    #brew install binaryen

# Test

    # Fast test
    cargo test

    # Long test
    ./build-all.sh
