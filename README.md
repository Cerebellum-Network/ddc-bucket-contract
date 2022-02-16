# Deployments

See known deployments in [deployments.yaml](deployments.yaml).

To deploy, see the sub-project [deployer/](deployer/).

# Setup
    
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
