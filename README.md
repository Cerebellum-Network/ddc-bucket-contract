# DDC v2 Smart Contracts

These smart contracts orchestrate the DDC network around clusters and buckets.

**[Background information](https://docs.cere.network/ddc/protocols/topology)**

**[Documentation homepage](https://docs.cere.network/ddc/protocols/smart-contracts)**

## Dependencies

Note: the substrate node version may be upgraded and the contract may not deploy, so if this is the case you can check if the ink version in [Cargo.toml](https://github.com/Cerebellum-Network/ddc-bucket-contract/blob/main/bucket/Cargo.toml) is supported by the current node version


```bash
# configure rust toolchain
rustup toolchain install nightly

rustup component add rust-src --toolchain nightly
rustup target add wasm32-unknown-unknown --toolchain nightly

cargo install cargo-contract --version 1.5.0 --force --locked

# Install binaryen in a version >= 99
brew install binaryen # apt-get install binaryen

# Install the documentation generator
git clone https://github.com/Cerebellum-Network/ink-doc-gen.git
(cd ink-doc-gen && git checkout v0.1.0 && yarn)
```

## How to build

Note: this contract can be build with errors due to different processor architecture, so we use docker container for unified build process. It's not necessary to use build through docker all the time, but it is important in order if you want to deploy the contract

```bash
# contract testing and building
cargo test &&
cargo contract build --release --manifest-path bucket/Cargo.toml

# build docker image
docker build -t ink-contract-builder .

# build the bucket contract
docker run --cpus=4 --memory=8g --rm -it -v $(pwd):/sources ink-contract-builder cargo +nightly contract build --manifest-path=/sources/bucket/Cargo.toml

```

## Deployment

Generated artifacts will be located in the target folder. Go to [EDC-1](https://explorer.cere.network/?rpc=wss%3A%2F%2Fext-devs-node-1.cluster-1.cere.network%3A9945#/explorer), select the developer tab and click the Upload & deploy code button, then deploy the contract.

Envs tested with deploy:
- [QAnet](https://explorer.cere.network/?rpc=wss%3A%2F%2Farchive.qanet.cere.network%2Fws#/contracts)
- [Dev](https://explorer.cere.network/?rpc=wss%3A%2F%2Farchive.devnet.cere.network%2Fws#/contracts)
- [EDC-1](https://explorer.cere.network/?rpc=wss%3A%2F%2Fext-devs-node-1.cluster-1.cere.network%3A9945#/explorer)


## How to get artifacts:
* Run workflow for any branch and wait until build finished
* Run the command to pull the artifacts:
```shell
aws ecr get-login-password --region us-west-2 --profile cere-network-dev | docker login --username AWS --password-stdin 625402836641.dkr.ecr.us-west-2.amazonaws.com
docker pull "625402836641.dkr.ecr.us-west-2.amazonaws.com/crb-smart-contracts:latest
id=$(docker create "625402836641.dkr.ecr.us-west-2.amazonaws.com/crb-smart-contracts:latest")
docker cp "$id":/contracts/target/ink/ddc_bucket/ ./
docker rm -v "$id"
```