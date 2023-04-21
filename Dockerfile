# FROM rust:1.54 as builder

# RUN apt-get update && \
#     apt-get -y upgrade

# WORKDIR /contracts
# COPY . /contracts

# # Install binaryen
# RUN curl --silent https://api.github.com/repos/WebAssembly/binaryen/releases/41561408 | \
# 		egrep --only-matching 'https://github.com/WebAssembly/binaryen/releases/download/version_[0-9]+/binaryen-version_[0-9]+-x86_64-linux.tar.gz' | \
# 		head -n1 | \
# 		xargs curl -L -O && \
# 	tar -xvzf binaryen-version_*-x86_64-linux.tar.gz  && \
# 	rm binaryen-version_*-x86_64-linux.tar.gz && \
# 	chmod +x binaryen-version_*/bin/wasm-opt && \
# 	mv binaryen-version_*/bin/wasm-opt /usr/local/bin/ && \
# 	rm -rf binaryen-version_*/

# # Install cargo-contract
# RUN rustup toolchain install nightly-2022-06-28 && \
# 	rustup default nightly-2022-06-28 && \
# 	rustup component add rust-src --toolchain nightly-2022-06-28 && \
# 	rustup target add wasm32-unknown-unknown --toolchain nightly-2022-06-28 && \
# 	cargo install cargo-contract --version ^0.12 --force --locked

# # Run tests
# RUN cargo test --manifest-path bucket/Cargo.toml

# # Build contract
# RUN cargo contract build --manifest-path bucket/Cargo.toml

ARG VCS_REF=master
ARG BUILD_DATE=""
ARG REGISTRY_PATH=docker.io/paritytech

FROM ${REGISTRY_PATH}/base-ci-linux:latest

ARG RUST_NIGHTLY="2023-03-21"

WORKDIR /builds

RUN apt-get update && apt-get upgrade

RUN apt-get install -y binaryen

RUN rustup target add wasm32-unknown-unknown --toolchain stable && \
	rustup component add rust-src --toolchain stable && \
	rustup default stable && \

	# We also use the nightly toolchain for linting. We perform checks using RustFmt, and
	# Cargo Clippy.
	#
	# Note that we pin the nightly toolchain since it often creates breaking changes during
	# the RustFmt and Clippy stages of the CI.
	rustup toolchain install nightly-${RUST_NIGHTLY} --target wasm32-unknown-unknown \
		--profile minimal --component rustfmt clippy rust-src && \

	# Alias pinned toolchain as nightly, otherwise it appears as though we
	# don't have a nightly toolchain (i.e rustc +nightly --version is empty)
	ln -s "/usr/local/rustup/toolchains/nightly-${RUST_NIGHTLY}-x86_64-unknown-linux-gnu" \
		/usr/local/rustup/toolchains/nightly-x86_64-unknown-linux-gnu && \

	# `cargo-dylint` and `dylint-link` are dependencies needed to run `cargo-contract`.
	cargo install cargo-dylint dylint-link && \

	# Install the latest `cargo-contract`
	cargo install cargo-contract@1.5.0 && \

	# apt clean up
	apt-get autoremove -y && \
	apt-get clean && \
	rm -rf /var/lib/apt/lists/*
