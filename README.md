# DDC v2 Smart Contracts

These smart contracts orchestrate the DDC network around clusters and buckets.

**[Background information](https://docs.cere.network/ddc/protocols/topology)**

**[Documentation homepage](https://docs.cere.network/ddc/protocols/smart-contracts)**

## Dependencies

Note: The ink! version specidied in the [Cargo.toml](https://github.com/Cerebellum-Network/ddc-bucket-contract/blob/main/bucket/Cargo.toml) must be compatible with the `pallet_contracts` version, which in its turn depends on the underlying substrate version. Currently, the Devnet, the QAnet and EDC environments support `ink! 3.4.0`

```bash
# Configure the compatible rust toolchain
rustup toolchain install nightly-2023-02-07
rustup component add rust-src --toolchain nightly-2023-02-07
rustup target add wasm32-unknown-unknown --toolchain nightly-2023-02-07

# Install cargo-contract with its dependencies
cargo install cargo-dylint
cargo install dylint-link
cargo install cargo-contract --version 1.5.0 --force --locked

# Install binaryen in a version >= 99
brew install binaryen 
# For Debian\Ubuntu:
# apt-get install binaryen
```

## Building

```bash
# Build DDC Bucket contract
cargo +nightly-2023-02-07 test &&
cargo +nightly-2023-02-07 contract build --release --manifest-path bucket/Cargo.toml
```

Note: if you are encountering errors during build process, they may be related to your specific processor's architecture. If this is the case, try out the *Instalation using Docker image* option, [described in the official docs](https://github.com/paritytech/cargo-contract#installation-using-docker-image)


## Deployment

Generated artifacts will be located in the `./target/ink` folder. Go to [Devnet](https://explorer.cere.network/?rpc=wss%3A%2F%2Farchive.devnet.cere.network%2Fws#/contracts), select the `Developer -> Contracts` tab, and click the `Upload & deploy code button`. Use the `.contract` file for deployment.

Additionally, there are scripts that allow you to deploy contracts from your local environment to remote network. [Check the guide](./scripts/README.md) for more details.

Environment tested with deploy:
- [EDC-1](https://explorer.cere.network/?rpc=wss%3A%2F%2Fext-devs-node-1.cluster-1.cere.network%3A9945#/contracts)
- [Devnet](https://explorer.cere.network/?rpc=wss%3A%2F%2Farchive.devnet.cere.network%2Fws#/contracts)
- [QAnet](https://explorer.cere.network/?rpc=wss%3A%2F%2Farchive.qanet.cere.network%2Fws#/contracts)
- [Testnet](https://explorer.cere.network/?rpc=wss%3A%2F%2Farchive.testnet.cere.network%2Fws#/contracts)