FROM rust:1.54 as builder

RUN apt-get update && \
    apt-get -y upgrade

WORKDIR /contracts
COPY . /contracts

# Install binaryen
RUN curl --silent https://api.github.com/repos/WebAssembly/binaryen/releases/41561408 | \
		egrep --only-matching 'https://github.com/WebAssembly/binaryen/releases/download/version_[0-9]+/binaryen-version_[0-9]+-x86_64-linux.tar.gz' | \
		head -n1 | \
		xargs curl -L -O && \
	tar -xvzf binaryen-version_*-x86_64-linux.tar.gz  && \
	rm binaryen-version_*-x86_64-linux.tar.gz && \
	chmod +x binaryen-version_*/bin/wasm-opt && \
	mv binaryen-version_*/bin/wasm-opt /usr/local/bin/ && \
	rm -rf binaryen-version_*/

# Install cargo-contract
RUN rustup toolchain install nightly-2022-06-28 && \
	rustup default nightly-2022-06-28 && \
	rustup component add rust-src --toolchain nightly-2022-06-28 && \
	rustup target add wasm32-unknown-unknown --toolchain nightly-2022-06-28 && \
	cargo install cargo-contract --version ^0.12 --force --locked

# Run tests
RUN cargo test --manifest-path bucket/Cargo.toml

# Build contract
RUN cargo contract build --manifest-path bucket/Cargo.toml
