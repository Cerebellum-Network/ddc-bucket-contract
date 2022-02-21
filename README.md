# Usage

See the [JavaScript SDK](sdk/).

    yarn add @cere/ddc-contracts-sdk

# Contract Deployments

See known deployments in [deployments.yaml](deployments.yaml).

To deploy:
- Update the [SDK ABIs](sdk/src/abi/) using `target/ink/CONTRACT/metadata.json`.
- See the script [deploy.js](deploy.js).
- Update the [SDK default contracts](sdk/src/deployments.js).

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
