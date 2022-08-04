# Documentation generator for Ink! smart contracts

## Install

    yarn

## Usage

Document functions and events using `///` comments.

Then compile, and run the documentation generator:

    cargo contract build --release

    ABI_PATH=target/ink/ddc_bucket/metadata.json \
    node ink-doc-gen/index.js


*Tested with Ink! 3.0.0-rc4.*

## ☟ Example Output ☟

---

# Contract my_contract 1.0.0

This is a smart contract.


## Functions

### create_thing (mutable, payable)

    fn create_thing(
        amount: u64,
    ) -> ThingId

Create a new thing and return its `thing_id`.


## Events


### ThingCreated

    event ThingCreated(
        thing_id: ThingId, (indexed)
        amount: u64,
    )

A thing was created.
